use chrono::Utc;
use sea_orm::{prelude::DateTimeUtc, ActiveModelTrait, ActiveValue::Set, ConnectionTrait};
use serde::{Deserialize, Serialize};

use crate::server::log::generate_request_id;

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiLog {
    pub id: String,
    pub project_id: u32,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub created_at: DateTimeUtc,
    pub error_desc: Option<String>,
    pub duration: u32,
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

// Helper struct for API log statistics
#[derive(serde::Serialize)]
pub struct ApiLogStats {
    pub total_count: i64,
    pub success_count: i64,
    pub client_error_count: i64,
    pub server_error_count: i64,
}

impl From<db::Model> for ApiLog {
    fn from(value: db::Model) -> Self {
        Self {
            id: value.id,
            project_id: value.project_id,
            method: value.method,
            path: value.path,
            status_code: value.status_code,
            request_body: value.request_body,
            response_body: value.response_body,
            created_at: value.created_at,
            error_desc: value.error_desc,
            duration: value.duration,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiLogBuilder {
    id: Option<String>,
    project_id: Option<u32>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<u16>,
    request_body: Option<String>,
    response_body: Option<String>,
    created_at: Option<DateTimeUtc>,
    error_desc: Option<String>,
    duration: Option<u32>,
}

impl ApiLogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn project_id(mut self, project_id: u32) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn method<S: Into<String>>(mut self, method: S) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn request_body<S: Into<String>>(mut self, request_body: S) -> Self {
        self.request_body = Some(request_body.into());
        self
    }

    pub fn response_body<S: Into<String>>(mut self, response_body: S) -> Self {
        self.response_body = Some(response_body.into());
        self
    }

    pub fn created_at(mut self, created_at: DateTimeUtc) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn error_desc<S: Into<String>>(mut self, error_desc: S) -> Self {
        self.error_desc = Some(error_desc.into());
        self
    }

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub async fn save<C: ConnectionTrait>(self, conn: &C) -> anyhow::Result<ApiLog> {
        let api = ApiLog {
            id: self.id.unwrap_or(generate_request_id()),
            project_id: self
                .project_id
                .ok_or(ApiLogBuilderError::MissingField("project_id"))?,
            method: self
                .method
                .ok_or(ApiLogBuilderError::MissingField("method"))?,
            path: self.path.ok_or(ApiLogBuilderError::MissingField("path"))?,
            status_code: self
                .status_code
                .ok_or(ApiLogBuilderError::MissingField("status_code"))?,
            request_body: self.request_body,
            response_body: self.response_body,
            created_at: self.created_at.unwrap_or(Utc::now().to_utc()),
            error_desc: self.error_desc,
            duration: self
                .duration
                .ok_or(ApiLogBuilderError::MissingField("duration"))?,
        };

        let create_api = db::ActiveModel {
            id: Set(api.id.clone()),
            project_id: Set(api.project_id),
            method: Set(api.method.clone()),
            path: Set(api.path.clone()),
            status_code: Set(api.status_code),
            request_body: Set(api.request_body.clone()),
            response_body: Set(api.response_body.clone()),
            created_at: Set(api.created_at),
            error_desc: Set(api.error_desc.clone()),
            duration: Set(api.duration),
        };

        create_api.insert(conn).await?;

        Ok(api)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiLogBuilderError {
    MissingField(&'static str),
}

impl std::fmt::Display for ApiLogBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiLogBuilderError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
        }
    }
}

impl std::error::Error for ApiLogBuilderError {}

impl ApiLog {
    pub fn builder() -> ApiLogBuilder {
        ApiLogBuilder::new()
    }
}
