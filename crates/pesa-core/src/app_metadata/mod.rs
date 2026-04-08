pub mod db;

use sea_orm::{IntoActiveModel, entity::prelude::*};

pub async fn get_metadata<C>(conn: &C, key: &str) -> Result<Option<String>, DbErr>
where
    C: ConnectionTrait,
{
    let result = db::Entity::find_by_id(key).one(conn).await?;
    Ok(result.map(|r| r.value))
}

pub async fn set_metadata<C>(conn: &C, key: &str, value: &str) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let existing = db::Entity::find_by_id(key).one(conn).await;

    match existing {
        Ok(Some(model)) => {
            let mut model = model.into_active_model();
            model.value = sea_orm::ActiveValue::Set(value.to_string());
            model.update(conn).await?;
        }
        _ => {
            let model = db::ActiveModel {
                key: sea_orm::ActiveValue::Set(key.to_string()),
                value: sea_orm::ActiveValue::Set(value.to_string()),
            };
            let _ = model.insert(conn).await;
        }
    }

    Ok(())
}

pub async fn table_exists<C>(conn: &C) -> bool
where
    C: ConnectionTrait,
{
    db::Entity::find().count(conn).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestDb;

    #[tokio::test]
    async fn test_table_exists() {
        let db = TestDb::in_memory().await.unwrap();

        let exists = table_exists(&db.conn).await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_set_and_get_metadata() {
        let db = TestDb::in_memory().await.unwrap();

        let key = "version";
        let value = "1.0.0";

        set_metadata(&db.conn, key, value).await.unwrap();

        let retrieved = get_metadata(&db.conn, key).await.unwrap();
        assert_eq!(retrieved, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_update_metadata() {
        let db = TestDb::in_memory().await.unwrap();

        let key = "theme";
        set_metadata(&db.conn, key, "light").await.unwrap();
        set_metadata(&db.conn, key, "dark").await.unwrap();

        let retrieved = get_metadata(&db.conn, key).await.unwrap();
        assert_eq!(retrieved, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_metadata() {
        let db = TestDb::in_memory().await.unwrap();

        let retrieved = get_metadata(&db.conn, "nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }
}
