#![cfg(feature = "cli")]

use clap::Parser;
use pesa_core::{db, AppContext, AppEventManager};
use pesa_lua::ScriptManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{error, info};

const TAURI_APP_ID: &str = "net.omenta.pesaplayground";

/// A dummy event manager for the CLI that does nothing.
struct CliEventManager;
impl AppEventManager for CliEventManager {
    fn emit_all(&self, _event: &str, _payload: serde_json::Value) -> anyhow::Result<()> {
        // In the CLI, we don't need to broadcast events.
        Ok(())
    }
}

/// A lightweight M-Pesa ecosystem simulator for developers.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Lua script to execute
    #[arg(short, long)]
    script: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Setting up data directory...");
    let data_dir = if let Some(mut dir) = dirs::data_dir() {
        dir.push(TAURI_APP_ID);
        dir
    } else {
        // Fallback to current directory
        PathBuf::from(".")
    };

    if !data_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            error!("Failed to create data directory: {}", e);
            // Exit if we can't create the data dir
            return Err(anyhow::anyhow!("Failed to create data directory: {}", e));
        }
    }

    let db_path = data_dir.join("database.sqlite");
    let settings_path = data_dir.join("settings.json");

    info!("Setting up database at {:?}...", db_path);
    let db = Arc::new(db::Database::new(&db_path).await?);
    db.init().await?;

    let settings_manager = pesa_core::settings::SettingsManager::new(settings_path).await?;

    let app_context = AppContext {
        db: db.conn.clone(),
        settings: settings_manager,
        event_manager: Arc::new(CliEventManager),
        running: Arc::new(Mutex::new(HashMap::new())),
    };

    info!("Initializing script manager...");
    let manager = ScriptManager::new(app_context, &data_dir)?;

    info!("Reading script from: {:?}", args.script);
    let script_code = fs::read_to_string(args.script).await?;

    info!("Executing script...");
    let result = manager.lock().await.execute_script(&script_code).await?;

    info!("Script finished.");
    println!("{}", result);

    if manager.lock().await.has_listeners() {
        info!("Event listeners registered. Script host will remain active.");
        // Keep the process alive to listen for events
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    } else {
        info!("No event listeners registered. Exiting.");
    }

    Ok(())
}
