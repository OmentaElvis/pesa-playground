use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tempfile::TempDir;

use crate::migrations::Migrator;
use crate::transaction_costs;

pub struct TestDb {
    pub conn: DatabaseConnection,
    _temp_dir: TempDir,
}

impl TestDb {
    pub async fn new() -> Result<Self, DbErr> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let connection_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let mut opt = ConnectOptions::new(connection_url);
        opt.max_connections(1000)
            .connect_timeout(std::time::Duration::from_secs(10));

        let conn = Database::connect(opt).await?;

        Migrator::up(&conn, None).await?;
        transaction_costs::init_default_costs(&conn).await?;

        Ok(Self {
            conn,
            _temp_dir: temp_dir,
        })
    }

    pub async fn in_memory() -> Result<Self, DbErr> {
        let mut opt = ConnectOptions::from("sqlite::memory:");
        opt.max_connections(1000)
            .connect_timeout(std::time::Duration::from_secs(10));

        let conn = Database::connect(opt).await?;

        Migrator::up(&conn, None).await?;
        transaction_costs::init_default_costs(&conn).await?;

        Ok(Self {
            conn,
            _temp_dir: TempDir::new().unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_db_initializes() {
        let db = TestDb::in_memory().await;
        assert!(db.is_ok());
    }
}
