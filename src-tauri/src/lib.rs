use std::{collections::HashMap, sync::Arc};

use serde_json::{json, Value};
use server::start_project_server;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};
use tokio::sync::{oneshot, Mutex};

mod api_keys;
mod api_logs;
mod callbacks;
mod db;
mod project;
mod server;
mod transaction;
mod user;

pub struct RunningSandbox {
    pub port: u16,
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub handle: tokio::task::JoinHandle<anyhow::Result<()>>,
}

pub struct SandboxManager {
    pub handle: AppHandle,
    pub pool: SqlitePool,
    pub running: Arc<Mutex<HashMap<i64, RunningSandbox>>>,
}

#[tauri::command]
async fn start_sandbox(
    state: State<'_, SandboxManager>,
    project_id: i64,
) -> Result<String, String> {
    let port: u16 = (8000 + (project_id % 1000))
        .try_into()
        .map_err(|_| "Failed to create port".to_string())?;
    let addr = format!("127.0.0.1:{}", port);

    let mut running = state.running.lock().await;
    if running.contains_key(&project_id) {
        return Ok(format!("http://{}", addr));
    }
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let handle = tokio::spawn(start_project_server(
        project_id,
        port,
        state.pool.clone(),
        shutdown_rx,
        state.handle.clone(),
    ));
    running.insert(
        project_id,
        RunningSandbox {
            shutdown: shutdown_tx,
            handle,
            port,
        },
    );

    Ok(format!("http://{}", addr))
}

#[tauri::command]
async fn stop_sandbox(state: State<'_, SandboxManager>, project_id: i64) -> Result<(), String> {
    let mut running = state.running.lock().await;
    let rs = if let Some(rs) = running.remove(&project_id) {
        rs
    } else {
        return Ok(());
    };

    if !rs.handle.is_finished() {
        rs.shutdown
            .send(())
            .map_err(|_| "Failed to send shutdown signal".to_string())?;
    }

    Ok(())
}
#[tauri::command]
async fn sandbox_status(
    state: State<'_, SandboxManager>,
    project_id: i64,
) -> Result<Value, String> {
    let mut running = state.running.lock().await;
    if let Some(rs) = running.get(&project_id) {
        if !rs.handle.is_finished() {
            return Ok(json! ({
                "status": "on",
                "port": rs.port
            }));
        }
    } else {
        return Ok(json!({
            "status": "off",
            "port": 0
        }));
    }

    let rs = running.remove(&project_id).unwrap();

    match rs.handle.await {
        Err(err) => Ok(json! ({
            "status": "error",
            "port": rs.port,
            "error": err.to_string()
        })),
        Ok(res) => match res {
            Ok(()) => Ok(json!({
                "status": "off",
                "port": 0
            })),
            Err(err) => Ok(json! ({
                "status": "error",
                "port": rs.port,
                "error": err.to_string()
            })),
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(move |app| {
            tauri::async_runtime::block_on(async move {
                let handle = app.handle();
                let database = db::Database::new(handle)
                    .await
                    .expect("Failed to initialize database");

                app.manage(SandboxManager {
                    pool: database.pool.clone(),
                    running: Arc::new(Mutex::new(HashMap::new())),
                    handle: app.handle().clone(),
                });
                app.manage(database);
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_sandbox,
            stop_sandbox,
            sandbox_status,
            project::create_project,
            project::get_project,
            project::get_projects,
            project::update_project,
            project::delete_project,
            // users
            user::get_users,
            user::create_user,
            user::remove_user,
            user::get_user,
            user::update_user,
            user::generate_user,
            // transactions
            transaction::create_transaction,
            transaction::get_transaction,
            transaction::delete_transaction,
            transaction::list_transactions,
            transaction::count_transactions,
            transaction::get_transaction_by_checkout_request,
            transaction::get_user_transactions,
            transaction::get_project_transactions,
            transaction::get_recent_transactions,
            transaction::get_transaction_stats,
            transaction::update_transaction,
            // logs
            api_logs::create_api_log,
            api_logs::get_api_log,
            api_logs::update_api_log,
            api_logs::delete_api_log,
            api_logs::list_api_logs,
            api_logs::count_api_logs,
            api_logs::get_project_api_logs,
            api_logs::get_api_logs_by_method,
            api_logs::get_error_api_logs,
            api_logs::get_recent_api_logs,
            api_logs::cleanup_old_api_logs,
            api_logs::get_api_log_stats,
            // Callbacks
            callbacks::resolve_stk_prompt
        ]);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
