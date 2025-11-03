use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

pub mod db;

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token: String,
    pub project_id: u32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<db::Model> for AccessToken {
    fn from(value: db::Model) -> Self {
        AccessToken {
            token: value.token,
            project_id: value.project_id,
            expires_at: value.expires_at,
            created_at: value.created_at,
        }
    }
}

impl AccessToken {
    pub async fn create(
        conn: &DatabaseConnection,
        access_token: AccessToken,
    ) -> Result<AccessToken> {
        let create = db::ActiveModel {
            token: Set(access_token.token),
            project_id: Set(access_token.project_id),
            expires_at: Set(access_token.expires_at),
            ..Default::default()
        };

        Ok(create.insert(conn).await?.into())
    }

    pub async fn read_by_token(
        conn: &DatabaseConnection,
        token: &str,
    ) -> Result<Option<AccessToken>> {
        let token = db::Entity::find_by_id(token).one(conn).await?;

        Ok(token.map(|t| t.into()))
    }

    pub async fn update(conn: &DatabaseConnection, access_token: &AccessToken) -> Result<bool> {
        let token = db::Entity::find_by_id(access_token.token.to_string())
            .one(conn)
            .await?;
        if token.is_none() {
            return Ok(false);
        }

        let mut token: db::ActiveModel = token.unwrap().into();
        token.project_id = Set(access_token.project_id);
        token.expires_at = Set(access_token.expires_at);

        token.update(conn).await?;
        Ok(true)
    }

    pub async fn delete(comm: &DatabaseConnection, token: String) -> Result<bool> {
        let result = db::Entity::delete_by_id(token).exec(comm).await?;
        Ok(result.rows_affected > 0)
    }
}
