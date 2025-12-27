//! A generic, type-safe framework for handling asynchronous M-Pesa API requests.
use std::fmt::Debug;

use axum::{Json, extract::State, http::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};
use tokio::task;
use tracing;

use super::{ApiError, ApiState};
use crate::{
    api_keys::ApiKey,
    callbacks::{CreateCallbackParams, orchestrator::CallbackOrchestrator},
    server::{api::auth, log::generate_conversation_id},
};

pub trait IntoCallbackPayload<C, T> {
    fn get_payload(&self, ctx: &C) -> T;
}

/// A generic trait for defining a two-step, stateful asynchronous API operation.
pub trait PpgAsyncRequest: Sized + Send + 'static {
    /// The struct for the incoming request from the client.
    type RequestData: DeserializeOwned;
    /// The struct for the immediate response sent back to the client.
    type SyncResponseData: Serialize;
    /// The struct for the final callback payload.
    type CallbackPayload: Serialize + Send + Sync;
    type Error: Send + Debug + IntoCallbackPayload<Self, Self::CallbackPayload>;

    /// Validates the request
    /// This runs in the synchronous context of the initial request.
    fn init(
        state: &ApiState,
        req: Self::RequestData,
        conversation_id: &str,
        api_key: ApiKey,
    ) -> impl std::future::Future<Output = Result<(Self::SyncResponseData, Self), ApiError>> + Send
    where
        Self: Sized;

    /// Executes the core business logic.
    /// This runs in a background task.
    fn execute(
        &mut self,
        state: &ApiState,
    ) -> impl std::future::Future<Output = Result<Self::CallbackPayload, Self::Error>> + Send;

    /// Gets the callback URL from the state stored in `self`.
    fn get_callback_url(&self) -> Option<&str> {
        None
    }

    /// Gets the externally provided originator ID from the state stored in `self`.
    fn get_originator_id(&self) -> &str;

    /// Extracts the optional M-Pesa transaction ID from the final callback payload.
    fn get_transaction_id(_payload: &Self::CallbackPayload) -> Option<String> {
        None
    }

    /// Gets the static name of the API for logging purposes.
    fn api_name() -> &'static str;
}

/// A generic Axum handler that processes any request implementing the `MpesaRequest` trait.
pub async fn handle_async_request<T: PpgAsyncRequest>(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(req_data): Json<T::RequestData>,
) -> Result<Json<T::SyncResponseData>, ApiError> {
    let api_key = auth::validate_bearer_token(&headers, &state).await?;
    let conversation_id = generate_conversation_id();

    let (sync_response, job) = T::init(&state, req_data, conversation_id.as_str(), api_key).await?;
    spawn_async_job::<T>(state, conversation_id, job);

    Ok(Json(sync_response))
}

/// Spawns a background Tokio task to run the `execute` step on the job object.
fn spawn_async_job<T: PpgAsyncRequest>(state: ApiState, conversation_id: String, mut job: T) {
    task::spawn(async move {
        tracing::trace!(
            "Starting async job for {} on project {}. Conversation Id: {}",
            T::api_name(),
            state.project_id,
            conversation_id
        );

        // Execute the core business logic using the state held by the job object.
        let result = job.execute(&state).await;

        // Determine the final payload (success or error).
        let final_payload = match result {
            Ok(success_payload) => success_payload,
            Err(e) => {
                tracing::error!(
                    "Asynchronous execution failed for {} ({}): {:?}",
                    T::api_name(),
                    conversation_id,
                    e,
                );
                e.get_payload(&job)
            }
        };

        if let Some(url) = job.get_callback_url() {
            let params = CreateCallbackParams {
                project_id: state.project_id,
                callback_type: T::api_name().parse().unwrap_or_default(),
                url: url.to_string(),
                conversation_id,
                originator_id: job.get_originator_id().to_string(),
                payload: serde_json::to_value(&final_payload).unwrap_or_default(),
                transaction_id: T::get_transaction_id(&final_payload),
            };

            // Delegate the entire callback lifecycle to the orchestrator.
            CallbackOrchestrator::handle_callback(&state.context, params, final_payload).await;
        }

        tracing::trace!(
            "Finished async job for {} on project {}.",
            T::api_name(),
            state.project_id
        );
    });
}
