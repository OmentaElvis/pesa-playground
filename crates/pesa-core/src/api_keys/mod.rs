use anyhow::Result;
use rand::{distr::Alphanumeric, Rng};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
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
        let consumer_key: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(18)
            .map(char::from)
            .collect();

        let consumer_secret: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        let passkey: String = rand::rng()
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

    pub async fn create(db: &DatabaseConnection, api_key: ApiKey) -> Result<ApiKey> {
        let mut create = db::ActiveModel {
            project_id: Set(api_key.project_id),
            consumer_key: Set(api_key.consumer_key),
            consumer_secret: Set(api_key.consumer_secret),
            passkey: Set(api_key.passkey),
            ..Default::default()
        };

        if api_key.id == 0 {
            create.id = Set(api_key.id);
        }

        Ok(create.insert(db).await?.into())
    }

    pub async fn read_by_id(db: &DatabaseConnection, id: u32) -> Result<Option<ApiKey>> {
        let api = db::Entity::find_by_id(id).one(db).await?;

        Ok(api.map(|a| a.into()))
    }

    pub async fn read_by_project_id(db: &DatabaseConnection, id: u32) -> Result<Option<ApiKey>> {
        let api = db::Entity::find()
            .filter(db::Column::ProjectId.eq(id))
            .one(db)
            .await?;

        Ok(api.map(|a| a.into()))
    }

    pub async fn read_by_consumer_key(
        conn: &DatabaseConnection,
        consumer_key: &str,
    ) -> Result<Option<ApiKey>> {
        let api_key = db::Entity::find()
            .filter(db::Column::ConsumerKey.eq(consumer_key))
            .one(conn)
            .await?;

        Ok(api_key.map(|f| f.into()))
    }

    pub async fn update(conn: &DatabaseConnection, api_key: &ApiKey) -> Result<bool> {
        let api = db::Entity::find_by_id(api_key.id).one(conn).await?;

        if api.is_none() {
            return Ok(false);
        }

        let mut api: db::ActiveModel = api.unwrap().into();
        api.project_id = Set(api_key.project_id);
        api.consumer_key = Set(api_key.consumer_key.clone());
        api.consumer_secret = Set(api_key.consumer_secret.clone());
        api.passkey = Set(api_key.passkey.clone());

        api.update(conn).await?;
        Ok(true)
    }

    pub async fn delete(db: &DatabaseConnection, id: u32) -> Result<bool> {
        let result = db::Entity::delete_by_id(id).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}
