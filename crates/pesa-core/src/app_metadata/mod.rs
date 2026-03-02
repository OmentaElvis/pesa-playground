pub mod db;

use sea_orm::{entity::prelude::*, IntoActiveModel};

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
    use sea_orm::Database;
    use sea_orm::ConnectOptions;

    async fn test_db() -> sea_orm::DatabaseConnection {
        let mut opts = ConnectOptions::from("sqlite::memory:");
        opts.max_connections(1).connect_timeout(std::time::Duration::from_secs(30));
        Database::connect(opts).await.unwrap()
    }

    #[tokio::test]
    async fn test_table_does_not_exist() {
        let conn = test_db().await;
        
        let exists = super::table_exists(&conn).await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_get_metadata_when_table_does_not_exist() {
        let conn = test_db().await;
        
        let result = super::get_metadata(&conn, "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_metadata_when_table_does_not_exist() {
        let conn = test_db().await;
        
        let result = super::set_metadata(&conn, "test_key", "test_value").await;
        assert!(result.is_ok());
    }
}
