use anyhow;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{sync::oneshot, time::Sleep};

use crate::AppEventManager;

/// An `AppEventManager` implementation designed for testing.
///
/// It captures events emitted by the application core and funnels them to
/// `EventWatcher` futures within a test step, instead of sending them to the UI.
#[derive(Debug, Clone)]
pub struct TestEventManager {
    listeners: Arc<DashMap<String, Vec<oneshot::Sender<Value>>>>,
}

impl Default for TestEventManager {
    fn default() -> Self {
        Self {
            listeners: Arc::new(DashMap::new()),
        }
    }
}

impl AppEventManager for TestEventManager {
    fn emit_all(&self, event: &str, payload: Value) -> anyhow::Result<()> {
        // If listeners are registered for this event, send the payload to them.
        if let Some((_, senders)) = self.listeners.remove(event) {
            for sender in senders {
                // The receiver might have been dropped if the test timed out,
                // so we ignore the result of send.
                let _ = sender.send(payload.clone());
            }
        }
        Ok(())
    }
}

impl TestEventManager {
    /// Registers a one-time listener for a specific event.
    pub fn listen_for<T: DeserializeOwned>(
        &self,
        event_name: &str,
        timeout: Duration,
    ) -> EventWatcher<T> {
        let (sender, receiver) = oneshot::channel();

        self.listeners
            .entry(event_name.to_string())
            .or_default()
            .push(sender);

        EventWatcher {
            receiver,
            event_name: event_name.to_string(),
            timeout,
            deadline: None,
            _marker: PhantomData,
        }
    }
}

/// A handle to a registered event listener.
///
/// This struct implements `Future`. A test can `.await` this future to wait
/// for the application to emit a specific event.
///
/// The future has a built-in timeout. If the event is not received
/// within the configured duration, the future will resolve to an error.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct EventWatcher<T: DeserializeOwned> {
    receiver: oneshot::Receiver<Value>,
    event_name: String,
    timeout: Duration,
    deadline: Option<Pin<Box<Sleep>>>,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned + Unpin> Future for EventWatcher<T> {
    type Output = anyhow::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Initialize deadline if not already set
        if this.deadline.is_none() {
            this.deadline = Some(Box::pin(tokio::time::sleep(this.timeout)));
        }

        // First, poll the actual receiver for the event payload
        match Pin::new(&mut this.receiver).poll(cx) {
            Poll::Ready(Ok(payload)) => {
                let typed_payload: T = serde_json::from_value(payload)?;
                return Poll::Ready(Ok(typed_payload));
            }
            Poll::Ready(Err(_)) => {
                return Poll::Ready(Err(anyhow::anyhow!(
                    "EventWatcher channel closed before event '{}' arrived",
                    this.event_name
                )));
            }
            Poll::Pending => { /* continue to poll deadline */ }
        }

        // If receiver is pending, poll the deadline
        if let Some(deadline) = &mut this.deadline
            && deadline.as_mut().poll(cx).is_ready()
        {
            return Poll::Ready(Err(anyhow::anyhow!(
                "Timed out waiting for event '{}'",
                this.event_name
            )));
        }

        Poll::Pending
    }
}
