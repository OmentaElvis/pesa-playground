use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::Context as AnyhowContext;
use axum::http::{self, HeaderMap, HeaderValue};
use axum::{
    Json, Router,
    body::{self, Body},
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::task::JoinHandle;
use tokio::{net::TcpListener, sync::oneshot, time::Sleep};
use tracing::error;

use crate::server::shutdown_signal;

/// A handle to a registered callback.
///
/// This struct implements `Future`. A test can `.await` this future to wait
/// for the application to make an HTTP request to the URL provided by `url()`.
///
/// The future has a built-in timeout. If the callback is not received
/// within the configured duration, the future will resolve to an error.
pub struct CallbackEntry<T: DeserializeOwned> {
    receiver: oneshot::Receiver<CallbackMessage>,
    url: String,
    timeout: Duration,
    deadline: Option<Pin<Box<Sleep>>>,
    _marker: PhantomData<T>,
}

#[derive(Debug)]
struct CallbackMessage {
    req: String,
    method: http::Method,
    headers: HeaderMap,
    respond: oneshot::Sender<CallbackResponse>,
}

/// The resolved state of a `CallbackEntry` future.
///
/// This struct contains the deserialized body and other details of the HTTP
/// request made by the application. It also provides a method to send a
/// response back to the application.
pub struct CallbackCall<T: DeserializeOwned> {
    /// The deserialized JSON body of the incoming callback request.
    pub body: T,
    /// The HTTP method of the incoming request.
    pub method: http::Method,
    /// The headers of the incoming request.
    pub headers: HeaderMap,
    respond: Option<oneshot::Sender<CallbackResponse>>,
}

#[derive(Debug)]
struct CallbackResponse {
    body: String,
    status: StatusCode,
    headers: HeaderMap,
}

/// Manages an internal Axum server for handling one-time, expected HTTP callbacks during tests.
///
/// This is a core part of the testing framework, allowing tests to provide
/// a temporary URL to the application and wait for the application to call it.
#[derive(Debug)]
pub struct CallbackManager {
    registry: Arc<DashMap<String, oneshot::Sender<CallbackMessage>>>,
    base_url: String,
    /// The `JoinHandle` for the background server task.
    pub server_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    callback_timeout: Duration,
}

impl CallbackManager {
    pub async fn new(callback_timeout: Duration) -> anyhow::Result<Self> {
        // Bind to a random available port
        let listener = TcpListener::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)).await?;
        let port = listener.local_addr()?.port();
        let base_url = format!("http://127.0.0.1:{}", port);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let registry = Arc::new(DashMap::new());

        let server_handle = tokio::spawn(Self::start_server(
            listener,
            shutdown_rx,
            Arc::clone(&registry),
        ));

        Ok(Self {
            base_url,
            registry,
            server_handle: Some(server_handle),
            shutdown_tx: Some(shutdown_tx),
            callback_timeout,
        })
    }
    async fn start_server(
        listener: TcpListener,
        shutdown_rx: oneshot::Receiver<()>,
        registry: Arc<DashMap<String, oneshot::Sender<CallbackMessage>>>,
    ) {
        let app = Router::new()
            .fallback(Self::handle_callback_request)
            .with_state(registry);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(shutdown_rx))
            .await
            .expect("Callback server failed");
    }

    async fn handle_callback_request(
        State(registry): State<Arc<DashMap<String, oneshot::Sender<CallbackMessage>>>>,
        req: Request<Body>,
    ) -> Response {
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        let content_type = req.headers().get(CONTENT_TYPE);
        let headers = req.headers().clone();

        // lets just enforce content type since we dont expect anything else
        if let Some(typ) = content_type
            && (typ != "application/json")
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid content type"})),
            )
                .into_response();
        }

        let bytes = match body::to_bytes(req.into_body(), 32 * 1024).await {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to read callback body: {:?}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to read body"})),
                )
                    .into_response();
            }
        };

        let body = match String::from_utf8((*bytes).to_vec()) {
            Ok(b) => b,
            Err(err) => {
                error!("Failed to parse callback body as string: {:?}", err);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid JSON body"})),
                )
                    .into_response();
            }
        };

        let (tx, rx) = oneshot::channel();

        // Use DashMap's remove method which returns an Option<OwnedEntry>
        if let Some((_, sender)) = registry.remove(&path) {
            // Signal the waiting test that the callback was received and processed
            let _ = sender.send(CallbackMessage {
                req: body,
                respond: tx,
                method,
                headers,
            });

            match rx.await {
                Ok(res) => {
                    let mut response = Response::new(res.body);
                    *response.status_mut() = res.status;
                    *response.headers_mut() = res.headers;

                    response.into_response()
                }
                Err(_err) => (StatusCode::NO_CONTENT,).into_response(),
            }
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("No handler registered for path '{}'", path)})),
            )
                .into_response()
        }
    }
    /// Provides a URL for a callback path.
    pub fn get_callback_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Registers a handler for a one-time callback at a specific path.
    ///
    /// This returns a `CallbackEntry`, which includes the full URL for the
    /// callback and can be `.await`ed.
    ///
    /// The generic `T` is the type that the test expects the application to
    /// send as a JSON body. It must implement `serde::DeserializeOwned`.
    pub fn register_callback<T: DeserializeOwned>(
        &self,
        path: impl Into<String>,
    ) -> anyhow::Result<CallbackEntry<T>> {
        let path_str = path.into();
        let (sender, receiver) = oneshot::channel();

        // Check if handler already exists
        match self.registry.entry(path_str.clone()) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                // An entry already exists. Check if the existing sender is closed.
                if entry.get().is_closed() {
                    // Previous receiver was dropped without being awaited.
                    // Replace the old sender with the new one.
                    entry.insert(sender);
                } else {
                    // Previous receiver is still active, this is a legitimate conflict.
                    return Err(anyhow::anyhow!(
                        "Callback handler already registered for path '{}' and is still active",
                        path_str
                    ));
                }
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // No entry exists, simply insert the new sender.
                entry.insert(sender);
            }
        }

        let url = self.get_callback_url(&path_str);
        Ok(CallbackEntry {
            receiver,
            url,
            timeout: self.callback_timeout,
            deadline: None,
            _marker: PhantomData,
        })
    }
    /// Checks if the internal Axum server has panicked. If it has, this
    /// method will return an error. If the server finished gracefully,
    /// it consumes the handle.
    pub async fn check_server_panic(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.server_handle.as_mut()
            && handle.is_finished()
        {
            // If it's finished, take it to await it and check for panic
            let owned_handle = self.server_handle.take().expect("Handle should be Some");
            if owned_handle.await.is_err() {
                return Err(anyhow::anyhow!("Callback server panicked"));
            }
        }
        Ok(())
    }
}

