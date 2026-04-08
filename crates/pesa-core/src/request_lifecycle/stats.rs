use crate::request_lifecycle::{Request, RequestStatus, SourceType};

use super::db;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};

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

impl RequestStatistics {
    pub async fn get_request_statistics<C>(
        conn: &C,
        project_id: Option<u32>,
        business_id: Option<u32>,
        date_from: Option<DateTime<Utc>>,
        date_to: Option<DateTime<Utc>>,
    ) -> Result<RequestStatistics, DbErr>
    where
        C: ConnectionTrait,
    {
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

        let requests: Vec<Request> = query.all(conn).await?.into_iter().map(Into::into).collect();

        let mut stats = RequestStatistics::default();
        for request in requests {
            stats.total += 1;
            match request.request_status {
                RequestStatus::Completed => stats.completed += 1,
                RequestStatus::Failed => stats.failed += 1,
                RequestStatus::Pending => stats.pending += 1,
                RequestStatus::Unknown => {}
            }
            match request.source_type {
                SourceType::Api => stats.api_requests += 1,
                SourceType::Ui => stats.ui_requests += 1,
                SourceType::System => stats.system_requests += 1,
            }
        }

        Ok(stats)
    }
}
