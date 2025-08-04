use axum::{
    body::Body,
    extract::{MatchedPath, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::json;
use std::collections::HashMap;
use tauri::Emitter;
use tokio::time::Instant;

use crate::api_logs::db::ActiveModel as CreateApiLogs;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

use rand::{rng, Rng};

pub fn generate_request_id() -> String {
    let mut rng = rng();

    let part1: String = (0..4)
        .map(|_| rng.random_range(0..16))
        .map(|x| format!("{:x}", x))
        .collect();
    let part2: String = (0..4)
        .map(|_| rng.random_range(0..16))
        .map(|x| format!("{:x}", x))
        .collect();
    let part3: String = (0..4)
        .map(|_| rng.random_range(0..16))
        .map(|x| format!("{:x}", x))
        .collect();
    let part4: String = (0..4)
        .map(|_| rng.random_range(0..16))
        .map(|x| format!("{:x}", x))
        .collect();
    let part5: String = (0..16)
        .map(|_| rng.random_range(0..16))
        .map(|x| format!("{:x}", x))
        .collect();

    format!("{}-{}-{}-{}{}", part1, part2, part3, part4, part5)
}

use super::{ApiError, ApiState};

// Middleware function that captures all request/response data
pub async fn logging_middleware(
    State(state): State<ApiState>,
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();

    // Extract request information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let path = matched_path
        .as_ref()
        .map(|mp| mp.as_str())
        .unwrap_or(uri.path())
        .to_string();

    // Extract headers as HashMap
    let headers_map = extract_headers(&headers);
    let (request, request_body) = extract_request_body(request).await;
    let response = next.run(request).await;
    if path == "/" {
        return Ok(response);
    }

    let error_desc = response
        .extensions()
        .get::<ApiError>()
        .map(|api_error| api_error.internal_description.clone());

    let response_headers = response.headers().clone();
    let response_headers_map = extract_headers(&response_headers);
    let duration = start_time.elapsed();

    // Extract response information
    let status_code = response.status();
    let (response, response_body) = extract_response_body(response).await;

    let create_api_log = CreateApiLogs {
        project_id: Set(state.project_id),
        method: Set(method.to_string()),
        path: Set(path),
        status_code: Set(status_code.as_u16()),
        request_body: Set(Some(
            json!({
                "headers": headers_map,
                "body": request_body,
            })
            .to_string(),
        )),
        response_body: Set(Some(
            json!({
                "headers": response_headers_map,
                "body": response_body
            })
            .to_string(),
        )),
        duration: Set(duration.as_millis() as u64),
        error_desc: Set(error_desc),
        ..Default::default()
    };

    create_api_log.insert(&state.conn).await.map_err(|err| {
        println!("{}", err);

        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _ = state.handle.emit("new-api-log", state.project_id);

    Ok(response)
}

// Helper function to extract headers as HashMap
fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    let mut headers_map = HashMap::new();

    for (key, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            headers_map.insert(key.to_string(), value_str.to_string());
        }
    }

    headers_map
}

// Helper function to extract request body
async fn extract_request_body(request: Request) -> (Request, Option<String>) {
    let (parts, body) = request.into_parts();

    // Collect the body
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return (Request::from_parts(parts, Body::empty()), None),
    };

    // Convert to string if possible
    let body_string = if body_bytes.is_empty() {
        None
    } else {
        match String::from_utf8(body_bytes.to_vec()) {
            Ok(s) => Some(s),
            Err(_) => Some(format!("<binary data: {} bytes>", body_bytes.len())),
        }
    };

    let new_request = Request::from_parts(parts, Body::from(body_bytes));

    (new_request, body_string)
}

// Helper function to extract response body
async fn extract_response_body(response: Response) -> (Response, Option<String>) {
    let (parts, body) = response.into_parts();

    // Collect the body
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return (Response::from_parts(parts, Body::empty()), None),
    };

    // Convert to string if possible
    let body_string = if body_bytes.is_empty() {
        None
    } else {
        match String::from_utf8(body_bytes.to_vec()) {
            Ok(s) => Some(s),
            Err(_) => Some(format!("<binary data: {} bytes>", body_bytes.len())),
        }
    };

    let new_response = Response::from_parts(parts, Body::from(body_bytes));

    (new_response, body_string)
}
