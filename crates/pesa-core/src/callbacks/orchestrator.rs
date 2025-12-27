use super::{CallbackLog, CreateCallbackParams, DispatchOutcome};
use crate::{
    AppContext,
    callbacks::dispatch::{CallbackDispatchService, DispatchConfig},
};
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

pub struct CallbackOrchestrator;

impl CallbackOrchestrator {
    pub async fn handle_callback<T: Serialize>(
        context: &AppContext,
        params: CreateCallbackParams,
        payload: T,
    ) {
        // Create and save a "Pending" callback record.
        let saved_log = match CallbackLog::create(&context.db, params).await {
            Ok(log) => log,
            Err(e) => {
                tracing::error!("Failed to insert pending callback into database: {:?}", e);
                return;
            }
        };

        // Dispatch the callback.
        let dispatch_service = CallbackDispatchService::new(DispatchConfig {
            timeout: Duration::from_secs(30),
            max_retries: 2,
        });

        let dispatch_result = dispatch_service
            .dispatch(&saved_log.callback_url, &payload)
            .await;

        // Update the database record with the outcome.
        let outcome = match dispatch_result {
            Ok(res) => DispatchOutcome::Delivered {
                status_code: res.final_status_code,
                headers: json!(
                    res.final_headers
                        .iter()
                        .map(|(name, value)| {
                            (
                                name.to_string(),
                                value.to_str().unwrap_or_default().to_string(),
                            )
                        })
                        .collect::<HashMap<String, String>>()
                ),
                body: res.final_body,
            },
            Err(e) => DispatchOutcome::Failed {
                error_message: e.to_string(),
            },
        };

        if let Err(e) = saved_log.update_dispatch_status(&context.db, outcome).await {
            tracing::error!("Failed to update callback status in database: {:?}", e);
        }
    }
}
