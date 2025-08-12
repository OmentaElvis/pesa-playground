use std::{collections::HashMap, sync::Arc};

use once_cell::sync::OnceCell;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::{json, Value};
use server::start_project_server;
use tauri::{AppHandle, Manager, State};
use tokio::sync::{oneshot, Mutex};

mod accounts;
mod api_keys;
mod api_logs;
mod business;
mod callbacks;
mod db;
mod projects;
mod server;
mod transaction_costs;
mod transactions;
mod transactions_log;
mod user;

pub static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
pub fn init_app_handle(app_handle: AppHandle) {
    GLOBAL_APP_HANDLE
        .set(app_handle)
        .expect("Failed to initialize app handle.");
}

pub fn get_app_handle() -> &'static AppHandle {
    GLOBAL_APP_HANDLE.get().expect("AppHandle not initialized.")
}

pub struct RunningSandbox {
    pub port: u16,
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub handle: tokio::task::JoinHandle<anyhow::Result<()>>,
}

pub struct SandboxManager {
    pub handle: AppHandle,
    pub conn: DatabaseConnection,
    pub running: Arc<Mutex<HashMap<u32, RunningSandbox>>>,
}

#[tauri::command]
async fn start_sandbox(
    state: State<'_, SandboxManager>,
    project_id: u32,
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
        state.conn.clone(),
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
async fn stop_sandbox(state: State<'_, SandboxManager>, project_id: u32) -> Result<(), String> {
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
    project_id: u32,
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
#[derive(Serialize)]
struct Status {
    project_id: u32,
    port: u16,
    error: Option<String>,
    status: String,
}

#[tauri::command]
async fn list_running_sandboxes(state: State<'_, SandboxManager>) -> Result<Vec<Status>, String> {
    let mut running = state.running.lock().await;

    let mut instances: Vec<Status> = Vec::new();
    for (project_id, rs) in running.iter_mut() {
        let mut status = Status {
            project_id: *project_id,
            port: rs.port,
            error: None,
            status: "on".to_string(),
        };

        if rs.handle.is_finished() {
            status.port = 0;
            status.status = "off".to_string();
        };

        instances.push(status);
    }

    Ok(instances)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(move |app| {
            tauri::async_runtime::block_on(async move {
                let handle = app.handle();
                init_app_handle(handle.clone());
                let database = db::Database::new(handle)
                    .await
                    .expect("Failed to initialize database");

                if let Err(err) = database.init().await {
                    eprintln!("Database error: {:?}", err);
                }

                app.manage(SandboxManager {
                    conn: database.conn.clone(),
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
            list_running_sandboxes,
            projects::ui::create_project,
            projects::ui::get_project,
            projects::ui::get_projects,
            projects::ui::update_project,
            projects::ui::delete_project,
            projects::ui::get_projects_by_business_id,
            // businesses
            business::ui::create_business,
            business::ui::get_business,
            business::ui::get_businesses,
            business::ui::update_business,
            business::ui::delete_business,
            // accounts
            accounts::ui::get_account,
            accounts::ui::create_account,
            accounts::paybill_accounts::ui::create_paybill_account,
            accounts::paybill_accounts::ui::get_paybill_account,
            accounts::paybill_accounts::ui::get_paybill_accounts,
            accounts::paybill_accounts::ui::get_paybill_accounts_by_business_id,
            accounts::paybill_accounts::ui::update_paybill_account,
            accounts::paybill_accounts::ui::delete_paybill_account,
            accounts::till_accounts::ui::create_till_account,
            accounts::till_accounts::ui::get_till_account,
            accounts::till_accounts::ui::get_till_accounts,
            accounts::till_accounts::ui::get_till_accounts_by_business_id,
            accounts::till_accounts::ui::update_till_account,
            accounts::till_accounts::ui::delete_till_account,
            // users
            accounts::user_profiles::ui::get_users,
            accounts::user_profiles::ui::create_user,
            accounts::user_profiles::ui::remove_user,
            accounts::user_profiles::ui::get_user,
            accounts::user_profiles::ui::update_user,
            accounts::user_profiles::ui::generate_user,
            accounts::user_profiles::ui::generate_users,
            accounts::user_profiles::ui::get_user_by_phone,
            // transactions
            transactions::ui::transfer,
            transactions::ui::reverse,
            transactions::ui::get_transaction,
            transactions::ui::list_transactions,
            transactions::ui::count_transactions,
            transactions::ui::get_transaction_by_checkout_request,
            transactions::ui::get_user_transactions,
            transactions::ui::get_recent_transactions,
            transactions::ui::get_transaction_stats,
            transactions_log::ui::get_transaction_log,
            transactions_log::ui::get_full_transaction_log,
            transactions_log::ui::list_full_transaction_logs,
            transactions_log::ui::list_accounts_full_transaction_logs,
            transactions_log::ui::count_transaction_logs,
            // logs
            api_logs::ui::get_api_log,
            api_logs::ui::update_api_log,
            api_logs::ui::delete_api_log,
            api_logs::ui::count_api_logs,
            api_logs::ui::get_project_api_logs,
            api_logs::ui::get_api_logs_by_method,
            api_logs::ui::list_api_logs,
            // Callbacks
            callbacks::stk::ui::resolve_stk_prompt,
            // Transaction Costs
            transaction_costs::ui::create_transaction_cost,
            transaction_costs::ui::list_transaction_costs,
            transaction_costs::ui::update_transaction_cost,
            transaction_costs::ui::delete_transaction_cost,
            transaction_costs::ui::calculate_transaction_fee,
        ]);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
