use std::{env, fs, path::Path};

use anyhow::Context;
use sea_orm::{ConnectOptions, DatabaseConnection, TransactionTrait};
use sea_orm_migration::MigratorTrait;
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};

use crate::migrations::Migrator;
use crate::transaction_costs;

pub struct Database {
    pub conn: DatabaseConnection,
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

        let db: DatabaseConnection = sea_orm::Database::connect(opt).await?;

        Ok(Self { conn: db })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
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
}
