use crate::{app::TransactionManagerStats, sandboxes::RunningSandbox};
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub mod accounts;
pub mod api_keys;
pub mod api_logs;
pub mod app;
pub mod app_metadata;
pub mod business;
pub mod business_operators;
pub mod callbacks;
pub mod db;
pub mod events;
pub mod info;
pub mod migrations;
pub mod projects;
pub mod request_lifecycle;
pub mod sandboxes;
pub mod self_test;
pub mod server;
pub mod settings;
pub mod system;
pub mod transaction_costs;
pub mod transaction_jobs;
pub mod transactions;
pub mod transactions_log;
pub mod utils;

pub use dashmap;

pub trait AppEventManager {
    fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct AppContext {
    pub db: DatabaseConnection,
    pub settings: settings::SettingsManager,
    pub event_manager: Arc<dyn AppEventManager + Send + Sync>,
    pub running: Arc<DashMap<u32, RunningSandbox>>,
    pub app_root: PathBuf,
    pub stats: Arc<RwLock<TransactionManagerStats>>,
    pub transaction_tx: tokio::sync::mpsc::Sender<transaction_jobs::task::ScheduledTransactionTask>,
    pub request_lifecycle_manager: Arc<request_lifecycle::RequestLifecycleManager>,
}
