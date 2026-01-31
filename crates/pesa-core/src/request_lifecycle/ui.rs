use super::{FindRequestQuery, FullRequestView, Request, RequestFilter, RequestStatistics};
use crate::AppContext;
use anyhow::Result;
use chrono::{DateTime, Utc};

// This file contains the public-facing API for the request lifecycle module,
// designed to be called from the tauri/axum handlers.

pub async fn get_full_request(
    ctx: &AppContext,
    request_id: &str,
) -> Result<Option<FullRequestView>> {
    super::get_full_request(ctx, request_id).await
}

pub async fn find_request(
    ctx: &AppContext,
    query: FindRequestQuery,
) -> Result<Option<FullRequestView>> {
    super::find_request(ctx, query).await
}

pub async fn get_project_requests(
    ctx: &AppContext,
    project_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    super::get_project_requests(ctx, project_id, filter).await
}

pub async fn get_business_requests(
    ctx: &AppContext,
    business_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    super::get_business_requests(ctx, business_id, filter).await
}

pub async fn get_by_external_id(
    ctx: &AppContext,
    id_type: &str,
    external_id: &str,
) -> Result<Option<Request>> {
    super::get_by_external_id(ctx, id_type, external_id).await
}

pub async fn list_requests(ctx: &AppContext, filter: RequestFilter) -> Result<Vec<Request>> {
    super::list_requests(ctx, filter).await
}

pub async fn count_requests(ctx: &AppContext, filter: RequestFilter) -> Result<u64> {
    super::count_requests(ctx, filter).await
}

pub async fn get_request_statistics(
    ctx: &AppContext,
    project_id: Option<u32>,
    business_id: Option<u32>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> Result<RequestStatistics> {
    super::get_request_statistics(ctx, project_id, business_id, date_from, date_to).await
}
