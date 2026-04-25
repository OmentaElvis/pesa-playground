use anyhow::Result;
use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use serde::Serialize;

pub mod db;

#[derive(Serialize, Debug, Clone)]
pub struct ApiKey {
    pub id: u32,
    pub project_id: u32,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
}

impl From<db::Model> for ApiKey {
    fn from(value: db::Model) -> Self {
        ApiKey {
            id: value.id,
            project_id: value.project_id,
            consumer_key: value.consumer_key,
            consumer_secret: value.consumer_secret,
            passkey: value.passkey,
        }
    }
}

impl ApiKey {
    pub fn generate(project_id: u32) -> ApiKey {
        let consumer_key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(18)
            .map(char::from)
            .collect();

        let consumer_secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        let passkey: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        ApiKey {
            id: 0,
            project_id,
            consumer_key,
            consumer_secret,
            passkey,
        }
    }

    pub async fn create<C>(db: &C, api_key: ApiKey) -> Result<ApiKey>
    where
        C: ConnectionTrait,
    {
        let mut create = db::ActiveModel {
            project_id: Set(api_key.project_id),
            consumer_key: Set(api_key.consumer_key),
            consumer_secret: Set(api_key.consumer_secret),
            passkey: Set(api_key.passkey),
            created_at: Set(Utc::now().to_utc()),
            ..Default::default()
        };

        if api_key.id != 0 {
            create.id = Set(api_key.id);
        }

        Ok(create.insert(db).await?.into())
    }

    pub async fn read_by_id<C>(db: &C, id: u32) -> Result<Option<ApiKey>>
    where
        C: ConnectionTrait,
    {
        let api = db::Entity::find_by_id(id).one(db).await?;

        Ok(api.map(|a| a.into()))
    }

    pub async fn read_by_project_id<C>(db: &C, id: u32) -> Result<Option<ApiKey>>
    where
        C: ConnectionTrait,
    {
        let api = db::Entity::find()
            .filter(db::Column::ProjectId.eq(id))
            .one(db)
            .await?;

        Ok(api.map(|a| a.into()))
    }

    pub async fn read_by_consumer_key<C>(db: &C, consumer_key: &str) -> Result<Option<ApiKey>>
    where
        C: ConnectionTrait,
    {
        let api_key = db::Entity::find()
            .filter(db::Column::ConsumerKey.eq(consumer_key))
            .one(db)
            .await?;

        Ok(api_key.map(|f| f.into()))
    }

    pub async fn update<C>(db: &C, api_key: &ApiKey) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let api = db::Entity::find_by_id(api_key.id).one(db).await?;

        if api.is_none() {
            return Ok(false);
        }

        let mut api: db::ActiveModel = api.unwrap().into();
        api.project_id = Set(api_key.project_id);
        api.consumer_key = Set(api_key.consumer_key.clone());
        api.consumer_secret = Set(api_key.consumer_secret.clone());
        api.passkey = Set(api_key.passkey.clone());

        api.update(db).await?;
        Ok(true)
    }

    pub async fn delete<C>(db: &C, id: u32) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::projects::{CreateProject, Project, SimulationMode};
    use crate::tests::TestDb;

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

    #[tokio::test]
    async fn test_generate_api_key() {
        let project_id = 1;
        let key = ApiKey::generate(project_id);

        assert_eq!(key.project_id, project_id);
        assert_eq!(key.consumer_key.len(), 18);
        assert_eq!(key.consumer_secret.len(), 40);
        assert_eq!(key.passkey.len(), 64);
    }

    #[tokio::test]
    async fn test_create_api_key_success() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let key_input = ApiKey::generate(project_id);
        // Project::create already created a key, delete it to test fresh creation
        let existing = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap()
            .unwrap();
        ApiKey::delete(&db.conn, existing.id).await.unwrap();

        let result = ApiKey::create(&db.conn, key_input.clone()).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.project_id, project_id);
        assert_eq!(created.consumer_key, key_input.consumer_key);
    }

    #[tokio::test]
    async fn test_read_by_id() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let existing = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap()
            .unwrap();

        let result = ApiKey::read_by_id(&db.conn, existing.id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().consumer_key, existing.consumer_key);
    }

    #[tokio::test]
    async fn test_read_by_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let result = ApiKey::read_by_id(&db.conn, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_by_project_id() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let result = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_read_by_project_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let result = ApiKey::read_by_project_id(&db.conn, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_by_consumer_key() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let existing = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap()
            .unwrap();

        let result = ApiKey::read_by_consumer_key(&db.conn, &existing.consumer_key)
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, existing.id);
    }

    #[tokio::test]
    async fn test_read_by_consumer_key_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let result = ApiKey::read_by_consumer_key(&db.conn, "invalid")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_api_key() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let mut key = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap()
            .unwrap();
        let new_consumer_key = "updated_key".to_string();
        key.consumer_key = new_consumer_key.clone();

        let result = ApiKey::update(&db.conn, &key).await.unwrap();
        assert!(result);

        let updated = ApiKey::read_by_id(&db.conn, key.id).await.unwrap().unwrap();
        assert_eq!(updated.consumer_key, new_consumer_key);
    }

    #[tokio::test]
    async fn test_update_api_key_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let key = ApiKey {
            id: 999,
            project_id: 1,
            consumer_key: "k".to_string(),
            consumer_secret: "s".to_string(),
            passkey: "p".to_string(),
        };

        let result = ApiKey::update(&db.conn, &key).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        let db = TestDb::in_memory().await.unwrap();
        let project_id = setup_test_project(&db).await;

        let key = ApiKey::read_by_project_id(&db.conn, project_id)
            .await
            .unwrap()
            .unwrap();

        let result = ApiKey::delete(&db.conn, key.id).await.unwrap();
        assert!(result);

        let verify = ApiKey::read_by_id(&db.conn, key.id).await.unwrap();
        assert!(verify.is_none());
    }

    #[tokio::test]
    async fn test_delete_api_key_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let result = ApiKey::delete(&db.conn, 999).await.unwrap();
        assert!(!result);
    }
}
