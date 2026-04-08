use super::{FullRequestView, Request, RequestFilter};
use crate::{AppContext, request_lifecycle::stats::RequestStatistics};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

// This file contains the public-facing API for the request lifecycle module,
// designed to be called from the tauri/axum handlers.

pub async fn get_full_request(
    ctx: &AppContext,
    request_id: &str,
) -> Result<Option<FullRequestView>> {
    FullRequestView::get_full_request(&ctx.db, request_id).await
}

pub async fn find_request(
    ctx: &AppContext,
    id_type: &str,
    external_id: &str,
) -> Result<Option<FullRequestView>> {
    FullRequestView::find_request(&ctx.db, id_type, external_id)
        .await
        .context("Failed to find request")
}

pub async fn get_project_requests(
    ctx: &AppContext,
    project_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    Request::get_project_requests(&ctx.db, project_id, filter)
        .await
        .context("Failed to get project requests")
}

pub async fn get_business_requests(
    ctx: &AppContext,
    business_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    Request::get_business_requests(&ctx.db, business_id, filter)
        .await
        .context("Failed to get business requests")
}

pub async fn get_by_external_id(
    ctx: &AppContext,
    id_type: &str,
    external_id: &str,
) -> Result<Option<Request>> {
    Request::find_by_external_id(&ctx.db, id_type, external_id)
        .await
        .context("Failed to get requests by external id")
}

pub async fn list_requests(ctx: &AppContext, filter: RequestFilter) -> Result<Vec<Request>> {
    Request::list_requests(&ctx.db, filter)
        .await
        .context("Failed to list requests")
}

pub async fn count_requests(ctx: &AppContext, filter: RequestFilter) -> Result<u64> {
    Request::count_requests(&ctx.db, filter)
        .await
        .context("Failed to count requests")
}

pub async fn get_request_statistics(
    ctx: &AppContext,
    project_id: Option<u32>,
    business_id: Option<u32>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> Result<RequestStatistics> {
    RequestStatistics::get_request_statistics(&ctx.db, project_id, business_id, date_from, date_to)
        .await
        .context("Failed to get request statistics")
}
