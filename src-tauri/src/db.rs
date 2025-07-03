use std::{env, fs};

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Connection, SqliteConnection, SqlitePool,
};
use tauri::{AppHandle, Manager};

use crate::transaction::TransactionRepository;
use crate::{
    api_keys::{AccessToken, ApiKey},
    api_logs::ApiLogRepository,
    callbacks::CallbackLog,
    project::Project,
    user::User,
};

pub struct Database {
    pub pool: SqlitePool,
}

pub async fn init_database(db: &SqlitePool) -> Result<()> {
    Project::init_table(db).await?;
    TransactionRepository::init_table(db).await?;
    ApiKey::init_table(db).await?;
    ApiLogRepository::init_table(db).await?;
    User::init_table(db).await?;
    AccessToken::init_table(db).await?;
    CallbackLog::init_table(db).await?;

    Ok(())
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

        // Set the DATABASE_URL environment variable to point to this SQLite file
        let connection_url = format!("sqlite://{}", db_path.display());
        env::set_var("DATABASE_URL", &connection_url);

        // From db.rs
        let pool = SqlitePoolOptions::new().connect(&connection_url).await?;
        init_database(&pool).await?;

        Ok(Self { pool })
    }
}
