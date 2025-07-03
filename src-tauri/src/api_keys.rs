use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{distr::Alphanumeric, Rng};
use sea_query::{ColumnDef, Expr, Iden, Query, SqliteQueryBuilder, Table};
use serde::Serialize;
// Added Expr import
use sqlx::{sqlite::SqliteRow, Executor, FromRow, Row, SqlitePool};

#[derive(Iden)]
pub enum ApiKeys {
    Table,
    Id,
    ProjectId,
    ConsumerKey,
    ConsumerSecret,
    PassKey,
}

#[derive(Iden)]
pub enum AccessTokens {
    Table,
    Token,
    ProjectId,
    ExpiresAt,
    CreatedAt,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct ApiKey {
    pub id: i64,
    pub project_id: i64,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
}

impl ApiKey {
    pub fn generate(project_id: i64) -> ApiKey {
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

    pub async fn init_table(db: &SqlitePool) -> Result<()> {
        let sql = {
            Table::create()
                .table(ApiKeys::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(ApiKeys::Id)
                        .not_null()
                        .integer()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(ApiKeys::ProjectId).not_null().integer())
                .col(
                    ColumnDef::new(ApiKeys::ConsumerKey)
                        .text()
                        .unique_key()
                        .not_null(),
                )
                .col(ColumnDef::new(ApiKeys::ConsumerSecret).text().not_null())
                .col(ColumnDef::new(ApiKeys::PassKey).text().not_null())
                .to_string(SqliteQueryBuilder)
        };
        db.execute(sql.as_str()).await?;
        Ok(())
    }

    pub async fn create(db: &SqlitePool, api_key: ApiKey) -> Result<ApiKey> {
        let sql = {
            Query::insert()
                .into_table(ApiKeys::Table)
                .columns([
                    ApiKeys::ProjectId,
                    ApiKeys::ConsumerKey,
                    ApiKeys::ConsumerSecret,
                    ApiKeys::PassKey,
                ])
                .values_panic([
                    api_key.project_id.into(),
                    api_key.consumer_key.into(),
                    api_key.consumer_secret.into(),
                    api_key.passkey.into(),
                ])
                .returning_all()
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_one(sql.as_str()).await?;
        Self::from_row(&row)
    }

    pub async fn read_by_id(db: &SqlitePool, id: i64) -> Result<Option<ApiKey>> {
        let sql = {
            Query::select()
                .columns([
                    ApiKeys::Id,
                    ApiKeys::ProjectId,
                    ApiKeys::ConsumerKey,
                    ApiKeys::ConsumerSecret,
                    ApiKeys::PassKey,
                ])
                .from(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };
        let row = db.fetch_optional(sql.as_str()).await?;
        let api_key = if let Some(row) = row {
            Some(Self::from_row(&row)?)
        } else {
            None
        };

        Ok(api_key)
    }
    pub async fn read_by_project_id(db: &SqlitePool, id: i64) -> Result<Option<ApiKey>> {
        let sql = {
            Query::select()
                .columns([
                    ApiKeys::Id,
                    ApiKeys::ProjectId,
                    ApiKeys::ConsumerKey,
                    ApiKeys::ConsumerSecret,
                    ApiKeys::PassKey,
                ])
                .from(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::ProjectId).eq(id))
                .to_string(SqliteQueryBuilder)
        };
        let row = db.fetch_optional(sql.as_str()).await?;
        let api_key = if let Some(row) = row {
            Some(Self::from_row(&row)?)
        } else {
            None
        };

        Ok(api_key)
    }

    pub async fn read_by_consumer_key(
        db: &SqlitePool,
        consumer_key: &str,
    ) -> Result<Option<ApiKey>> {
        let sql = {
            Query::select()
                .columns([
                    ApiKeys::Id,
                    ApiKeys::ProjectId,
                    ApiKeys::ConsumerKey,
                    ApiKeys::ConsumerSecret,
                    ApiKeys::PassKey,
                ])
                .from(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::ConsumerKey).eq(consumer_key))
                .to_string(SqliteQueryBuilder)
        };
        let row = db.fetch_optional(sql.as_str()).await?;
        let api_key = if let Some(row) = row {
            Some(Self::from_row(&row)?)
        } else {
            None
        };

        Ok(api_key)
    }

    pub async fn update(db: &SqlitePool, api_key: &ApiKey) -> Result<bool> {
        let sql = {
            Query::update()
                .table(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::Id).eq(api_key.id))
                .value(ApiKeys::ProjectId, api_key.project_id)
                .value(ApiKeys::ConsumerKey, api_key.consumer_key.clone())
                .value(ApiKeys::ConsumerSecret, api_key.consumer_secret.clone())
                .value(ApiKeys::PassKey, api_key.passkey.clone())
                .to_string(SqliteQueryBuilder)
        };

        let result = db.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(db: &SqlitePool, id: i64) -> Result<bool> {
        let sql = {
            Query::delete()
                .from_table(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::Id).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        let result = db.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    pub fn from_row(row: &SqliteRow) -> Result<ApiKey> {
        Ok(ApiKey {
            id: row.try_get(ApiKeys::Id.to_string().as_str())?,
            project_id: row.try_get(ApiKeys::ProjectId.to_string().as_str())?,
            consumer_key: row.try_get(ApiKeys::ConsumerKey.to_string().as_str())?,
            consumer_secret: row.try_get(ApiKeys::ConsumerSecret.to_string().as_str())?,
            passkey: row.try_get(ApiKeys::PassKey.to_string().as_str())?,
        })
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AccessToken {
    pub token: String,
    pub project_id: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AccessToken {
    pub async fn init_table(db: &SqlitePool) -> Result<()> {
        let access_tokens = {
            Table::create()
                .table(AccessTokens::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AccessTokens::Token)
                        .text()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(AccessTokens::ProjectId).not_null().integer())
                .col(ColumnDef::new(AccessTokens::ExpiresAt).integer())
                .col(
                    ColumnDef::new(AccessTokens::CreatedAt)
                        .integer()
                        .default(Expr::cust("(strftime('%s', 'now'))")),
                )
                .to_string(SqliteQueryBuilder)
        };
        db.execute(access_tokens.as_str()).await?;
        Ok(())
    }

    pub async fn create(db: &SqlitePool, access_token: AccessToken) -> Result<AccessToken> {
        let sql = {
            Query::insert()
                .into_table(AccessTokens::Table)
                .columns([
                    AccessTokens::ProjectId,
                    AccessTokens::Token,
                    AccessTokens::ExpiresAt,
                ])
                .values_panic([
                    access_token.project_id.into(),
                    access_token.token.into(),
                    access_token.expires_at.timestamp().into(),
                ])
                .returning_all()
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_one(sql.as_str()).await?;
        Self::from_row(&row)
    }

    pub async fn read_by_token(db: &SqlitePool, token: &str) -> Result<Option<AccessToken>> {
        let sql = {
            Query::select()
                .columns([
                    AccessTokens::Token,
                    AccessTokens::ProjectId,
                    AccessTokens::ExpiresAt,
                    AccessTokens::CreatedAt,
                ])
                .from(AccessTokens::Table)
                .and_where(Expr::col(AccessTokens::Token).eq(token))
                .to_string(SqliteQueryBuilder)
        };
        let row = db.fetch_optional(sql.as_str()).await?;
        let api_key = if let Some(row) = row {
            Some(Self::from_row(&row)?)
        } else {
            None
        };

        Ok(api_key)
    }

    pub async fn update(db: &SqlitePool, access_token: &AccessToken) -> Result<bool> {
        let sql = {
            Query::update()
                .table(AccessTokens::Table)
                .and_where(Expr::col(AccessTokens::Token).eq(access_token.token.clone()))
                .value(AccessTokens::ProjectId, access_token.project_id)
                .value(
                    AccessTokens::ExpiresAt,
                    access_token.expires_at.timestamp().clone(),
                )
                .to_string(SqliteQueryBuilder)
        };

        let result = db.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(db: &SqlitePool, token: String) -> Result<bool> {
        let sql = {
            Query::delete()
                .from_table(AccessTokens::Table)
                .and_where(Expr::col(AccessTokens::Token).eq(token))
                .to_string(SqliteQueryBuilder)
        };

        let result = db.execute(sql.as_str()).await?;
        Ok(result.rows_affected() > 0)
    }

    pub fn from_row(row: &SqliteRow) -> Result<AccessToken> {
        let expires_at = DateTime::from_timestamp(
            row.try_get(AccessTokens::ExpiresAt.to_string().as_str())?,
            0,
        )
        .unwrap_or(DateTime::UNIX_EPOCH);
        let created_at = DateTime::from_timestamp(
            row.try_get(AccessTokens::CreatedAt.to_string().as_str())?,
            0,
        )
        .unwrap_or(DateTime::UNIX_EPOCH);
        Ok(AccessToken {
            token: row.try_get(AccessTokens::Token.to_string().as_str())?,
            project_id: row.try_get(AccessTokens::ProjectId.to_string().as_str())?,
            expires_at,
            created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    // Helper function to set up an in-memory SQLite database for testing
    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite database");
        ApiKey::init_table(&pool)
            .await
            .expect("Failed to initialize table");
        pool
    }

    #[tokio::test]
    async fn test_api_key_crud() -> Result<()> {
        let db = setup_db().await;

        // 1. Create an API Key
        let new_api_key = ApiKey::generate(1);
        let created_api_key = ApiKey::create(&db, new_api_key.clone()).await?;
        assert_ne!(created_api_key.id, 0); // ID should be assigned by DB
        assert_eq!(created_api_key.project_id, 1);
        assert_eq!(created_api_key.consumer_key.len(), 18);
        assert_eq!(created_api_key.consumer_secret.len(), 40);
        assert_eq!(created_api_key.passkey.len(), 64);

        // 2. Read by ID
        let fetched_api_key = ApiKey::read_by_id(&db, created_api_key.id).await?;
        assert!(fetched_api_key.is_some());
        let fetched_api_key = fetched_api_key.unwrap();
        assert_eq!(fetched_api_key.id, created_api_key.id);
        assert_eq!(fetched_api_key.consumer_key, created_api_key.consumer_key);

        // 3. Read by Consumer Key
        let fetched_by_consumer_key =
            ApiKey::read_by_consumer_key(&db, &created_api_key.consumer_key).await?;
        assert!(fetched_by_consumer_key.is_some());
        assert_eq!(fetched_by_consumer_key.unwrap().id, created_api_key.id);

        // 4. Update API Key
        let original_consumer_secret = created_api_key.consumer_secret.clone();
        let mut updated_api_key = created_api_key.clone();
        updated_api_key.consumer_secret =
            "new_secret_updated_123456789012345678901234567890".to_string();
        updated_api_key.project_id = 2;

        let updated = ApiKey::update(&db, &updated_api_key).await?;
        assert!(updated);

        let re_fetched_api_key = ApiKey::read_by_id(&db, updated_api_key.id).await?;
        assert!(re_fetched_api_key.is_some());
        let re_fetched_api_key = re_fetched_api_key.unwrap();
        assert_eq!(
            re_fetched_api_key.consumer_secret,
            updated_api_key.consumer_secret
        );
        assert_ne!(re_fetched_api_key.consumer_secret, original_consumer_secret);
        assert_eq!(re_fetched_api_key.project_id, 2);

        // 5. Delete API Key
        let deleted = ApiKey::delete(&db, updated_api_key.id).await?;
        assert!(deleted);

        let deleted_api_key_check = ApiKey::read_by_id(&db, updated_api_key.id).await?;
        assert!(deleted_api_key_check.is_none());

        Ok(())
    }
}
