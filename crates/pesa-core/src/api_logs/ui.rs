use super::UpdateApiLogRequest;
use anyhow::Result;

use crate::AppContext;
use crate::api_logs::ApiLog;

pub use super::ApiLogFilter;

pub async fn get_api_log(ctx: &AppContext, log_id: String) -> Result<Option<ApiLog>> {
    ApiLog::read_by_id(&ctx.db, log_id).await
}

pub async fn update_api_log(
    ctx: &AppContext,
    log_id: String,
    request: UpdateApiLogRequest,
) -> Result<Option<ApiLog>> {
    ApiLog::update(&ctx.db, log_id, request).await
}

pub async fn delete_api_log(ctx: &AppContext, log_id: String) -> Result<bool> {
    ApiLog::delete(&ctx.db, log_id).await
}

pub async fn list_api_logs(ctx: &AppContext, filter: ApiLogFilter) -> Result<Vec<ApiLog>> {
    ApiLog::find(&ctx.db, filter).await
}

pub async fn count_api_logs(
    ctx: &AppContext,
    project_id: Option<i64>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<i32>,
) -> Result<usize> {
    let filter = ApiLogFilter {
        project_id: project_id.map(|id| id as u32),
        method,
        path,
        status_code: status_code.map(|s| s as u16),
        ..Default::default()
    };
    ApiLog::count(&ctx.db, filter).await
}

pub async fn get_project_api_logs(
    ctx: &AppContext,
    project_id: u32,
    filter: ApiLogFilter,
) -> Result<Vec<ApiLog>> {
    let filter = ApiLogFilter {
        project_id: Some(project_id),
        ..filter
    };
    ApiLog::find(&ctx.db, filter).await
}

pub async fn get_api_logs_by_method(
    ctx: &AppContext,
    project_id: u32,
    method: String,
    limit: Option<u64>,
) -> Result<Vec<ApiLog>> {
    let filter = ApiLogFilter {
        project_id: Some(project_id),
        method: Some(method),
        limit,
        ..Default::default()
    };
    ApiLog::find(&ctx.db, filter).await
}
