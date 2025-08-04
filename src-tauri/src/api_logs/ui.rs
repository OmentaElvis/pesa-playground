use super::db::ActiveModel;
use super::{db::Entity as ApiLogs, UpdateApiLogRequest};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use tauri::State;

use crate::{api_logs::ApiLog, db::Database};

#[tauri::command]
pub async fn get_api_log(
    state: State<'_, Database>,
    log_id: String,
) -> Result<Option<ApiLog>, String> {
    let log = ApiLogs::find_by_id(log_id)
        .one(&state.conn)
        .await
        .map_err(|err| format!("Failed to get API log: {}", err))?;

    Ok(log.map(|log| ApiLog {
        id: log.id,
        method: log.method,
        path: log.path,
        project_id: log.project_id,
        status_code: log.status_code,
        request_body: log.request_body,
        response_body: log.response_body,
        error_desc: log.error_desc,
        created_at: log.created_at,
    }))
}

#[tauri::command]
pub async fn update_api_log(
    state: State<'_, Database>,
    log_id: String,
    request: UpdateApiLogRequest,
) -> Result<Option<ApiLog>, String> {
    let log = ApiLogs::find_by_id(log_id)
        .one(&state.conn)
        .await
        .map_err(|err| format!("Failed to get API log: {}", err))?;

    if log.is_none() {
        return Ok(None);
    }

    let mut log: ActiveModel = log.unwrap().into();

    if let Some(status_code) = request.status_code {
        log.status_code = Set(status_code);
    }

    if let Some(body) = &request.request_body {
        log.request_body = Set(Some(body.to_string()));
    }

    if let Some(body) = request.response_body {
        log.response_body = Set(Some(body.to_string()));
    }

    if let Some(desc) = request.error_desc {
        log.error_desc = Set(Some(desc.to_string()));
    }

    let model = log
        .update(&state.conn)
        .await
        .map_err(|err| format!("Failed to update logs: {}", err))?;

    Ok(Some(ApiLog {
        id: model.id,
        method: model.method,
        path: model.path,
        project_id: model.project_id,
        status_code: model.status_code,
        request_body: model.request_body,
        response_body: model.response_body,
        error_desc: model.error_desc,
        created_at: model.created_at,
    }))
}

#[tauri::command]
pub async fn delete_api_log(state: State<'_, Database>, log_id: String) -> Result<bool, String> {
    let res = ApiLogs::delete_by_id(log_id)
        .exec(&state.conn)
        .await
        .map_err(|err| format!("Failed to delete API log: {}", err))?;

    Ok(res.rows_affected > 0)
}

// #[tauri::command]
pub async fn list_api_logs(
    state: State<'_, Database>,
    project_id: Option<u32>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<u16>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<ApiLog>, String> {
    let mut q = ApiLogs::find();

    if let Some(project) = project_id {
        q = q.filter(super::db::Column::ProjectId.eq(project));
    }

    if let Some(method) = method {
        q = q.filter(super::db::Column::Path.eq(method));
    }

    if let Some(path) = path {
        q = q.filter(super::db::Column::Path.eq(path));
    }

    if let Some(status) = status_code {
        q = q.filter(super::db::Column::StatusCode.eq(status));
    }

    if let Some(limit) = limit {
        q = q.limit(Some(limit));
    }

    if let Some(offset) = offset {
        q = q.offset(Some(offset));
    }

    let logs = q
        .all(&state.conn)
        .await
        .map_err(|err| format!("Failed to get logs: {}", err))?;

    Ok(logs
        .into_iter()
        .map(|log| ApiLog {
            id: log.id,
            method: log.method,
            path: log.path,
            project_id: log.project_id,
            status_code: log.status_code,
            request_body: log.request_body,
            response_body: log.response_body,
            error_desc: log.error_desc,
            created_at: log.created_at,
        })
        .collect())
}

#[tauri::command]
pub async fn count_api_logs(
    state: State<'_, Database>,
    project_id: Option<i64>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<i32>,
) -> Result<usize, String> {
    let mut q = ApiLogs::find();

    if let Some(project) = project_id {
        q = q.filter(super::db::Column::ProjectId.eq(project));
    }

    if let Some(method) = method {
        q = q.filter(super::db::Column::Path.eq(method));
    }

    if let Some(path) = path {
        q = q.filter(super::db::Column::Path.eq(path));
    }

    if let Some(status) = status_code {
        q = q.filter(super::db::Column::StatusCode.eq(status));
    }

    Ok(q.all(&state.conn)
        .await
        .map_err(|err| format!("Failed to get logs: {}", err))?
        .len())
}

#[tauri::command]
pub async fn get_project_api_logs(
    state: State<'_, Database>,
    project_id: i64,
    method: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<ApiLog>, String> {
    let mut q = ApiLogs::find().filter(super::db::Column::ProjectId.eq(project_id));

    if let Some(method) = method {
        q = q.filter(super::db::Column::Path.eq(method));
    }

    if let Some(limit) = limit {
        q = q.limit(Some(limit));
    }

    if let Some(offset) = offset {
        q = q.offset(Some(offset));
    }

    let logs = q
        .all(&state.conn)
        .await
        .map_err(|err| format!("Failed to get logs: {}", err))?;

    Ok(logs
        .into_iter()
        .map(|log| ApiLog {
            id: log.id,
            method: log.method,
            path: log.path,
            project_id: log.project_id,
            status_code: log.status_code,
            request_body: log.request_body,
            response_body: log.response_body,
            error_desc: log.error_desc,
            created_at: log.created_at,
        })
        .collect())
}

#[tauri::command]
pub async fn get_api_logs_by_method(
    state: State<'_, Database>,
    project_id: i64,
    method: String,
    limit: Option<u64>,
) -> Result<Vec<ApiLog>, String> {
    let mut q = ApiLogs::find()
        .filter(super::db::Column::ProjectId.eq(project_id))
        .filter(super::db::Column::Path.eq(method));

    if let Some(limit) = limit {
        q = q.limit(Some(limit));
    }

    let logs = q
        .all(&state.conn)
        .await
        .map_err(|err| format!("Failed to get logs: {}", err))?;

    Ok(logs
        .into_iter()
        .map(|log| ApiLog {
            id: log.id,
            method: log.method,
            path: log.path,
            project_id: log.project_id,
            status_code: log.status_code,
            request_body: log.request_body,
            response_body: log.response_body,
            error_desc: log.error_desc,
            created_at: log.created_at,
        })
        .collect())
}
