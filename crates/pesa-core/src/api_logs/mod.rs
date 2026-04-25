use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, prelude::DateTimeUtc,
};
use serde::{Deserialize, Serialize};

use crate::utils::identifiers::Identifiers;

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiLog {
    pub id: String,
    pub project_id: u32,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub created_at: DateTimeUtc,
    pub error_desc: Option<String>,
    pub duration: u32,
    pub request_id: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UpdateApiLogRequest {
    pub status_code: Option<u16>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_desc: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ApiLogFilter {
    pub project_id: Option<u32>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub status_code: Option<u16>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

// Helper struct for API log statistics
#[derive(serde::Serialize)]
pub struct ApiLogStats {
    pub total_count: i64,
    pub success_count: i64,
    pub client_error_count: i64,
    pub server_error_count: i64,
}

impl From<db::Model> for ApiLog {
    fn from(value: db::Model) -> Self {
        Self {
            id: value.id,
            project_id: value.project_id,
            method: value.method,
            path: value.path,
            status_code: value.status_code,
            request_body: value.request_body,
            response_body: value.response_body,
            created_at: value.created_at,
            error_desc: value.error_desc,
            duration: value.duration,
            request_id: value.request_id,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiLogBuilder {
    id: Option<String>,
    project_id: Option<u32>,
    method: Option<String>,
    path: Option<String>,
    status_code: Option<u16>,
    request_body: Option<String>,
    response_body: Option<String>,
    created_at: Option<DateTimeUtc>,
    error_desc: Option<String>,
    duration: Option<u32>,
}

impl ApiLogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn project_id(mut self, project_id: u32) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn method<S: Into<String>>(mut self, method: S) -> Self {
        self.method = Some(method.into());
        self
    }

    pub fn path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn request_body<S: Into<String>>(mut self, request_body: S) -> Self {
        self.request_body = Some(request_body.into());
        self
    }

    pub fn response_body<S: Into<String>>(mut self, response_body: S) -> Self {
        self.response_body = Some(response_body.into());
        self
    }

    pub fn created_at(mut self, created_at: DateTimeUtc) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn error_desc<S: Into<String>>(mut self, error_desc: S) -> Self {
        self.error_desc = Some(error_desc.into());
        self
    }

    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub async fn save<C: ConnectionTrait>(
        self,
        conn: &C,
        ids: &Identifiers,
    ) -> anyhow::Result<ApiLog> {
        let api = ApiLog {
            id: ids.request_id.clone(),
            project_id: self
                .project_id
                .ok_or(ApiLogBuilderError::MissingField("project_id"))?,
            method: self
                .method
                .ok_or(ApiLogBuilderError::MissingField("method"))?,
            path: self.path.ok_or(ApiLogBuilderError::MissingField("path"))?,
            status_code: self
                .status_code
                .ok_or(ApiLogBuilderError::MissingField("status_code"))?,
            request_body: self.request_body.clone(),
            response_body: self.response_body,
            created_at: self
                .created_at
                .ok_or(ApiLogBuilderError::MissingField("created_at"))?,
            error_desc: self.error_desc.clone(),
            duration: self
                .duration
                .ok_or(ApiLogBuilderError::MissingField("duration"))?,
            request_id: Some(ids.request_id.clone()),
        };

        let create_api = db::ActiveModel {
            id: Set(api.id.clone()),
            project_id: Set(api.project_id),
            method: Set(api.method.clone()),
            path: Set(api.path.clone()),
            status_code: Set(api.status_code),
            request_body: Set(api.request_body.clone()),
            response_body: Set(api.response_body.clone()),
            created_at: Set(api.created_at),
            error_desc: Set(api.error_desc.clone()),
            duration: Set(api.duration),
            request_id: Set(api.request_id.clone()),
        };

        create_api.insert(conn).await?;

        Ok(api)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiLogBuilderError {
    MissingField(&'static str),
}

impl std::fmt::Display for ApiLogBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiLogBuilderError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
        }
    }
}

impl std::error::Error for ApiLogBuilderError {}

impl ApiLog {
    pub fn builder() -> ApiLogBuilder {
        ApiLogBuilder::new()
    }

    pub async fn read_by_id<C: ConnectionTrait>(
        db: &C,
        id: String,
    ) -> anyhow::Result<Option<ApiLog>> {
        let log = db::Entity::find_by_id(id).one(db).await?;
        Ok(log.map(|l| l.into()))
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: String,
        request: UpdateApiLogRequest,
    ) -> anyhow::Result<Option<ApiLog>> {
        let log = db::Entity::find_by_id(id).one(db).await?;

        if log.is_none() {
            return Ok(None);
        }

        let mut log: db::ActiveModel = log.unwrap().into();

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

        let model = log.update(db).await?;

        Ok(Some(model.into()))
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: String) -> anyhow::Result<bool> {
        let res = db::Entity::delete_by_id(id).exec(db).await?;
        Ok(res.rows_affected > 0)
    }

    pub async fn find<C: ConnectionTrait>(
        db: &C,
        filter: ApiLogFilter,
    ) -> anyhow::Result<Vec<ApiLog>> {
        let mut q = db::Entity::find().order_by_desc(db::Column::CreatedAt);

        if let Some(project) = filter.project_id {
            q = q.filter(db::Column::ProjectId.eq(project));
        }

        if let Some(method) = filter.method {
            q = q.filter(db::Column::Method.eq(method));
        }

        if let Some(path) = filter.path {
            q = q.filter(db::Column::Path.eq(path));
        }

        if let Some(status) = filter.status_code {
            q = q.filter(db::Column::StatusCode.eq(status));
        }

        if let Some(limit) = filter.limit {
            q = q.limit(Some(limit));
        }

        if let Some(offset) = filter.offset {
            q = q.offset(Some(offset));
        }

        let logs = q.all(db).await?;

        Ok(logs.into_iter().map(|log| log.into()).collect())
    }

    pub async fn count<C: ConnectionTrait>(db: &C, filter: ApiLogFilter) -> anyhow::Result<usize> {
        let mut q = db::Entity::find();

        if let Some(project) = filter.project_id {
            q = q.filter(db::Column::ProjectId.eq(project));
        }

        if let Some(method) = filter.method {
            q = q.filter(db::Column::Method.eq(method));
        }

        if let Some(path) = filter.path {
            q = q.filter(db::Column::Path.eq(path));
        }

        if let Some(status) = filter.status_code {
            q = q.filter(db::Column::StatusCode.eq(status));
        }

        Ok(q.all(db).await?.len())
    }

    pub async fn get_by_request_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> anyhow::Result<Option<ApiLog>> {
        let log = crate::api_logs::db::Entity::find()
            .filter(crate::api_logs::db::Column::RequestId.eq(id))
            .one(db)
            .await?;
        Ok(log.map(|l| l.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::projects::{CreateProject, Project, SimulationMode};
    use crate::tests::TestDb;
    use chrono::Utc;

    async fn setup_test_project(db: &TestDb) -> u32 {
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
            txn_delay: 0,
            prefix: None,
        };
        let project = Project::create(&db.conn, project_input).await.unwrap();
        project.id
    }

    async fn setup_test_request(db: &TestDb, identifiers: &Identifiers, project_id: u32) {
        use crate::request_lifecycle::{RequestLifecycleManager, RequestType};
        let manager = RequestLifecycleManager::new(db.conn.clone());
        manager
            .create_system_request(
                identifiers,
                "test",
                RequestType::StkPush,
                Some(project_id),
                None,
                None,
            )
            .await
            .unwrap();
    }

    fn create_test_log_builder(project_id: u32) -> ApiLogBuilder {
        ApiLog::builder()
            .project_id(project_id)
            .method("POST")
            .path("/stkpush")
            .status_code(200)
            .created_at(Utc::now().to_utc())
            .duration(150)
    }

    #[tokio::test]
    async fn test_create_api_log() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;
        let mut ids = Identifiers::new();
        ids.request_id = "test-req-id".to_string();
        setup_test_request(&db, &ids, project_id).await;

        let result = create_test_log_builder(project_id)
            .save(&db.conn, &ids)
            .await;

        assert!(result.is_ok());
        let log = result.unwrap();
        assert_eq!(log.id, "test-req-id");
        assert_eq!(log.project_id, project_id);
    }

    #[tokio::test]
    async fn test_read_api_log_by_id() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;
        let mut ids = Identifiers::new();
        ids.request_id = "test-req-id".to_string();
        setup_test_request(&db, &ids, project_id).await;

        create_test_log_builder(project_id)
            .save(&db.conn, &ids)
            .await
            .unwrap();

        let result = ApiLog::read_by_id(&db.conn, "test-req-id".to_string())
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().method, "POST");
    }

    #[tokio::test]
    async fn test_update_api_log() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;
        let mut ids = Identifiers::new();
        ids.request_id = "test-req-id".to_string();
        setup_test_request(&db, &ids, project_id).await;

        create_test_log_builder(project_id)
            .save(&db.conn, &ids)
            .await
            .unwrap();

        let update = UpdateApiLogRequest {
            status_code: Some(400),
            error_desc: Some("Bad Request".to_string()),
            ..Default::default()
        };

        let result = ApiLog::update(&db.conn, "test-req-id".to_string(), update)
            .await
            .unwrap();
        assert!(result.is_some());
        let updated = result.unwrap();
        assert_eq!(updated.status_code, 400);
        assert_eq!(updated.error_desc, Some("Bad Request".to_string()));
    }

    #[tokio::test]
    async fn test_delete_api_log() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;
        let mut ids = Identifiers::new();
        ids.request_id = "test-req-id".to_string();
        setup_test_request(&db, &ids, project_id).await;

        create_test_log_builder(project_id)
            .save(&db.conn, &ids)
            .await
            .unwrap();

        let result = ApiLog::delete(&db.conn, "test-req-id".to_string())
            .await
            .unwrap();
        assert!(result);

        let verify = ApiLog::read_by_id(&db.conn, "test-req-id".to_string())
            .await
            .unwrap();
        assert!(verify.is_none());
    }

    #[tokio::test]
    async fn test_find_api_logs() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        for i in 1..=3 {
            let mut ids = Identifiers::new();
            ids.request_id = format!("req-{}", i);
            setup_test_request(&db, &ids, project_id).await;
            create_test_log_builder(project_id)
                .save(&db.conn, &ids)
                .await
                .unwrap();
        }

        let filter = ApiLogFilter {
            project_id: Some(project_id),
            ..Default::default()
        };
        let logs = ApiLog::find(&db.conn, filter).await.unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[tokio::test]
    async fn test_count_api_logs() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        for i in 1..=5 {
            let mut ids = Identifiers::new();
            ids.request_id = format!("req-{}", i);
            setup_test_request(&db, &ids, project_id).await;
            create_test_log_builder(project_id)
                .save(&db.conn, &ids)
                .await
                .unwrap();
        }

        let filter = ApiLogFilter {
            project_id: Some(project_id),
            ..Default::default()
        };
        let count = ApiLog::count(&db.conn, filter).await.unwrap();
        assert_eq!(count, 5);
    }
}
