use std::{env, fs};

use anyhow::Context;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Schema};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};
use tauri::{AppHandle, Manager};

use crate::{
    accounts, api_keys, api_logs, business, callbacks, projects,
    server::{self},
    transaction_costs, transactions, transactions_log,
};

pub struct Database {
    pub conn: DatabaseConnection,
    pub handle: AppHandle,
}

impl Database {
    pub async fn new(app_handle: &AppHandle) -> anyhow::Result<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app dir");

        // Ensure the app directory exists
        fs::create_dir_all(&app_dir)?;

        let db_path = app_dir.join("database.sqlite");

        if !db_path.exists() {
            let options = SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true);
            SqliteConnection::connect_with(&options)
                .await
                .expect("Failed to create database");
        }
        let connection_url = format!("sqlite://{}?mode=rwc", db_path.display());
        env::set_var("DATABASE_URL", &connection_url);

        let db: DatabaseConnection = sea_orm::Database::connect(connection_url).await?;

        Ok(Self {
            conn: db,
            handle: app_handle.clone(),
        })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        let schema = Schema::new(DbBackend::Sqlite);
        let db = &self.conn;

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
}
