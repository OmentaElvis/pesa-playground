use chrono::{DateTime, Utc};

use anyhow::Context;
use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, RelationTrait,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::api_keys::ApiKey;

pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Display, EnumString, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum SimulationMode {
    AlwaysSuccess,
    AlwaysFail,
    Random,
    Realistic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateProject {
    pub business_id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub stk_delay: u32,
    pub prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: Option<SimulationMode>,
    pub stk_delay: Option<u32>,
    pub prefix: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ProjectCredentials {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub project_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDetails {
    pub id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub business_id: u32,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTime<Utc>,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
}

#[derive(Serialize, Deserialize, Debug, Default, FromQueryResult)]
pub struct ProjectSummary {
    pub id: u32,
    pub name: String,
    pub simulation_mode: String,
    pub business_id: u32,
    pub business_name: String,
    pub created_at: DateTime<Utc>,
    pub short_code: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub simulation_mode: Option<SimulationMode>,
    pub has_callback_url: Option<bool>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Project {
    pub async fn get_by_id<C>(db: &C, id: u32) -> Result<Option<Project>, DbErr>
    where
        C: ConnectionTrait,
    {
        let project = db::Entity::find_by_id(id).one(db).await?;
        Ok(project.as_ref().map(|p| p.into()))
    }

    pub async fn get_by_name<C>(db: &C, name: &str) -> Result<Option<Project>, DbErr>
    where
        C: ConnectionTrait,
    {
        let project = db::Entity::find()
            .filter(db::Column::Name.eq(name))
            .one(db)
            .await?;
        Ok(project.as_ref().map(|p| p.into()))
    }

    pub async fn create<C>(db: &C, input: CreateProject) -> anyhow::Result<ProjectDetails>
    where
        C: ConnectionTrait,
    {
        use sea_orm::ActiveModelTrait;
        use sea_orm::ActiveValue::Set;

        let create = db::ActiveModel {
            business_id: Set(input.business_id),
            name: Set(input.name.clone()),
            callback_url: Set(input.callback_url),
            prefix: Set(input.prefix),
            simulation_mode: Set(input.simulation_mode.to_string()),
            stk_delay: Set(input.stk_delay),
            created_at: Set(Utc::now().to_utc()),
            ..Default::default()
        };

        let project = create
            .insert(db)
            .await
            .context("Failed to create project")?;

        let key = crate::api_keys::ApiKey::generate(project.id);
        let key = ApiKey::create(db, key)
            .await
            .context("Failed to create api keys")?;

        Ok(ProjectDetails {
            id: project.id,
            name: project.name,
            callback_url: project.callback_url,
            simulation_mode: input.simulation_mode,
            stk_delay: project.stk_delay,
            prefix: project.prefix,
            created_at: project.created_at,
            consumer_key: key.consumer_key,
            consumer_secret: key.consumer_secret,
            passkey: key.passkey,
            business_id: project.business_id,
        })
    }

    pub async fn get_all<C>(db: &C) -> anyhow::Result<Vec<ProjectSummary>>
    where
        C: ConnectionTrait,
    {
        use sea_orm::{JoinType, QuerySelect};

        let projects = db::Entity::find()
            .join(JoinType::InnerJoin, db::Relation::Business.def())
            .select_only()
            .column(db::Column::Id)
            .column(db::Column::Name)
            .column(db::Column::SimulationMode)
            .column(db::Column::CreatedAt)
            .column(db::Column::BusinessId)
            .column(db::Column::StkDelay)
            .column(crate::business::db::Column::ShortCode)
            .column_as(crate::business::db::Column::Name, "business_name")
            .into_model::<ProjectSummary>()
            .all(db)
            .await
            .context("Failed to fetch projects")?;

        Ok(projects)
    }

    pub async fn get_by_business_id<C>(
        db: &C,
        business_id: u32,
    ) -> anyhow::Result<Vec<ProjectSummary>>
    where
        C: ConnectionTrait,
    {
        use sea_orm::{JoinType, QueryFilter, QuerySelect};

        let projects = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .join(JoinType::InnerJoin, db::Relation::Business.def())
            .select_only()
            .column(db::Column::Id)
            .column(db::Column::Name)
            .column(db::Column::SimulationMode)
            .column(db::Column::CreatedAt)
            .column(db::Column::BusinessId)
            .column_as(crate::business::db::Column::Name, "business_name")
            .column(crate::business::db::Column::ShortCode)
            .into_model::<ProjectSummary>()
            .all(db)
            .await
            .context(format!(
                "Failed to fetch projects for business {}",
                business_id
            ))?;

        Ok(projects)
    }

    pub async fn get_details<C>(db: &C, id: u32) -> anyhow::Result<ProjectDetails>
    where
        C: ConnectionTrait,
    {
        let project = db::Entity::find_by_id(id)
            .one(db)
            .await
            .context(format!("Failed to fetch project with ID {}", id))?
            .ok_or_else(|| anyhow::anyhow!("Project with ID {} not found", id))?;

        let api_key = crate::api_keys::ApiKey::read_by_project_id(db, project.id as u32)
            .await
            .context(format!(
                "Failed to fetch API keys for project {}",
                project.id
            ))?
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to fetch API keys for project {}", project.id)
            })?;

        Ok(ProjectDetails {
            id: project.id,
            name: project.name,
            callback_url: project.callback_url,
            stk_delay: project.stk_delay,
            created_at: project.created_at,
            simulation_mode: project
                .simulation_mode
                .parse()
                .unwrap_or(SimulationMode::Realistic),
            prefix: project.prefix,
            consumer_key: api_key.consumer_key,
            consumer_secret: api_key.consumer_secret,
            passkey: api_key.passkey,
            business_id: project.business_id,
        })
    }

    pub async fn update<C>(db: &C, id: u32, input: UpdateProject) -> anyhow::Result<Option<Project>>
    where
        C: ConnectionTrait,
    {
        use sea_orm::{ActiveModelTrait, ActiveValue::Set};

        let project = db::Entity::find_by_id(id)
            .one(db)
            .await
            .context(format!("Failed to fetch project with ID {}", id))?
            .ok_or_else(|| anyhow::anyhow!("Project with ID {} not found", id))?;

        let mut active_model: db::ActiveModel = project.into();

        if let Some(name) = input.name {
            active_model.name = Set(name);
        }
        if let Some(callback_url) = input.callback_url {
            active_model.callback_url = Set(Some(callback_url));
        }
        if let Some(simulation_mode) = input.simulation_mode {
            active_model.simulation_mode = Set(simulation_mode.to_string());
        }
        if let Some(stk_delay) = input.stk_delay {
            active_model.stk_delay = Set(stk_delay);
        }
        if let Some(prefix) = input.prefix {
            active_model.prefix = Set(Some(prefix));
        }

        let updated_project = active_model
            .update(db)
            .await
            .context(format!("Failed to update project {}", id))?;

        Ok(Some(Project {
            id: updated_project.id,
            name: updated_project.name,
            callback_url: updated_project.callback_url,
            simulation_mode: updated_project
                .simulation_mode
                .parse()
                .unwrap_or(SimulationMode::Realistic),
            stk_delay: updated_project.stk_delay,
            prefix: updated_project.prefix,
            created_at: updated_project.created_at,
        }))
    }

    pub async fn delete<C>(db: &C, id: u32) -> anyhow::Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(id)
            .exec(db)
            .await
            .context(format!("Failed to delete project with ID {}", id))?;

        Ok(result.rows_affected > 0)
    }
}

impl From<&db::Model> for Project {
    fn from(value: &db::Model) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
            callback_url: value.callback_url.clone(),
            simulation_mode: value
                .simulation_mode
                .parse()
                .unwrap_or(SimulationMode::Realistic),
            stk_delay: value.stk_delay,
            prefix: value.prefix.clone(),
            created_at: value.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::tests::TestDb;

    async fn setup_test_business(db: &TestDb) -> Business {
        let input = CreateBusiness {
            name: "Test Business".to_string(),
            short_code: "999999".to_string(),
            initial_working_balance: 1000.0,
            initial_utility_balance: 500.0,
        };
        Business::create(&db.conn, input).await.unwrap()
    }

    fn create_test_project_input(business_id: u32) -> CreateProject {
        CreateProject {
            business_id,
            name: "Test Project".to_string(),
            callback_url: Some("https://example.com/callback".to_string()),
            simulation_mode: SimulationMode::Realistic,
            stk_delay: 1000,
            prefix: Some("TEST".to_string()),
        }
    }

    #[tokio::test]
    async fn test_create_project_success() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        let input = create_test_project_input(business.id);
        let result = Project::create(&db.conn, input).await;

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.business_id, business.id);
        assert_eq!(project.simulation_mode, SimulationMode::Realistic);
        assert!(!project.consumer_key.is_empty());
        assert!(!project.consumer_secret.is_empty());
        assert!(!project.passkey.is_empty());
    }

    #[tokio::test]
    async fn test_create_project_creates_api_keys() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        let input = create_test_project_input(business.id);
        let project = Project::create(&db.conn, input).await.unwrap();

        // Verify API keys exist
        let api_key = crate::api_keys::ApiKey::read_by_project_id(&db.conn, project.id)
            .await
            .unwrap();
        assert!(api_key.is_some());
        let key = api_key.unwrap();
        assert_eq!(key.consumer_key, project.consumer_key);
    }

    #[tokio::test]
    async fn test_get_project_by_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Project::get_by_id(&db.conn, 9999).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_project_details_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Project::get_details(&db.conn, 9999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_project_by_name_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Project::get_by_name(&db.conn, "NonExistent").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_all_projects_empty() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Project::get_all(&db.conn).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_projects() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        Project::create(&db.conn, create_test_project_input(business.id))
            .await
            .unwrap();

        let result = Project::get_all(&db.conn).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Test Project");
    }

    #[tokio::test]
    async fn test_get_projects_by_business_id() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        Project::create(&db.conn, create_test_project_input(business.id))
            .await
            .unwrap();

        let result = Project::get_by_business_id(&db.conn, business.id)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_update_project_name() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        let created = Project::create(&db.conn, create_test_project_input(business.id))
            .await
            .unwrap();

        let update_input = UpdateProject {
            name: Some("Updated Project".to_string()),
            ..Default::default()
        };

        let result = Project::update(&db.conn, created.id, update_input)
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Updated Project");
    }

    #[tokio::test]
    async fn test_update_project_simulation_mode() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        let created = Project::create(&db.conn, create_test_project_input(business.id))
            .await
            .unwrap();

        let update_input = UpdateProject {
            simulation_mode: Some(SimulationMode::AlwaysFail),
            ..Default::default()
        };

        let result = Project::update(&db.conn, created.id, update_input)
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().simulation_mode, SimulationMode::AlwaysFail);
    }

    #[tokio::test]
    async fn test_update_project_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let update_input = UpdateProject {
            name: Some("Test".to_string()),
            ..Default::default()
        };

        let result = Project::update(&db.conn, 9999, update_input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_project() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        let created = Project::create(&db.conn, create_test_project_input(business.id))
            .await
            .unwrap();

        let result = Project::delete(&db.conn, created.id).await.unwrap();

        assert!(result);

        // Verify deleted
        let verify = Project::get_by_id(&db.conn, created.id).await.unwrap();
        assert!(verify.is_none());
    }

    #[tokio::test]
    async fn test_delete_project_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let result = Project::delete(&db.conn, 9999).await.unwrap();

        assert!(!result);
    }

    #[tokio::test]
    async fn test_project_simulation_modes() {
        let db = TestDb::in_memory().await.unwrap();
        let business = setup_test_business(&db).await;

        for mode in &[
            SimulationMode::AlwaysSuccess,
            SimulationMode::AlwaysFail,
            SimulationMode::Random,
            SimulationMode::Realistic,
        ] {
            let input = CreateProject {
                business_id: business.id,
                name: format!("{:?} Project", mode),
                callback_url: None,
                simulation_mode: mode.clone(),
                stk_delay: 0,
                prefix: None,
            };
            let project = Project::create(&db.conn, input).await.unwrap();
            assert_eq!(project.simulation_mode, *mode);
        }
    }
}
