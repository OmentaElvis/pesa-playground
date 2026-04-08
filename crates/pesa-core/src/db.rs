use std::{env, fs, path::Path, path::PathBuf};

use anyhow::Context;
use sea_orm::{ConnectOptions, DatabaseConnection, TransactionTrait};
use sea_orm_migration::MigratorTrait;
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};

use crate::migrations::Migrator;
use crate::transaction_costs;
use semver::Version;

const SCHEMA_VERSION_KEY: &str = "schema_version";

pub struct Database {
    pub conn: DatabaseConnection,
    db_path: PathBuf,
}

impl Database {
    pub async fn new(db_path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !db_path.exists() {
            let options = SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true);
            SqliteConnection::connect_with(&options)
                .await
                .expect("Failed to create database");
        }
        let connection_url = format!("sqlite://{}?mode=rwc", db_path.display());
        unsafe { env::set_var("DATABASE_URL", &connection_url) };

        let mut opt = ConnectOptions::new(connection_url);
        opt.sqlx_logging(false);
        opt.max_connections(1000);

        let db: DatabaseConnection = sea_orm::Database::connect(opt).await?;

        Ok(Self {
            conn: db,
            db_path: db_path.to_path_buf(),
        })
    }

    pub async fn init(&mut self) -> anyhow::Result<Option<PathBuf>> {
        self.run_migrations().await?;

        let table_exists = crate::app_metadata::table_exists(&self.conn).await;

        if !table_exists {
            self.set_metadata(SCHEMA_VERSION_KEY, env!("CARGO_PKG_VERSION"))
                .await?;
            return Ok(None);
        }

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let current_major = current_version.major as u32;

        let stored_version = match self.get_metadata(SCHEMA_VERSION_KEY).await {
            Ok(Some(v)) => v,
            _ => {
                self.set_metadata(SCHEMA_VERSION_KEY, env!("CARGO_PKG_VERSION"))
                    .await?;
                return Ok(None);
            }
        };

        let stored_major = Version::parse(&stored_version).map(|v| v.major as u32);

        match stored_major {
            Ok(stored) if stored == current_major => {
                self.set_metadata(SCHEMA_VERSION_KEY, env!("CARGO_PKG_VERSION"))
                    .await?;
                Ok(None)
            }
            Ok(_) => {
                let backup_path = self.backup_and_reset().await?;
                self.run_migrations().await?;
                self.set_metadata(SCHEMA_VERSION_KEY, env!("CARGO_PKG_VERSION"))
                    .await?;
                Ok(Some(backup_path))
            }
            Err(_) => {
                self.set_metadata(SCHEMA_VERSION_KEY, env!("CARGO_PKG_VERSION"))
                    .await?;
                Ok(None)
            }
        }
    }

    async fn run_migrations(&mut self) -> anyhow::Result<()> {
        let txn = self
            .conn
            .begin()
            .await
            .context("Failed to start migration transaction")?;
        Migrator::up(&txn, None).await?;
        transaction_costs::init_default_costs(&txn).await?;
        txn.commit().await?;
        Ok(())
    }

    async fn get_metadata(&mut self, key: &str) -> anyhow::Result<Option<String>> {
        let result = crate::app_metadata::get_metadata(&self.conn, key).await?;
        Ok(result)
    }

    async fn set_metadata(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        crate::app_metadata::set_metadata(&self.conn, key, value).await?;
        Ok(())
    }

    async fn backup_and_reset(&mut self) -> anyhow::Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_name = format!("data.db.backup.{}", timestamp);
        let backup_path = self.db_path.with_file_name(backup_name);

        fs::rename(&self.db_path, &backup_path).context("Failed to backup old database")?;

        let options = SqliteConnectOptions::new()
            .filename(&self.db_path)
            .create_if_missing(true);
        SqliteConnection::connect_with(&options)
            .await
            .context("Failed to create fresh database")?;

        let connection_url = format!("sqlite://{}?mode=rwc", self.db_path.display());
        unsafe { env::set_var("DATABASE_URL", &connection_url) };

        let mut opt = ConnectOptions::new(connection_url);
        opt.sqlx_logging(false);

        self.conn = sea_orm::Database::connect(opt)
            .await
            .context("Failed to connect to fresh database")?;

        Ok(backup_path)
    }
}

