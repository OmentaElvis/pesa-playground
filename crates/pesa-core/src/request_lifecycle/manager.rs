use anyhow::Result;
use chrono::Utc;
use sea_orm::{prelude::*, *};
use sea_query::Expr;

use super::db::*;
use super::request_ids::db as request_ids_db;
use crate::utils::identifiers::Identifiers;

#[derive(Debug, Clone)]
pub enum RequestSource {
    Api { api_key_id: u32 },
    System { component: String },
}

pub struct RequestLifecycleManager {
    db: DatabaseConnection,
}

impl RequestLifecycleManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_api_request(
        &self,
        identifiers: &Identifiers,
        api_key_id: u32,
        request_type: super::RequestType,
        request_body: Option<String>,
    ) -> Result<()> {
        self.create_request(
            identifiers,
            RequestSource::Api { api_key_id },
            request_type,
            None,
            None,
            None,
            request_body,
            None,
            None,
        )
        .await
    }

    pub async fn create_system_request(
        &self,
        identifiers: &Identifiers,
        component: &str,
        request_type: super::RequestType,
        project_id: Option<u32>,
        business_id: Option<u32>,
        user_id: Option<u32>,
    ) -> Result<()> {
        self.create_request(
            identifiers,
            RequestSource::System {
                component: component.to_string(),
            },
            request_type,
            project_id,
            business_id,
            user_id,
            None,
            None,
            None,
        )
        .await
    }

    async fn create_request(
        &self,
        identifiers: &Identifiers,
        source: RequestSource,
        request_type: super::RequestType,
        project_id: Option<u32>,
        business_id: Option<u32>,
        user_id: Option<u32>,
        request_body: Option<String>,
        response_body: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();

        let (source_type, source_api_key_id, source_component) = match source {
            RequestSource::Api { api_key_id } => ("api".to_string(), Some(api_key_id), None),
            RequestSource::System { component } => ("system".to_string(), None, Some(component)),
        };

        let active_model = ActiveModel {
            id: Set(identifiers.request_id.clone()),
            source_type: Set(source_type),
            source_api_key_id: Set(source_api_key_id),
            source_operator_id: Set(None),
            source_component: Set(source_component),
            request_type: Set(request_type.to_string()),
            request_status: Set("pending".to_string()),
            created_at: Set(now),
            started_at: Set(None),
            completed_at: Set(None),
            project_id: Set(project_id),
            business_id: Set(business_id),
            user_id: Set(user_id),
            request_body: Set(request_body),
            response_body: Set(response_body),
            error_message: Set(error_message),
        };

        let _result = Entity::insert(active_model).exec(&self.db).await?;

        Ok(())
    }

    pub async fn start_request(&self, request_id: &str) -> Result<()> {
        let now = Utc::now();

        Entity::update_many()
            .filter(Column::Id.eq(request_id))
            .col_expr(Column::StartedAt, Expr::value(now))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn complete_request(
        &self,
        request_id: &str,
        response_body: Option<String>,
        error_message: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        let status = if error_message.is_some() {
            "failed"
        } else {
            "completed"
        };

        let mut active_model = Entity::find_by_id(request_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Request not found: {}", request_id))?
            .into_active_model();

        active_model.request_status = Set(status.to_string());
        active_model.completed_at = Set(Some(now));

        if let Some(body) = response_body {
            active_model.response_body = Set(Some(body));
        }

        if let Some(error) = error_message {
            active_model.error_message = Set(Some(error));
        }

        let _result = active_model.update(&self.db).await?;

        Ok(())
    }

    pub async fn link_api_log(&self, request_id: &str, api_log_id: &str) -> Result<()> {
        use crate::api_logs::db as api_logs_db;

        api_logs_db::Entity::update_many()
            .filter(api_logs_db::Column::Id.eq(api_log_id))
            .col_expr(api_logs_db::Column::RequestId, Expr::value(request_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn link_transaction(&self, request_id: &str, transaction_id: &str) -> Result<()> {
        use crate::transactions::db as transactions_db;

        transactions_db::Entity::update_many()
            .filter(transactions_db::Column::Id.eq(transaction_id))
            .col_expr(transactions_db::Column::RequestId, Expr::value(request_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn link_callback(&self, request_id: &str, callback_id: u32) -> Result<()> {
        use crate::callbacks::db as callbacks_db;

        callbacks_db::Entity::update_many()
            .filter(callbacks_db::Column::Id.eq(callback_id))
            .col_expr(callbacks_db::Column::RequestId, Expr::value(request_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn link_transaction_job(&self, request_id: &str, job_id: &str) -> Result<()> {
        use crate::transaction_jobs::db as transaction_jobs_db;

        transaction_jobs_db::Entity::update_many()
            .filter(transaction_jobs_db::Column::Id.eq(job_id))
            .col_expr(
                transaction_jobs_db::Column::RequestId,
                Expr::value(request_id),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn add_external_ids(
        &self,
        identifiers: &Identifiers,
        checkout_request_id: Option<String>,
    ) -> Result<()> {
        let external_ids_to_store = vec![
            (
                "originator_conversation_id",
                identifiers.originator_conversation_id.clone(),
            ),
            ("conversation_id", identifiers.conversation_id.clone()),
            ("transaction_id", identifiers.transaction_id.clone()),
        ];

        for (id_type, external_id) in external_ids_to_store {
            let active_model = request_ids_db::ActiveModel {
                request_id: Set(identifiers.request_id.clone()),
                id_type: Set(id_type.to_string()),
                external_id: Set(external_id),
                ..Default::default()
            };

            request_ids_db::Entity::insert(active_model)
                .exec(&self.db)
                .await?;
        }

        if let Some(checkout_id) = checkout_request_id {
            let active_model = request_ids_db::ActiveModel {
                request_id: Set(identifiers.request_id.clone()),
                id_type: Set("checkout_request_id".to_string()),
                external_id: Set(checkout_id),
                ..Default::default()
            };

            request_ids_db::Entity::insert(active_model)
                .exec(&self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn get_request(&self, request_id: &str) -> Result<Option<Model>> {
        let request = Entity::find_by_id(request_id).one(&self.db).await?;

        Ok(request)
    }

    pub async fn find_by_external_id(
        &self,
        id_type: &str,
        external_id: &str,
    ) -> Result<Option<Model>> {
        let request = Entity::find()
            .join(JoinType::InnerJoin, Relation::RequestIds.def())
            .filter(request_ids_db::Column::IdType.eq(id_type))
            .filter(request_ids_db::Column::ExternalId.eq(external_id))
            .one(&self.db)
            .await?;

        Ok(request)
    }

    pub async fn get_project_requests(
        &self,
        project_id: u32,
        filter: crate::request_lifecycle::RequestFilter,
    ) -> Result<Vec<Model>> {
        let mut query = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .order_by_desc(Column::CreatedAt);

        // Apply filters
        if let Some(request_type) = filter.request_type {
            query = query.filter(Column::RequestType.eq(request_type));
        }

        if let Some(status) = filter.status {
            query = query.filter(Column::RequestStatus.eq(status));
        }

        if let Some(source_type) = filter.source_type {
            query = query.filter(Column::SourceType.eq(source_type));
        }

        if let Some(date_from) = filter.date_from {
            query = query.filter(Column::CreatedAt.gte(date_from));
        }

        if let Some(date_to) = filter.date_to {
            query = query.filter(Column::CreatedAt.lte(date_to));
        }

        if let Some(limit) = filter.limit {
            query = query.limit(Some(limit));
        }

        if let Some(offset) = filter.offset {
            query = query.offset(Some(offset));
        }

        let requests = query.all(&self.db).await?;
        Ok(requests)
    }

    pub async fn get_business_requests(
        &self,
        business_id: u32,
        filter: crate::request_lifecycle::RequestFilter,
    ) -> Result<Vec<Model>> {
        let mut query = Entity::find()
            .filter(Column::BusinessId.eq(business_id))
            .order_by_desc(Column::CreatedAt);

        // Apply filters
        if let Some(request_type) = filter.request_type {
            query = query.filter(Column::RequestType.eq(request_type));
        }

        if let Some(status) = filter.status {
            query = query.filter(Column::RequestStatus.eq(status));
        }

        if let Some(source_type) = filter.source_type {
            query = query.filter(Column::SourceType.eq(source_type));
        }

        if let Some(date_from) = filter.date_from {
            query = query.filter(Column::CreatedAt.gte(date_from));
        }

        if let Some(date_to) = filter.date_to {
            query = query.filter(Column::CreatedAt.lte(date_to));
        }

        if let Some(limit) = filter.limit {
            query = query.limit(Some(limit));
        }

        if let Some(offset) = filter.offset {
            query = query.offset(Some(offset));
        }

        let requests = query.all(&self.db).await?;
        Ok(requests)
    }
}
