use anyhow::{Context, Result};
use sea_orm::{prelude::*, *};
use sea_query::Expr;

use super::request_ids::db as request_ids_db;
use super::{CreateRequest, Request, RequestFilter};
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
        project_id: Option<u32>,
        request_type: super::RequestType,
        request_body: Option<String>,
    ) -> Result<Request> {
        let create = CreateRequest::new_api(
            identifiers.request_id.clone(),
            api_key_id,
            project_id,
            request_type,
            request_body,
        );
        Request::create(&self.db, create)
            .await
            .context("Failed to create API request")
    }

    pub async fn create_system_request(
        &self,
        identifiers: &Identifiers,
        component: &str,
        request_type: super::RequestType,
        project_id: Option<u32>,
        business_id: Option<u32>,
        user_id: Option<u32>,
    ) -> Result<Request> {
        let create = CreateRequest::new_system(
            identifiers.request_id.clone(),
            component.to_string(),
            request_type,
            project_id,
            business_id,
            user_id,
        );
        Request::create(&self.db, create)
            .await
            .context("Failed to create system request")
    }

    pub async fn start_request(&self, request_id: &str) -> Result<()> {
        Request::start(&self.db, request_id)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }

    pub async fn complete_request(
        &self,
        request_id: &str,
        response_body: Option<String>,
        error_message: Option<String>,
    ) -> Result<Request> {
        Request::complete(&self.db, request_id, response_body, error_message)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
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

    pub async fn get_request(&self, request_id: &str) -> Result<Option<Request>> {
        Request::get_request(&self.db, request_id)
            .await
            .context(format!("Failed to get request: {}", request_id))
    }

    pub async fn find_by_external_id(
        &self,
        id_type: &str,
        external_id: &str,
    ) -> Result<Option<Request>> {
        Request::find_by_external_id(&self.db, id_type, external_id)
            .await
            .context(format!(
                "Failed to get request by external id: {} ({})",
                external_id, id_type
            ))
    }

    pub async fn get_project_requests(
        &self,
        project_id: u32,
        filter: RequestFilter,
    ) -> Result<Vec<Request>> {
        Request::get_project_requests(&self.db, project_id, filter)
            .await
            .context("Failed to get project requests")
    }

    pub async fn get_business_requests(
        &self,
        business_id: u32,
        filter: RequestFilter,
    ) -> Result<Vec<Request>> {
        Request::get_business_requests(&self.db, business_id, filter)
            .await
            .context("Failed to get business requests")
    }

    pub async fn list_requests(&self, filter: RequestFilter) -> Result<Vec<Request>> {
        Request::list_requests(&self.db, filter)
            .await
            .context("Failed to list requests")
    }

    pub async fn count_requests(&self, filter: RequestFilter) -> Result<u64> {
        Request::count_requests(&self.db, filter)
            .await
            .context("Failed to count requests")
    }
}
