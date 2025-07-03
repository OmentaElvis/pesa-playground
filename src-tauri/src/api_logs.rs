use chrono::{DateTime, Utc};
use sea_query::{ColumnDef, Expr, Query, SqliteQueryBuilder, Table};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row, SqlitePool};
use tauri::State;

use crate::{db::Database, server::log::generate_request_id};

#[derive(sea_query::Iden)]
pub enum ApiLogs {
    Table,
    Id,
    ProjectId,
    Method,
    Path,
    StatusCode,
    RequestBody,
    ResponseBody,
    ErrorDescription,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLog {
    pub id: String,
    pub project_id: i64,
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
    pub status_code: Option<i32>,
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

pub struct ApiLogRepository;

impl ApiLogRepository {
    /// Create a new API log entry
    pub async fn create(
        pool: &SqlitePool,
        request: CreateApiLogRequest,
    ) -> Result<ApiLog, sqlx::Error> {
        let id = generate_request_id();

        let sql = {
            Query::insert()
                .into_table(ApiLogs::Table)
                .columns([
                    ApiLogs::Id,
                    ApiLogs::ProjectId,
                    ApiLogs::Method,
                    ApiLogs::Path,
                    ApiLogs::StatusCode,
                    ApiLogs::RequestBody,
                    ApiLogs::ResponseBody,
                    ApiLogs::ErrorDescription,
                ])
                .values_panic([
                    id.into(),
                    request.project_id.into(),
                    request.method.clone().into(),
                    request.path.clone().into(),
                    request.status_code.into(),
                    request.request_body.clone().into(),
                    request.response_body.clone().into(),
                    request.error_desc.clone().into(),
                ])
                .returning(Query::returning().columns([ApiLogs::Id, ApiLogs::CreatedAt]))
                .to_string(SqliteQueryBuilder)
        };

        let row = pool.fetch_one(sql.as_str()).await?;
        let id: String = row.get(0);
        let created_at: i64 = row.get(1);
        let created_at = DateTime::from_timestamp(created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

        Ok(ApiLog {
            id,
            project_id: request.project_id,
            method: request.method,
            path: request.path,
            status_code: request.status_code,
            request_body: request.request_body,
            response_body: request.response_body,
            created_at,
            error_desc: request.error_desc,
        })
    }

    /// Get API log by ID
    pub async fn get_by_id(pool: &SqlitePool, id: String) -> Result<Option<ApiLog>, sqlx::Error> {
        let sql = {
            Query::select()
                .columns([
                    ApiLogs::Id,
                    ApiLogs::ProjectId,
                    ApiLogs::Method,
                    ApiLogs::Path,
                    ApiLogs::StatusCode,
                    ApiLogs::RequestBody,
                    ApiLogs::ResponseBody,
                    ApiLogs::CreatedAt,
                    ApiLogs::ErrorDescription,
                ])
                .from(ApiLogs::Table)
                .and_where(Expr::col(ApiLogs::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let row = pool.fetch_optional(sql.as_str()).await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_api_log(row)?)),
            None => Ok(None),
        }
    }

    /// Update API log
    pub async fn update(
        pool: &SqlitePool,
        id: String,
        request: UpdateApiLogRequest,
    ) -> Result<Option<ApiLog>, sqlx::Error> {
        let sql = {
            let mut query = Query::update();
            query.table(ApiLogs::Table);

            if let Some(status_code) = request.status_code {
                query.value(ApiLogs::StatusCode, status_code);
            }
            if let Some(request_body) = &request.request_body {
                query.value(ApiLogs::RequestBody, request_body.clone());
            }
            if let Some(response_body) = &request.response_body {
                query.value(ApiLogs::ResponseBody, response_body.clone());
            }
            if let Some(err_desc) = &request.error_desc {
                query.value(ApiLogs::ErrorDescription, err_desc.clone());
            }

            query.and_where(Expr::col(ApiLogs::Id).eq(id.clone()));

            query.to_string(SqliteQueryBuilder)
        };

        let result = pool.execute(sql.as_str()).await?;

        if result.rows_affected() > 0 {
            Self::get_by_id(pool, id).await
        } else {
            Ok(None)
        }
    }

    /// Delete API log
    pub async fn delete(pool: &SqlitePool, id: String) -> Result<bool, sqlx::Error> {
        let sql = {
            Query::delete()
                .from_table(ApiLogs::Table)
                .and_where(Expr::col(ApiLogs::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let result = pool.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    /// List API logs with filtering
    pub async fn list(pool: &SqlitePool, filter: ApiLogFilter) -> Result<Vec<ApiLog>, sqlx::Error> {
        let sql = {
            let mut query = Query::select();
            query
                .columns([
                    ApiLogs::Id,
                    ApiLogs::ProjectId,
                    ApiLogs::Method,
                    ApiLogs::Path,
                    ApiLogs::StatusCode,
                    ApiLogs::RequestBody,
                    ApiLogs::ResponseBody,
                    ApiLogs::CreatedAt,
                    ApiLogs::ErrorDescription,
                ])
                .from(ApiLogs::Table);

            // Apply filters
            if let Some(project_id) = &filter.project_id {
                query.and_where(Expr::col(ApiLogs::ProjectId).eq(*project_id));
            }
            if let Some(method) = &filter.method {
                query.and_where(Expr::col(ApiLogs::Method).eq(method.clone()));
            }
            if let Some(path) = &filter.path {
                query.and_where(Expr::col(ApiLogs::Path).like(format!("%{}%", path)));
            }
            if let Some(status_code) = filter.status_code {
                query.and_where(Expr::col(ApiLogs::StatusCode).eq(status_code));
            }

            // Order by created_at descending (most recent first)
            query.order_by(ApiLogs::CreatedAt, sea_query::Order::Desc);

            // Apply pagination
            if let Some(limit) = filter.limit {
                query.limit(limit as u64);
            }
            if let Some(offset) = filter.offset {
                query.offset(offset as u64);
            }

            query.to_string(SqliteQueryBuilder)
        };

        let rows = pool.fetch_all(sql.as_str()).await?;

        let mut api_logs = Vec::new();
        for row in rows {
            api_logs.push(Self::row_to_api_log(row)?);
        }

        Ok(api_logs)
    }

    /// Count API logs with filtering
    pub async fn count(pool: &SqlitePool, filter: ApiLogFilter) -> Result<i64, sqlx::Error> {
        let sql = {
            let mut query = Query::select();
            query
                .expr(Expr::col(ApiLogs::Id).count())
                .from(ApiLogs::Table);

            // Apply same filters as list
            if let Some(project_id) = &filter.project_id {
                query.and_where(Expr::col(ApiLogs::ProjectId).eq(*project_id));
            }
            if let Some(method) = &filter.method {
                query.and_where(Expr::col(ApiLogs::Method).eq(method.clone()));
            }
            if let Some(path) = &filter.path {
                query.and_where(Expr::col(ApiLogs::Path).like(format!("%{}%", path)));
            }
            if let Some(status_code) = filter.status_code {
                query.and_where(Expr::col(ApiLogs::StatusCode).eq(status_code));
            }

            query.to_string(SqliteQueryBuilder)
        };

        let row = pool.fetch_one(sql.as_str()).await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Get logs by project and method
    pub async fn get_by_project_and_method(
        pool: &SqlitePool,
        project_id: i64,
        method: &str,
        limit: Option<u32>,
    ) -> Result<Vec<ApiLog>, sqlx::Error> {
        let filter = ApiLogFilter {
            project_id: Some(project_id),
            method: Some(method.to_string()),
            path: None,
            status_code: None,
            limit: limit.or(Some(50)),
            offset: Some(0),
        };

        Self::list(pool, filter).await
    }

    /// Get error logs (status codes >= 400)
    pub async fn get_error_logs(
        pool: &SqlitePool,
        project_id: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<ApiLog>, sqlx::Error> {
        let sql = {
            let mut query = Query::select();
            query
                .columns([
                    ApiLogs::Id,
                    ApiLogs::ProjectId,
                    ApiLogs::Method,
                    ApiLogs::Path,
                    ApiLogs::StatusCode,
                    ApiLogs::RequestBody,
                    ApiLogs::ResponseBody,
                    ApiLogs::CreatedAt,
                    ApiLogs::ErrorDescription,
                ])
                .from(ApiLogs::Table);

            if let Some(project_id) = &project_id {
                query.and_where(Expr::col(ApiLogs::ProjectId).eq(*project_id));
            }

            query.and_where(Expr::col(ApiLogs::StatusCode).gte(400));
            query.order_by(ApiLogs::CreatedAt, sea_query::Order::Desc);

            if let Some(limit) = limit {
                query.limit(limit as u64);
            }

            query.to_string(SqliteQueryBuilder)
        };

        let rows = pool.fetch_all(sql.as_str()).await?;

        let mut api_logs = Vec::new();
        for row in rows {
            api_logs.push(Self::row_to_api_log(row)?);
        }

        Ok(api_logs)
    }

    /// Delete old API logs (older than specified days)
    pub async fn cleanup_old_logs(
        pool: &SqlitePool,
        days_to_keep: u32,
    ) -> Result<u64, sqlx::Error> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep as i64);

        let sql = {
            Query::delete()
                .from_table(ApiLogs::Table)
                .and_where(Expr::col(ApiLogs::CreatedAt).lt(cutoff_date.timestamp()))
                .to_string(SqliteQueryBuilder)
        };

        let result = pool.execute(sql.as_str()).await?;
        Ok(result.rows_affected())
    }

    /// Helper function to convert SQLite row to ApiLog struct
    fn row_to_api_log(row: sqlx::sqlite::SqliteRow) -> Result<ApiLog, sqlx::Error> {
        let created_at_str: i64 = row.try_get("created_at")?;
        let created_at =
            DateTime::from_timestamp(created_at_str, 0).unwrap_or(DateTime::UNIX_EPOCH);

        Ok(ApiLog {
            id: row.try_get("id")?,
            project_id: row.try_get("project_id")?,
            method: row.try_get("method")?,
            path: row.try_get("path")?,
            status_code: row.try_get("status_code")?,
            request_body: row.try_get("request_body")?,
            response_body: row.try_get("response_body")?,
            created_at,
            error_desc: row.try_get("error_description")?,
        })
    }

    pub async fn init_table(db: &SqlitePool) -> anyhow::Result<()> {
        let api_sql = {
            Table::create()
                .table(ApiLogs::Table)
                .if_not_exists()
                .col(ColumnDef::new(ApiLogs::Id).text().not_null().primary_key())
                .col(ColumnDef::new(ApiLogs::ProjectId).integer().not_null())
                .col(ColumnDef::new(ApiLogs::Method).text().not_null())
                .col(ColumnDef::new(ApiLogs::Path).text().not_null())
                .col(ColumnDef::new(ApiLogs::StatusCode).integer().not_null())
                .col(ColumnDef::new(ApiLogs::RequestBody).text().null())
                .col(ColumnDef::new(ApiLogs::ResponseBody).text().null())
                .col(ColumnDef::new(ApiLogs::ErrorDescription).text().null())
                .col(
                    ColumnDef::new(ApiLogs::CreatedAt)
                        .integer()
                        .default(Expr::cust("(strftime('%s', 'now'))")),
                )
                .to_string(SqliteQueryBuilder)
        };
        db.execute(api_sql.as_str()).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn create_api_log(
    state: State<'_, Database>,
    request: CreateApiLogRequest,
) -> Result<ApiLog, String> {
    ApiLogRepository::create(&state.pool, request)
        .await
        .map_err(|err| format!("Failed to create API log: {}", err))
}

#[tauri::command]
pub async fn get_api_log(
    state: State<'_, Database>,
    log_id: String,
) -> Result<Option<ApiLog>, String> {
    ApiLogRepository::get_by_id(&state.pool, log_id)
        .await
        .map_err(|err| format!("Failed to get API log: {}", err))
}

#[tauri::command]
pub async fn update_api_log(
    state: State<'_, Database>,
    log_id: String,
    request: UpdateApiLogRequest,
) -> Result<Option<ApiLog>, String> {
    ApiLogRepository::update(&state.pool, log_id, request)
        .await
        .map_err(|err| format!("Failed to update API log: {}", err))
}

#[tauri::command]
pub async fn delete_api_log(state: State<'_, Database>, log_id: String) -> Result<bool, String> {
    ApiLogRepository::delete(&state.pool, log_id)
        .await
        .map_err(|err| format!("Failed to delete API log: {}", err))
}

#[tauri::command]
pub async fn list_api_logs(
    state: State<'_, Database>,
    project_id: Option<i64>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<i32>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ApiLog>, String> {
    let filter = ApiLogFilter {
        project_id,
        method,
        path,
        status_code,
        limit: limit.or(Some(50)),
        offset: offset.or(Some(0)),
    };

    ApiLogRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to list API logs: {}", err))
}

#[tauri::command]
pub async fn count_api_logs(
    state: State<'_, Database>,
    project_id: Option<i64>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<i32>,
) -> Result<i64, String> {
    let filter = ApiLogFilter {
        project_id,
        method,
        path,
        status_code,
        limit: None,
        offset: None,
    };

    ApiLogRepository::count(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to count API logs: {}", err))
}

#[tauri::command]
pub async fn get_project_api_logs(
    state: State<'_, Database>,
    project_id: i64,
    method: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ApiLog>, String> {
    let filter = ApiLogFilter {
        project_id: Some(project_id),
        method,
        path: None,
        status_code: None,
        limit: limit.or(Some(50)),
        offset: offset.or(Some(0)),
    };

    ApiLogRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to get project API logs: {}", err))
}

#[tauri::command]
pub async fn get_api_logs_by_method(
    state: State<'_, Database>,
    project_id: i64,
    method: String,
    limit: Option<u32>,
) -> Result<Vec<ApiLog>, String> {
    ApiLogRepository::get_by_project_and_method(&state.pool, project_id, &method, limit)
        .await
        .map_err(|err| format!("Failed to get API logs by method: {}", err))
}

#[tauri::command]
pub async fn get_error_api_logs(
    state: State<'_, Database>,
    project_id: Option<i64>,
    limit: Option<u32>,
) -> Result<Vec<ApiLog>, String> {
    ApiLogRepository::get_error_logs(&state.pool, project_id, limit)
        .await
        .map_err(|err| format!("Failed to get error API logs: {}", err))
}

#[tauri::command]
pub async fn get_recent_api_logs(
    state: State<'_, Database>,
    project_id: Option<i64>,
    limit: Option<u32>,
) -> Result<Vec<ApiLog>, String> {
    let filter = ApiLogFilter {
        project_id,
        method: None,
        path: None,
        status_code: None,
        limit: limit.or(Some(20)),
        offset: Some(0),
    };

    ApiLogRepository::list(&state.pool, filter)
        .await
        .map_err(|err| format!("Failed to get recent API logs: {}", err))
}

#[tauri::command]
pub async fn cleanup_old_api_logs(
    state: State<'_, Database>,
    days_to_keep: u32,
) -> Result<u64, String> {
    ApiLogRepository::cleanup_old_logs(&state.pool, days_to_keep)
        .await
        .map_err(|err| format!("Failed to cleanup old API logs: {}", err))
}

#[tauri::command]
pub async fn get_api_log_stats(
    state: State<'_, Database>,
    project_id: Option<i64>,
) -> Result<ApiLogStats, String> {
    let filter = ApiLogFilter {
        project_id,
        method: None,
        path: None,
        status_code: None,
        limit: None,
        offset: None,
    };

    let total_count = ApiLogRepository::count(&state.pool, filter.clone())
        .await
        .map_err(|err| format!("Failed to get total count: {}", err))?;

    // Count successful requests (2xx status codes)
    let success_query = {
        let mut success_query = Query::select();
        success_query
            .expr(Expr::col(crate::api_logs::ApiLogs::Id).count())
            .from(crate::api_logs::ApiLogs::Table);

        if let Some(project_id) = &project_id {
            success_query.and_where(Expr::col(crate::api_logs::ApiLogs::ProjectId).eq(*project_id));
        }
        success_query.and_where(Expr::col(crate::api_logs::ApiLogs::StatusCode).between(200, 299));

        success_query.to_string(sea_query::SqliteQueryBuilder)
    };

    let success_count = state
        .pool
        .fetch_one(success_query.as_str())
        .await
        .map_err(|err| format!("Failed to get success count: {}", err))?
        .get::<i64, _>(0);

    // Count client errors (4xx status codes)
    let client_error_query = {
        let mut client_error_query = Query::select();
        client_error_query
            .expr(Expr::col(crate::api_logs::ApiLogs::Id).count())
            .from(crate::api_logs::ApiLogs::Table);

        if let Some(project_id) = &project_id {
            client_error_query
                .and_where(Expr::col(crate::api_logs::ApiLogs::ProjectId).eq(*project_id));
        }
        client_error_query
            .and_where(Expr::col(crate::api_logs::ApiLogs::StatusCode).between(400, 499));

        client_error_query.to_string(sea_query::SqliteQueryBuilder)
    };
    let client_error_count = state
        .pool
        .fetch_one(client_error_query.as_str())
        .await
        .map_err(|err| format!("Failed to get client error count: {}", err))?
        .get::<i64, _>(0);

    // Count server errors (5xx status codes)
    let server_error_query = {
        let mut server_error_query = Query::select();
        server_error_query
            .expr(Expr::col(crate::api_logs::ApiLogs::Id).count())
            .from(crate::api_logs::ApiLogs::Table);

        if let Some(project_id) = &project_id {
            server_error_query
                .and_where(Expr::col(crate::api_logs::ApiLogs::ProjectId).eq(*project_id));
        }
        server_error_query
            .and_where(Expr::col(crate::api_logs::ApiLogs::StatusCode).between(500, 599));

        server_error_query.to_string(sea_query::SqliteQueryBuilder)
    };

    let server_error_count = state
        .pool
        .fetch_one(server_error_query.as_str())
        .await
        .map_err(|err| format!("Failed to get server error count: {}", err))?
        .get::<i64, _>(0);

    Ok(ApiLogStats {
        total_count,
        success_count,
        client_error_count,
        server_error_count,
    })
}

// Helper struct for API log statistics
#[derive(serde::Serialize)]
pub struct ApiLogStats {
    pub total_count: i64,
    pub success_count: i64,
    pub client_error_count: i64,
    pub server_error_count: i64,
}
