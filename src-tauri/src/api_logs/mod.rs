use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLog {
    pub id: String,
    pub project_id: u32,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub error_desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateApiLogRequest {
    pub project_id: i64,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub duration: u128,
    pub error_desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateApiLogRequest {
    pub status_code: Option<u16>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_desc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiLogFilter {
    pub project_id: Option<i64>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub status_code: Option<i32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for ApiLogFilter {
    fn default() -> Self {
        Self {
            project_id: None,
            method: None,
            path: None,
            status_code: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

// Helper struct for API log statistics
#[derive(serde::Serialize)]
pub struct ApiLogStats {
    pub total_count: i64,
    pub success_count: i64,
    pub client_error_count: i64,
    pub server_error_count: i64,
}
