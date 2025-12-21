use std::{env, fs, path::Path};

use anyhow::Context;
use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseConnection, DbBackend, Schema};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};

use crate::{
    accounts, api_keys, api_logs, business, callbacks, projects,
    server::{self},
    transaction_costs, transactions, transactions_log,
};

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
        env::set_var("DATABASE_URL", &connection_url);

        let mut opt = ConnectOptions::new(connection_url);
        opt.sqlx_logging(false);

        let db: DatabaseConnection = sea_orm::Database::connect(opt).await?;

        Ok(Self { conn: db })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        run_migrations(&self.conn).await
    }
}

pub async fn run_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    let schema = Schema::new(DbBackend::Sqlite);

    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create accounts table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::paybill_accounts::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create paybill_accounts table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::till_accounts::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create till_accounts table")?;

    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::utility_accounts::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create utility_accounts table")?;

    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::mmf_accounts::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create mmf_accounts table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(accounts::user_profiles::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create user_profiles table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(api_keys::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create api_keys table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(api_logs::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create api_logs table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(business::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create business table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(callbacks::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create callback_logs table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(projects::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create projects table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(server::access_token::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create access_tokens table")?;
    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(transactions::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create transactions table")?;

    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(transactions_log::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create transactions_log table")?;

    db.execute(
        db.get_database_backend().build(
            schema
                .create_table_from_entity(transaction_costs::db::Entity)
                .if_not_exists(),
        ),
    )
    .await
    .context("Failed to create transactions_cost table")?;

    transaction_costs::init_default_costs(db).await?;

    Ok(())
}
