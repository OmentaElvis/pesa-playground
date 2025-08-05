use std::time::Duration;

use rand::Rng;
use reqwest::Client;
use serde_json::{json, Value};
use tauri::Emitter;
use tokio::time::{sleep, Instant};

use crate::{api_logs::ApiLog, server::ApiState};

use super::stk::StkCodes;

pub async fn return_body(
    state: &ApiState,
    status: StkCodes,
    url: String,
    merchant_id: String,
    checkout_id: String,
    metadata: Option<Value>,
) {
    let mut callback = json!({
        "MerchantRequestID": merchant_id,
        "CheckoutRequestID": checkout_id,
        "ResultCode": status.code(),
        "ResultDesc": status.message(),
    });

    if status.code() == 0 {
        // Only attach CallbackMetadata on success
        let callback_metadata = match metadata {
            Some(Value::Array(items)) => {
                json!({ "Item": items })
            }
            Some(Value::Object(map)) => {
                // Convert object to array format
                let item_vec: Vec<_> = map
                    .into_iter()
                    .map(|(k, v)| json!({ "Name": k, "Value": v }))
                    .collect();
                json!({ "Item": item_vec })
            }
            _ => Value::Null,
        };

        if !callback_metadata.is_null() {
            callback
                .as_object_mut()
                .unwrap()
                .insert("CallbackMetadata".to_string(), callback_metadata);
        }
    }

    let body_json = json!({
        "Body": {
            "stkCallback": callback
        }
    });

    // POST the callback
    let client = Client::new();

    const MAX_ATTEMPTS: usize = 4;
    for attempt in 1..=MAX_ATTEMPTS {
        let start = Instant::now();
        let result = client.post(&url).json(&body_json).send().await;
        let duration = start.elapsed();
        let db = &state.conn;

        match result {
            Ok(resp) => {
                let status_code = resp.status();
                let response_headers_map = headers_to_map(resp.headers());
                let response_body = resp.text().await.unwrap_or_default();

                if status_code.is_success() {
                    println!("[CALLBACK] Delivered successfully on attempt {attempt}");
                    if let Err(err) = ApiLog::builder()
                        .project_id(state.project_id)
                        .method("POST")
                        .path(url.clone())
                        .status_code(status_code.as_u16())
                        .request_body(
                            json! ({
                                "headers": {
                                    "content-type": "application/json"
                                },
                                "body": body_json
                            })
                            .to_string(),
                        )
                        .response_body(
                            json!({
                                "headers": response_headers_map,
                                "body": response_body
                            })
                            .to_string(),
                        )
                        .duration(duration.as_millis() as u32)
                        .save(db)
                        .await
                    {
                        eprintln!("Api log save error: {}", err);
                    }

                    let _ = state.handle.emit("new-api-log", state.project_id);
                    return;
                } else {
                    if let Err(err) = ApiLog::builder()
                        .project_id(state.project_id)
                        .method("POST")
                        .path(url.clone())
                        .status_code(status_code.as_u16())
                        .request_body(
                            json! ({
                                "headers": {
                                    "content-type": "application/json"
                                },
                                "body": body_json
                            })
                            .to_string(),
                        )
                        .response_body(
                            json!({
                                "headers": response_headers_map,
                                "body": response_body
                            })
                            .to_string(),
                        )
                        .duration(duration.as_millis() as u32)
                        .error_desc(format!("Non-2xx callback response (attempt {attempt})"))
                        .save(db)
                        .await
                    {
                        eprintln!("Api log save error: {}", err);
                    }

                    let _ = state.handle.emit("new-api-log", state.project_id);
                }
            }

            Err(err) => {
                eprintln!("[CALLBACK] Attempt {attempt} failed: {err}");
                if let Err(err) = ApiLog::builder()
                    .project_id(state.project_id)
                    .method("POST")
                    .path(url.clone())
                    .status_code(0)
                    .request_body(
                        json! ({
                            "headers": {
                                "content-type": "application/json"
                            },
                            "body": body_json
                        })
                        .to_string(),
                    )
                    .duration(duration.as_millis() as u32)
                    .error_desc(format!("Non-2xx callback response (attempt {attempt})"))
                    .save(db)
                    .await
                {
                    eprintln!("Api log save error: {}", err);
                }

                let _ = state.handle.emit("new-api-log", state.project_id);
            }
        }

        // Exponential backoff with jitter
        let backoff_ms = 2_u64.pow(attempt as u32) * 1000 + rand::rng().random_range(0..500);
        sleep(Duration::from_millis(backoff_ms)).await;
    }
    eprintln!("[CALLBACK] Final failure after {MAX_ATTEMPTS} attempts to {url}");
}

fn headers_to_map(headers: &reqwest::header::HeaderMap) -> serde_json::Map<String, Value> {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                Value::String(v.to_str().unwrap_or_default().to_string()),
            )
        })
        .collect()
}