impl Drop for CallbackManager {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).ok();
        }

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

impl<T: DeserializeOwned> CallbackCall<T> {
    fn from_message(message: CallbackMessage) -> anyhow::Result<CallbackCall<T>> {
        let body: T = serde_json::from_str(&message.req)
            .context("Failed to deserialize request body to json")?;

        Ok(CallbackCall {
            body,
            method: message.method,
            headers: message.headers,
            respond: Some(message.respond),
        })
    }

    /// Sends a JSON response back to the application that made the callback request.
    pub async fn respond<R: Serialize>(
        mut self,
        status: StatusCode,
        response: &R,
        headers: Option<HeaderMap>,
    ) -> anyhow::Result<()> {
        let body =
            serde_json::to_string(response).context("Failed to serialize response as json")?;

        let mut headers = headers.unwrap_or_default();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);

        if let Some(sender) = self.respond.take() {
            sender
                .send(CallbackResponse {
                    body,
                    status,
                    headers,
                })
                .map_err(|err| anyhow::anyhow!("Failed to send response: {:?}", err))?;
        }

        Ok(())
    }
}

impl<T: DeserializeOwned + Unpin> Future for CallbackEntry<T> {
    type Output = anyhow::Result<CallbackCall<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Initialize deadline if not already set
        if this.deadline.is_none() {
            this.deadline = Some(Box::pin(tokio::time::sleep(this.timeout)));
        }

        // First, poll the actual receiver for the callback message
        match Pin::new(&mut this.receiver).poll(cx) {
            Poll::Ready(Ok(msg)) => {
                return Poll::Ready(CallbackCall::from_message(msg));
            }
            Poll::Ready(Err(_)) => {
                return Poll::Ready(Err(anyhow::anyhow!(
                    "Callback channel closed before request arrived"
                )));
            }
            Poll::Pending => { /* continue to poll deadline */ }
        }

        // If receiver is pending, poll the deadline
        if let Some(deadline) = &mut this.deadline
            && deadline.as_mut().poll(cx).is_ready()
        {
            return Poll::Ready(Err(anyhow::anyhow!(
                "Timed out waiting for callback at {}",
                this.url
            )));
        }

        Poll::Pending
    }
}

impl<T: DeserializeOwned> CallbackEntry<T> {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}
