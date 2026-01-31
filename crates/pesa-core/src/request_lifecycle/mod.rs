use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod db;
pub mod manager;
pub mod paths;
pub mod request_ids;
pub mod ui;

pub use manager::RequestLifecycleManager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum RequestStatus {
    Completed,
    Failed,
    Pending,
    Unknown(String),
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestStatus::Completed => write!(f, "completed"),
            RequestStatus::Failed => write!(f, "failed"),
            RequestStatus::Pending => write!(f, "pending"),
            RequestStatus::Unknown(s) => write!(f, "unknown:{}", s),
        }
    }
}

impl FromStr for RequestStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "completed" => RequestStatus::Completed,
            "failed" => RequestStatus::Failed,
            "pending" => RequestStatus::Pending,
            s if s.starts_with("unknown:") => {
                RequestStatus::Unknown(s.strip_prefix("unknown:").unwrap_or(s).to_string())
            }
            _ => RequestStatus::Unknown(s.to_string()),
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
pub enum RequestType {
    StkPush,
    C2bRegisterUrl,
    B2cPayment,
    BalanceQuery,
    C2bLipa,
    Oauth,
    Other(String),
}

impl std::fmt::Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::StkPush => write!(f, "stk_push"),
            RequestType::C2bRegisterUrl => write!(f, "c2b_register_url"),
            RequestType::B2cPayment => write!(f, "b2c_payment"),
            RequestType::BalanceQuery => write!(f, "balance_query"),
            RequestType::C2bLipa => write!(f, "c2b_lipa"),
            RequestType::Oauth => write!(f, "oauth"),
            RequestType::Other(path) => write!(f, "other:{}", path),
        }
    }
}

impl FromStr for RequestType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stk_push" => RequestType::StkPush,
            "c2b_register_url" => RequestType::C2bRegisterUrl,
            "b2c_payment" => RequestType::B2cPayment,
            "balance_query" => RequestType::BalanceQuery,
            "c2b_lipa" => RequestType::C2bLipa,
            "oauth" => RequestType::Oauth,
            s if s.starts_with("other:") => {
                RequestType::Other(s.strip_prefix("other:").unwrap_or(s).to_string())
            }
            _ => RequestType::Other(s.to_string()),
        })
    }
}

impl RequestType {
    pub fn from_path(path: &str) -> Self {
        match path {
            paths::OAUTH => RequestType::Oauth,
            paths::STK_PUSH => RequestType::StkPush,
            paths::C2B_REGISTER_URL => RequestType::C2bRegisterUrl,
            paths::B2C_PAYMENT => RequestType::B2cPayment,
            paths::BALANCE_QUERY => RequestType::BalanceQuery,
            _ => RequestType::Other(path.to_string()),
        }
    }

