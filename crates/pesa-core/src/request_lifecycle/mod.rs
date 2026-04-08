use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
pub use db::RequestStatus;
pub use db::RequestType;
pub use db::SourceType;

use sea_orm::ActiveModelTrait;
use sea_orm::RelationTrait;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod manager;
pub mod paths;
pub mod request_ids;
pub mod stats;
pub mod ui;

pub use manager::RequestLifecycleManager;

use crate::api_logs::ApiLog;
use crate::transaction_jobs::TransactionJob;

impl RequestType {
    pub fn from_path(path: &str) -> Self {
        match path {
            paths::OAUTH => RequestType::Oauth,
            paths::STK_PUSH => RequestType::StkPush,
            paths::C2B_REGISTER_URL => RequestType::C2bRegisterUrl,
            paths::B2C_PAYMENT => RequestType::B2cPayment,
            paths::BALANCE_QUERY => RequestType::BalanceQuery,
            _ => RequestType::Other,
        }
    }

    pub fn is_trackable(&self) -> bool {
        !matches!(self, RequestType::Other)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub source_type: SourceType,
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
            request_type: model.request_type,
            request_status: model.request_status,
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CreateRequest {
    pub id: String,
    pub source_type: SourceType,
    pub source_api_key_id: Option<u32>,
    pub source_component: Option<String>,
    pub request_type: RequestType,
    pub project_id: Option<u32>,
    pub business_id: Option<u32>,
    pub user_id: Option<u32>,
    pub request_body: Option<String>,
}

impl CreateRequest {
    pub fn new_api(
        id: String,
        api_key_id: u32,
        project_id: Option<u32>,
        request_type: RequestType,
        request_body: Option<String>,
    ) -> Self {
        Self {
            id,
            source_type: SourceType::Api,
            source_api_key_id: Some(api_key_id),
            source_component: None,
            request_type,
            project_id,
            business_id: None,
            user_id: None,
            request_body,
        }
    }

    pub fn new_system(
        id: String,
        component: String,
        request_type: RequestType,
        project_id: Option<u32>,
        business_id: Option<u32>,
        user_id: Option<u32>,
    ) -> Self {
        Self {
            id,
            source_type: SourceType::System,
            source_api_key_id: None,
            source_component: Some(component),
            request_type,
            project_id,
            business_id,
            user_id,
            request_body: None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FindRequestQuery {
    pub id_type: String,
    pub external_id: String,
}

impl FullRequestView {
    pub async fn get_full_request<C>(conn: &C, request_id: &str) -> Result<Option<FullRequestView>>
    where
        C: ConnectionTrait,
    {
        let request_model = Request::get_request(conn, request_id).await?;

        if let Some(request_model) = request_model {
            let external_ids =
                request_ids::RequestId::find_by_request_id(conn, request_id.to_string()).await?;

            let api_log = ApiLog::get_by_request_id(conn, request_id).await?;

            let transaction_models = crate::transactions::db::Entity::find()
                .filter(crate::transactions::db::Column::RequestId.eq(request_id))
                .all(conn)
                .await?;

            let callback_models = crate::callbacks::db::Entity::find()
                .filter(crate::callbacks::db::Column::RequestId.eq(request_id))
                .all(conn)
                .await?;

            let job_models = TransactionJob::get_by_request_id(conn, request_id)
                .await
                .context("Failed to fetch transaction jobs")?;

            Ok(Some(FullRequestView {
                request: request_model,
                external_ids,
                api_log,
                transactions: transaction_models.into_iter().map(Into::into).collect(),
                callbacks: callback_models.into_iter().map(Into::into).collect(),
                jobs: job_models,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn find_request<C>(
        conn: &C,
        id_type: &str,
        external_id: &str,
    ) -> Result<Option<FullRequestView>>
    where
        C: ConnectionTrait,
    {
        let request = Request::find_by_external_id(conn, id_type, external_id).await?;

        if let Some(request) = request {
            Self::get_full_request(conn, &request.id)
                .await
                .context("Failed to get full request")
        } else {
            Ok(None)
        }
    }
}

impl Request {
    pub async fn get_request<C>(conn: &C, request_id: &str) -> Result<Option<Request>, DbErr>
    where
        C: ConnectionTrait,
    {
        db::Entity::find_by_id(request_id)
            .one(conn)
            .await
            .map(|opt| opt.map(Into::into))
    }

    pub async fn find_by_external_id<C>(
        conn: &C,
        id_type: &str,
        external_id: &str,
    ) -> Result<Option<Request>, DbErr>
    where
        C: ConnectionTrait,
    {
        use db::Relation;
        use request_ids::db as request_ids_db;
        use sea_orm::JoinType;

        db::Entity::find()
            .join(JoinType::InnerJoin, Relation::RequestIds.def())
            .filter(request_ids_db::Column::IdType.eq(id_type))
            .filter(request_ids_db::Column::ExternalId.eq(external_id))
            .one(conn)
            .await
            .map(|opt| opt.map(Into::into))
    }

    pub async fn get_project_requests<C>(
        conn: &C,
        project_id: u32,
        filter: RequestFilter,
    ) -> Result<Vec<Request>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = db::Entity::find()
            .filter(db::Column::ProjectId.eq(project_id))
            .order_by_desc(db::Column::CreatedAt);

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

        let requests = query.all(conn).await?.into_iter().map(Into::into).collect();
        Ok(requests)
    }

    pub async fn get_business_requests<C>(
        conn: &C,
        business_id: u32,
        filter: RequestFilter,
    ) -> Result<Vec<Request>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .order_by_desc(db::Column::CreatedAt);

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

        let requests = query.all(conn).await?.into_iter().map(Into::into).collect();
        Ok(requests)
    }

    pub async fn list_requests<C>(conn: &C, filter: RequestFilter) -> Result<Vec<Request>, DbErr>
    where
        C: ConnectionTrait,
    {
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

        let requests = query.all(conn).await?.into_iter().map(Into::into).collect();
        Ok(requests)
    }

    pub async fn count_requests<C>(conn: &C, filter: RequestFilter) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
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

        query.count(conn).await
    }

    pub async fn create<C>(conn: &C, create: CreateRequest) -> Result<Request, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();

        let active_model = db::ActiveModel {
            id: Set(create.id.clone()),
            source_type: Set(create.source_type.clone()),
            source_api_key_id: Set(create.source_api_key_id),
            source_operator_id: Set(None),
            source_component: Set(create.source_component.clone()),
            request_type: Set(create.request_type.clone()),
            request_status: Set(RequestStatus::Pending),
            created_at: Set(now),
            started_at: Set(None),
            completed_at: Set(None),
            project_id: Set(create.project_id),
            business_id: Set(create.business_id),
            user_id: Set(create.user_id),
            request_body: Set(create.request_body.clone()),
            response_body: Set(None),
            error_message: Set(None),
        };

        let _ = db::Entity::insert(active_model).exec(conn).await?;

        Ok(Request {
            id: create.id,
            source_type: create.source_type,
            source_api_key_id: create.source_api_key_id,
            source_operator_id: None,
            source_component: create.source_component,
            request_type: create.request_type,
            request_status: RequestStatus::Pending,
            created_at: now,
            started_at: None,
            completed_at: None,
            project_id: create.project_id,
            business_id: create.business_id,
            user_id: create.user_id,
            request_body: create.request_body,
            response_body: None,
            error_message: None,
        })
    }

    pub async fn start<C>(conn: &C, request_id: &str) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_query::Expr;

        let now = Utc::now();
        db::Entity::update_many()
            .filter(db::Column::Id.eq(request_id))
            .col_expr(db::Column::StartedAt, Expr::value(now))
            .exec(conn)
            .await?;
        Ok(())
    }

    pub async fn complete<C>(
        conn: &C,
        request_id: &str,
        response_body: Option<String>,
        error_message: Option<String>,
    ) -> Result<Request, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        let status = if error_message.is_some() {
            RequestStatus::Failed
        } else {
            RequestStatus::Completed
        };

        let mut active_model = db::Entity::find_by_id(request_id)
            .one(conn)
            .await?
            .ok_or(DbErr::RecordNotFound(format!(
                "Request {} not found",
                request_id
            )))?
            .into_active_model();

        active_model.request_status = Set(status);
        active_model.completed_at = Set(Some(now));

        if let Some(body) = response_body {
            active_model.response_body = Set(Some(body));
        }

        if let Some(error) = error_message {
            active_model.error_message = Set(Some(error));
        }

        let result = active_model.update(conn).await?;
        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::projects::{CreateProject, Project, SimulationMode};
    use crate::tests::TestDb;

    async fn setup_test_request_data(db: &TestDb) -> (u32, u32, u32) {
        let business_input = CreateBusiness {
            name: "Test Business".to_string(),
            short_code: "999999".to_string(),
            initial_working_balance: 1000.0,
            initial_utility_balance: 500.0,
        };
        let business = Business::create(&db.conn, business_input).await.unwrap();

        let project_input = CreateProject {
            business_id: business.id,
            name: "Test Project".to_string(),
            callback_url: None,
            simulation_mode: SimulationMode::Realistic,
            stk_delay: 0,
            prefix: None,
        };
        let project = Project::create(&db.conn, project_input).await.unwrap();

        let api_keys = crate::api_keys::ApiKey::read_by_project_id(&db.conn, project.id)
            .await
            .unwrap()
            .unwrap();

        (api_keys.id, project.id, business.id)
    }

    #[tokio::test]
    async fn test_create_api_request() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, project_id, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_api_001".to_string(),
            api_key_id,
            Some(project_id),
            RequestType::StkPush,
            Some(r#"{"amount": 100}"#.to_string()),
        );

        let result = Request::create(&db.conn, create).await;
        assert!(result.is_ok(), "Failed to create request: {:?}", result);

        let request = result.unwrap();
        assert_eq!(request.id, "req_api_001");
        assert_eq!(request.source_type, SourceType::Api);
        assert_eq!(request.source_api_key_id, Some(api_key_id));
        assert_eq!(request.source_component, None);
        assert_eq!(request.request_type, RequestType::StkPush);
        assert_eq!(request.request_status, RequestStatus::Pending);
        assert_eq!(request.request_body, Some(r#"{"amount": 100}"#.to_string()));
        assert_eq!(request.project_id, Some(project_id));
    }

    #[tokio::test]
    async fn test_create_system_request() {
        let db = TestDb::in_memory().await.unwrap();
        let (_, project_id, business_id) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_system(
            "req_sys_001".to_string(),
            "test_component".to_string(),
            RequestType::B2cPayment,
            Some(project_id),
            Some(business_id),
            None,
        );

        let result = Request::create(&db.conn, create).await;
        assert!(result.is_ok(), "Failed to create request: {:?}", result);

        let request = result.unwrap();
        assert_eq!(request.id, "req_sys_001");
        assert_eq!(request.source_type, SourceType::System);
        assert_eq!(request.source_api_key_id, None);
        assert_eq!(request.source_component, Some("test_component".to_string()));
        assert_eq!(request.request_type, RequestType::B2cPayment);
        assert_eq!(request.project_id, Some(project_id));
        assert_eq!(request.business_id, Some(business_id));
    }

    #[tokio::test]
    async fn test_get_request_by_id() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_get_001".to_string(),
            api_key_id,
            None,
            RequestType::StkPush,
            None,
        );
        Request::create(&db.conn, create).await.unwrap();

        let result = Request::get_request(&db.conn, "req_get_001").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_request_by_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Request::get_request(&db.conn, "nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_start_request() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_start_001".to_string(),
            api_key_id,
            None,
            RequestType::StkPush,
            None,
        );
        Request::create(&db.conn, create).await.unwrap();

        let result = Request::start(&db.conn, "req_start_001").await;
        assert!(result.is_ok());

        let request = Request::get_request(&db.conn, "req_start_001")
            .await
            .unwrap()
            .unwrap();
        assert!(request.started_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_request_success() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_complete_001".to_string(),
            api_key_id,
            None,
            RequestType::StkPush,
            None,
        );
        Request::create(&db.conn, create).await.unwrap();

        let result = Request::complete(
            &db.conn,
            "req_complete_001",
            Some(r#"{"status": "success"}"#.to_string()),
            None,
        )
        .await;
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.request_status, RequestStatus::Completed);
        assert_eq!(
            request.response_body,
            Some(r#"{"status": "success"}"#.to_string())
        );
        assert!(request.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_request_failure() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_fail_001".to_string(),
            api_key_id,
            None,
            RequestType::StkPush,
            None,
        );
        Request::create(&db.conn, create).await.unwrap();

        let result = Request::complete(
            &db.conn,
            "req_fail_001",
            None,
            Some("Insufficient funds".to_string()),
        )
        .await;
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.request_status, RequestStatus::Failed);
        assert_eq!(
            request.error_message,
            Some("Insufficient funds".to_string())
        );
        assert!(request.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_request_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Request::complete(&db.conn, "nonexistent", None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_requests() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        for i in 1..=5 {
            let create = CreateRequest::new_api(
                format!("req_list_{:03}", i),
                api_key_id,
                None,
                RequestType::StkPush,
                None,
            );
            Request::create(&db.conn, create).await.unwrap();
        }

        let result = Request::list_requests(&db.conn, RequestFilter::default()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_list_requests_with_limit() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        for i in 1..=10 {
            let create = CreateRequest::new_api(
                format!("req_limit_{:03}", i),
                api_key_id,
                None,
                RequestType::StkPush,
                None,
            );
            Request::create(&db.conn, create).await.unwrap();
        }

        let filter = RequestFilter {
            limit: Some(3),
            ..Default::default()
        };
        let result = Request::list_requests(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_list_requests_with_offset() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        for i in 1..=10 {
            let create = CreateRequest::new_api(
                format!("req_offset_{:03}", i),
                api_key_id,
                None,
                RequestType::StkPush,
                None,
            );
            Request::create(&db.conn, create).await.unwrap();
        }

        let filter = RequestFilter {
            limit: Some(10),
            offset: Some(5),
            ..Default::default()
        };
        let result = Request::list_requests(&db.conn, filter).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_count_requests() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        for i in 1..=5 {
            let create = CreateRequest::new_api(
                format!("req_count_{:03}", i),
                api_key_id,
                None,
                RequestType::StkPush,
                None,
            );
            Request::create(&db.conn, create).await.unwrap();
        }

        let result = Request::count_requests(&db.conn, RequestFilter::default()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[tokio::test]
    async fn test_request_type_from_path() {
        use super::paths::*;
        assert_eq!(RequestType::from_path(OAUTH), RequestType::Oauth);
        assert_eq!(RequestType::from_path(STK_PUSH), RequestType::StkPush);
        assert_eq!(RequestType::from_path(B2C_PAYMENT), RequestType::B2cPayment);
        assert_eq!(
            RequestType::from_path(BALANCE_QUERY),
            RequestType::BalanceQuery
        );
        assert_eq!(
            RequestType::from_path(C2B_REGISTER_URL),
            RequestType::C2bRegisterUrl
        );
        assert_eq!(RequestType::from_path("/unknown/path"), RequestType::Other);
    }

    #[tokio::test]
    async fn test_request_type_is_trackable() {
        assert!(RequestType::StkPush.is_trackable());
        assert!(RequestType::B2cPayment.is_trackable());
        assert!(RequestType::Oauth.is_trackable());
        assert!(RequestType::BalanceQuery.is_trackable());
        assert!(RequestType::C2bRegisterUrl.is_trackable());
        assert!(RequestType::C2bLipa.is_trackable());
        assert!(!RequestType::Other.is_trackable());
    }

    #[tokio::test]
    async fn test_create_request_idempotency() {
        let db = TestDb::in_memory().await.unwrap();
        let (api_key_id, _, _) = setup_test_request_data(&db).await;

        let create = CreateRequest::new_api(
            "req_idemp_1".to_string(),
            api_key_id,
            None,
            RequestType::StkPush,
            None,
        );

        let result1 = Request::create(&db.conn, create.clone()).await;
        assert!(result1.is_ok());

        let result2 = Request::create(&db.conn, create).await;
        assert!(result2.is_err());
    }
}