#[cfg(test)]
mod version_tests {
    use semver::Version;

    fn extract_major(version: &str) -> u64 {
        Version::parse(version).unwrap().major
    }

    #[test]
    fn test_same_major_version_compatible() {
        assert_eq!(extract_major("1.4.1"), extract_major("1.4.2"));
        assert_eq!(extract_major("1.4.1"), extract_major("1.5.0"));
        assert_eq!(extract_major("1.4.1"), extract_major("1.9.99"));
    }

    #[test]
    fn test_different_major_version_incompatible() {
        assert_ne!(extract_major("1.4.1"), extract_major("2.0.0"));
        assert_ne!(extract_major("1.4.1"), extract_major("0.9.0"));
        assert_ne!(extract_major("2.0.0"), extract_major("3.0.0"));
    }

    #[test]
    fn test_version_parsing() {
        assert_eq!(extract_major("1.0.0"), 1);
        assert_eq!(extract_major("2.1.0"), 2);
        assert_eq!(extract_major("10.20.30"), 10);
        assert_eq!(extract_major("0.1.0"), 0);
    }
}

#[cfg(test)]
mod init_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_fresh_db() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        (temp_dir, db_path)
    }

    #[tokio::test]
    async fn test_fresh_database_initializes_successfully() {
        let (_temp_dir, db_path) = setup_fresh_db();

        let mut db = Database::new(&db_path).await.unwrap();
        let result = db.init().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        let version = crate::app_metadata::get_metadata(&db.conn, "schema_version")
            .await
            .unwrap();
        assert!(version.is_some());
    }

    #[tokio::test]
    async fn test_same_major_version_no_reset() {
        let (_temp_dir, db_path) = setup_fresh_db();

        let mut db = Database::new(&db_path).await.unwrap();
        db.init().await.unwrap();

        let version = crate::app_metadata::get_metadata(&db.conn, "schema_version")
            .await
            .unwrap();
        assert!(version.is_some());
    }

    #[tokio::test]
    async fn test_incompatible_major_version_triggers_reset() {
        let (_temp_dir, db_path) = setup_fresh_db();

        let mut db = Database::new(&db_path).await.unwrap();
        db.init().await.unwrap();

        crate::app_metadata::set_metadata(&db.conn, "schema_version", "0.0.1")
            .await
            .unwrap();

        let result = db.init().await.unwrap();

        assert!(result.is_some());

        let backup_path = result.unwrap();
        assert!(backup_path.exists());

        let version = crate::app_metadata::get_metadata(&db.conn, "schema_version")
            .await
            .unwrap();
        assert!(version.is_some());
    }

    #[tokio::test]
    async fn test_connection_works_after_reset() {
        let (_temp_dir, db_path) = setup_fresh_db();

        let mut db = Database::new(&db_path).await.unwrap();
        db.init().await.unwrap();

        crate::app_metadata::set_metadata(&db.conn, "test_key", "before_reset")
            .await
            .unwrap();

        let initial_value = crate::app_metadata::get_metadata(&db.conn, "test_key")
            .await
            .unwrap();
        assert_eq!(initial_value, Some("before_reset".to_string()));

        crate::app_metadata::set_metadata(&db.conn, "schema_version", "0.0.1")
            .await
            .unwrap();

        db.init().await.unwrap();

        let after_value = crate::app_metadata::get_metadata(&db.conn, "test_key")
            .await
            .unwrap();
        assert!(after_value.is_none());

        let new_version = crate::app_metadata::get_metadata(&db.conn, "schema_version")
            .await
            .unwrap();
        assert!(new_version.is_some());
    }

    #[tokio::test]
    async fn test_minor_version_difference_no_reset() {
        let (_temp_dir, db_path) = setup_fresh_db();

        let mut db = Database::new(&db_path).await.unwrap();
        db.init().await.unwrap();

        crate::app_metadata::set_metadata(&db.conn, "schema_version", "1.3.0")
            .await
            .unwrap();

        let result = db.init().await.unwrap();

        assert!(result.is_none());
        assert!(db_path.exists());
    }
}
