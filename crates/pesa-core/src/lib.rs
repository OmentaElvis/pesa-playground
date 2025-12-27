use std::{collections::HashMap, sync::Arc};

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use crate::sandboxes::RunningSandbox;

pub mod accounts;
pub mod api_keys;
pub mod api_logs;
pub mod business;
pub mod business_operators;
pub mod callbacks;
pub mod db;
pub mod events;
pub mod info;
pub mod projects;
pub mod sandboxes;
pub mod server;
pub mod settings;
pub mod system;
pub mod transaction_costs;
pub mod transactions;
pub mod transactions_log;

pub trait AppEventManager {
    fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct AppContext {
    pub db: DatabaseConnection,
    pub settings: settings::SettingsManager,
    pub event_manager: Arc<dyn AppEventManager + Send + Sync>,
    pub running: Arc<Mutex<HashMap<u32, RunningSandbox>>>,
}