    pub fn is_trackable(&self) -> bool {
        !matches!(self, RequestType::Other(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub source_type: String,
    pub source_api_key_id: Option<u32>,
    pub source_operator_id: Option<u32>,
    pub source_component: Option<String>,
    pub request_type: RequestType,
    pub request_status: RequestStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub project_id: Option<u32>,
    pub business_id: Option<u32>,
    pub user_id: Option<u32>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
}

impl From<db::Model> for Request {
    fn from(model: db::Model) -> Self {
        Self {
            id: model.id,
            source_type: model.source_type,
            source_api_key_id: model.source_api_key_id,
            source_operator_id: model.source_operator_id,
            source_component: model.source_component,
            request_type: model.request_type.parse().expect("Failed to parse request_type from database. This should not happen as `from_str` always returns `Ok`."),
            request_status: model
                .request_status
                .parse()
                .expect("Failed to parse request_status, this should not happen"),
            created_at: model.created_at,
            started_at: model.started_at,
            completed_at: model.completed_at,
            project_id: model.project_id,
            business_id: model.business_id,
            user_id: model.user_id,
            request_body: model.request_body,
            response_body: model.response_body,
            error_message: model.error_message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullRequestView {
    pub request: Request,
    pub external_ids: Vec<request_ids::RequestId>,
    pub api_log: Option<crate::api_logs::ApiLog>,
    pub transactions: Vec<crate::transactions::Transaction>,
    pub callbacks: Vec<crate::callbacks::CallbackLog>,
    pub jobs: Vec<crate::transaction_jobs::TransactionJob>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct RequestFilter {
    pub project_id: Option<u32>,
    pub business_id: Option<u32>,
    pub request_type: Option<String>,
    pub status: Option<String>,
    pub source_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FindRequestQuery {
    pub id_type: String,
    pub external_id: String,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct RequestStatistics {
    pub total: u64,
    pub completed: u64,
    pub failed: u64,
    pub pending: u64,
    pub api_requests: u64,
    pub ui_requests: u64,
    pub system_requests: u64,
}

pub async fn get_full_request(
    ctx: &crate::AppContext,
    request_id: &str,
) -> Result<Option<FullRequestView>> {
    let request_model = ctx
        .request_lifecycle_manager
        .get_request(request_id)
        .await?;

    if let Some(request_model) = request_model {
        let external_ids =
            request_ids::RequestId::find_by_request_id(&ctx.db, request_id.to_string()).await?;

        let api_log_model = crate::api_logs::db::Entity::find()
            .filter(crate::api_logs::db::Column::RequestId.eq(request_id))
            .one(&ctx.db)
            .await?;

        let transaction_models = crate::transactions::db::Entity::find()
            .filter(crate::transactions::db::Column::RequestId.eq(request_id))
            .all(&ctx.db)
            .await?;

        let callback_models = crate::callbacks::db::Entity::find()
            .filter(crate::callbacks::db::Column::RequestId.eq(request_id))
            .all(&ctx.db)
            .await?;

        let job_models = crate::transaction_jobs::db::Entity::find()
            .filter(crate::transaction_jobs::db::Column::RequestId.eq(request_id))
            .all(&ctx.db)
            .await?;

        let mut jobs = Vec::new();
        for model in job_models {
            jobs.push(model.try_into()?);
        }

        Ok(Some(FullRequestView {
            request: request_model.into(),
            external_ids,
            api_log: api_log_model.map(Into::into),
            transactions: transaction_models.into_iter().map(Into::into).collect(),
            callbacks: callback_models.into_iter().map(Into::into).collect(),
            jobs,
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_request(
    ctx: &crate::AppContext,
    query: FindRequestQuery,
) -> Result<Option<FullRequestView>> {
    let request = ctx
        .request_lifecycle_manager
        .find_by_external_id(&query.id_type, &query.external_id)
        .await?;

    if let Some(request) = request {
        get_full_request(ctx, &request.id).await
    } else {
        Ok(None)
    }
}

pub async fn get_project_requests(
    ctx: &crate::AppContext,
    project_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    let models = ctx
        .request_lifecycle_manager
        .get_project_requests(project_id, filter)
        .await?;
    Ok(models.into_iter().map(Into::into).collect())
}

pub async fn get_business_requests(
    ctx: &crate::AppContext,
    business_id: u32,
    filter: RequestFilter,
) -> Result<Vec<Request>> {
    let models = ctx
        .request_lifecycle_manager
        .get_business_requests(business_id, filter)
        .await?;
    Ok(models.into_iter().map(Into::into).collect())
}

pub async fn get_by_external_id(
    ctx: &crate::AppContext,
    id_type: &str,
    external_id: &str,
) -> Result<Option<Request>> {
    let model = ctx
        .request_lifecycle_manager
        .find_by_external_id(id_type, external_id)
        .await?;
    Ok(model.map(Into::into))
}

pub async fn list_requests(ctx: &crate::AppContext, filter: RequestFilter) -> Result<Vec<Request>> {
    let mut query = db::Entity::find().order_by_desc(db::Column::CreatedAt);

    if let Some(project_id) = filter.project_id {
        query = query.filter(db::Column::ProjectId.eq(project_id));
    }
    if let Some(business_id) = filter.business_id {
        query = query.filter(db::Column::BusinessId.eq(business_id));
    }
    if let Some(request_type) = filter.request_type {
        query = query.filter(db::Column::RequestType.eq(request_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(db::Column::RequestStatus.eq(status));
    }
    if let Some(source_type) = filter.source_type {
        query = query.filter(db::Column::SourceType.eq(source_type));
    }
    if let Some(date_from) = filter.date_from {
        query = query.filter(db::Column::CreatedAt.gte(date_from));
    }
    if let Some(date_to) = filter.date_to {
        query = query.filter(db::Column::CreatedAt.lte(date_to));
    }
    if let Some(limit) = filter.limit {
        query = query.limit(Some(limit));
    }
    if let Some(offset) = filter.offset {
        query = query.offset(Some(offset));
    }

    let requests = query
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(requests)
}

pub async fn count_requests(ctx: &crate::AppContext, filter: RequestFilter) -> Result<u64> {
    let mut query = db::Entity::find();

    if let Some(project_id) = filter.project_id {
        query = query.filter(db::Column::ProjectId.eq(project_id));
    }
    if let Some(business_id) = filter.business_id {
        query = query.filter(db::Column::BusinessId.eq(business_id));
    }
    if let Some(request_type) = filter.request_type {
        query = query.filter(db::Column::RequestType.eq(request_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(db::Column::RequestStatus.eq(status));
    }
    if let Some(source_type) = filter.source_type {
        query = query.filter(db::Column::SourceType.eq(source_type));
    }
    if let Some(date_from) = filter.date_from {
        query = query.filter(db::Column::CreatedAt.gte(date_from));
    }
    if let Some(date_to) = filter.date_to {
        query = query.filter(db::Column::CreatedAt.lte(date_to));
    }

    query.count(&ctx.db).await.map_err(Into::into)
}

pub async fn get_request_statistics(
    ctx: &crate::AppContext,
    project_id: Option<u32>,
    business_id: Option<u32>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> Result<RequestStatistics> {
    let mut query = db::Entity::find();

    if let Some(pid) = project_id {
        query = query.filter(db::Column::ProjectId.eq(pid));
    }
    if let Some(bid) = business_id {
        query = query.filter(db::Column::BusinessId.eq(bid));
    }
    if let Some(from) = date_from {
        query = query.filter(db::Column::CreatedAt.gte(from));
    }
    if let Some(to) = date_to {
        query = query.filter(db::Column::CreatedAt.lte(to));
    }

    let requests: Vec<Request> = query
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    let mut stats = RequestStatistics::default();
    for request in requests {
        stats.total += 1;
        match request.request_status {
            RequestStatus::Completed => stats.completed += 1,
            RequestStatus::Failed => stats.failed += 1,
            RequestStatus::Pending => stats.pending += 1,
            RequestStatus::Unknown(_) => {}
        }
        match request.source_type.as_str() {
            "api" => stats.api_requests += 1,
            "ui" => stats.ui_requests += 1,
            "system" => stats.system_requests += 1,
            _ => {}
        }
    }

    Ok(stats)
}
