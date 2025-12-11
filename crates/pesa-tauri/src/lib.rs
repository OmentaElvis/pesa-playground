use pesa_core::{
    accounts::{
        paybill_accounts::{CreatePaybillAccount, UpdatePaybillAccount},
        till_accounts::{CreateTillAccount, UpdateTillAccount},
    },
    api_logs::{ui::ApiLogFilter, UpdateApiLogRequest},
    business::{CreateBusiness, UpdateBusiness},
    callbacks::stk::UserResponse,
    projects::{CreateProject, UpdateProject},
    transaction_costs::ui::TransactionCostData,
    transactions::{
        ui::{LipaArgs, TransactionFilter},
        TransactionType,
    },
    AppContext, AppEventManager,
};
use pesa_lua::ScriptManager;
use pesa_macros::generate_tauri_wrappers;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager, Runtime, State};
use tokio::sync::{broadcast, Mutex};

const WEBSOCKET_CHANNEL_CAPACITY: usize = 100;

#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    // Close splashscreen
    if let Some(splashscreen) = window.get_webview_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    // Show main window
    window.get_webview_window("main").unwrap().show().unwrap();
}

pub struct TauriEventManager<R: Runtime> {
    pub app_handle: tauri::AppHandle<R>,
    pub sender: broadcast::Sender<serde_json::Value>,
}

impl<R: Runtime> AppEventManager for TauriEventManager<R> {
    fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()> {
        // Emit to Tauri frontend
        self.app_handle.emit(event, payload.clone())?;
        // Broadcast internally
        self.sender.send(serde_json::json!({
            "event": event,
            "payload": payload,
        }))?;
        Ok(())
    }
}

// Define the Tauri application state
pub struct TauriAppState {
    pub context: AppContext,
    pub script_manager: Arc<Mutex<ScriptManager>>,
}

generate_tauri_wrappers! {
    TauriAppState,
    // Existing commands
    start_sandbox(project_id: u32) => pesa_core::sandboxes::ui::start_sandbox,
    stop_sandbox(project_id: u32) => pesa_core::sandboxes::ui::stop_sandbox,
    sandbox_status(project_id: u32) => pesa_core::sandboxes::ui::sandbox_status,
    list_running_sandboxes() => pesa_core::sandboxes::ui::list_running_sandboxes,

    create_project(input: CreateProject) => pesa_core::projects::ui::create_project,
    get_project(id: u32) => pesa_core::projects::ui::get_project,
    get_projects() => pesa_core::projects::ui::get_projects,
    get_projects_by_business_id(business_id: u32) => pesa_core::projects::ui::get_projects_by_business_id,
    update_project(id: u32, input: UpdateProject) => pesa_core::projects::ui::update_project,
    delete_project(id: u32) => pesa_core::projects::ui::delete_project,

    create_business(input: CreateBusiness) => pesa_core::business::ui::create_business,
    get_business(id: u32) => pesa_core::business::ui::get_business,
    get_businesses() => pesa_core::business::ui::get_businesses,
    update_business(id: u32, input: UpdateBusiness) => pesa_core::business::ui::update_business,
    delete_business(id: u32) => pesa_core::business::ui::delete_business,

    get_users() => pesa_core::accounts::user_profiles::ui::get_users,
    create_user(name: String, phone: String, balance: f64, pin: String) => pesa_core::accounts::user_profiles::ui::create_user,
    remove_user(user_id: u32) => pesa_core::accounts::user_profiles::ui::remove_user,
    get_user(user_id: u32) => pesa_core::accounts::user_profiles::ui::get_user,
    #[no_context]
    generate_user() => pesa_core::accounts::user_profiles::ui::generate_user,
    #[no_context]
    generate_users(count: u32) => pesa_core::accounts::user_profiles::ui::generate_users,
    get_user_by_phone(phone: String) => pesa_core::accounts::user_profiles::ui::get_user_by_phone,
    update_user(user_id: u32, name: Option<String>, balance: Option<i64>, pin: Option<String>) => pesa_core::accounts::user_profiles::ui::update_user,

    create_paybill_account(input: CreatePaybillAccount) => pesa_core::accounts::paybill_accounts::ui::create_paybill_account,
    get_paybill_account(id: u32) => pesa_core::accounts::paybill_accounts::ui::get_paybill_account,
    get_paybill_accounts() => pesa_core::accounts::paybill_accounts::ui::get_paybill_accounts,
    get_paybill_accounts_by_business_id(business_id: u32) => pesa_core::accounts::paybill_accounts::ui::get_paybill_accounts_by_business_id,
    update_paybill_account(id: u32, input: UpdatePaybillAccount) => pesa_core::accounts::paybill_accounts::ui::update_paybill_account,
    delete_paybill_account(id: u32) => pesa_core::accounts::paybill_accounts::ui::delete_paybill_account,

    create_till_account(input: CreateTillAccount) => pesa_core::accounts::till_accounts::ui::create_till_account,
    get_till_account(id: u32) => pesa_core::accounts::till_accounts::ui::get_till_account,
    get_till_accounts() => pesa_core::accounts::till_accounts::ui::get_till_accounts,
    get_till_accounts_by_business_id(business_id: u32) => pesa_core::accounts::till_accounts::ui::get_till_accounts_by_business_id,
    update_till_account(id: u32, input: UpdateTillAccount) => pesa_core::accounts::till_accounts::ui::update_till_account,
    delete_till_account(id: u32) => pesa_core::accounts::till_accounts::ui::delete_till_account,

    get_transaction(transaction_id: String) => pesa_core::transactions::ui::get_transaction,
    list_transactions(filter: TransactionFilter) => pesa_core::transactions::ui::list_transactions,
    list_system_transactions(limit: Option<u32>, offset: Option<u32>) => pesa_core::transactions::ui::list_system_transactions,
    count_transactions(filter: TransactionFilter) => pesa_core::transactions::ui::count_transactions,
    get_transaction_by_checkout_request(checkout_request_id: String) => pesa_core::transactions::ui::get_transaction_by_checkout_request,
    get_user_transactions(user_id: u32, limit: Option<u32>, offset: Option<u32>) => pesa_core::transactions::ui::get_user_transactions,
    get_recent_transactions(limit: Option<u32>) => pesa_core::transactions::ui::get_recent_transactions,
    get_transaction_stats() => pesa_core::transactions::ui::get_transaction_stats,
    transfer(source: Option<u32>, destination: u32, amount: i64, txn_type: TransactionType) => pesa_core::transactions::ui::transfer,
    reverse(id: String) => pesa_core::transactions::ui::reverse,
    lipa(args: LipaArgs) => pesa_core::transactions::ui::lipa,

    get_transaction_log(transaction_id: i32) => pesa_core::transactions_log::ui::get_transaction_log,
    get_full_transaction_log(transaction_log_id: i32) => pesa_core::transactions_log::ui::get_full_transaction_log,
    list_full_transaction_logs(account_id: i32, limit: Option<u64>, offset: Option<u64>) => pesa_core::transactions_log::ui::list_full_transaction_logs,
    list_accounts_full_transaction_logs(accounts: Vec<u32>, limit: Option<u64>, offset: Option<u64>) => pesa_core::transactions_log::ui::list_accounts_full_transaction_logs,
    count_transaction_logs(accounts: Vec<u32>) => pesa_core::transactions_log::ui::count_transaction_logs,

    get_api_log(log_id: String) => pesa_core::api_logs::ui::get_api_log,
    update_api_log(log_id: String, request: UpdateApiLogRequest) => pesa_core::api_logs::ui::update_api_log,
    delete_api_log(log_id: String) => pesa_core::api_logs::ui::delete_api_log,
    list_api_logs(filter: ApiLogFilter) => pesa_core::api_logs::ui::list_api_logs,
    count_api_logs(project_id: Option<i64>, method: Option<String>, path: Option<String>, status_code: Option<i32>) => pesa_core::api_logs::ui::count_api_logs,
    get_project_api_logs(project_id: u32, filter: ApiLogFilter) => pesa_core::api_logs::ui::get_project_api_logs,
    get_api_logs_by_method(project_id: u32, method: String, limit: Option<u64>) => pesa_core::api_logs::ui::get_api_logs_by_method,

    create_transaction_cost(data: TransactionCostData) => pesa_core::transaction_costs::ui::create_transaction_cost,
    list_transaction_costs() => pesa_core::transaction_costs::ui::list_transaction_costs,
    update_transaction_cost(id: i32, data: TransactionCostData) => pesa_core::transaction_costs::ui::update_transaction_cost,
    delete_transaction_cost(id: i32) => pesa_core::transaction_costs::ui::delete_transaction_cost,
    calculate_transaction_fee(txn_type: TransactionType, amount: i64) => pesa_core::transaction_costs::ui::calculate_transaction_fee,

    resolve_stk_prompt(checkout_id: String, result: UserResponse) => pesa_core::callbacks::stk::ui::resolve_stk_prompt,
    #[no_context]
    get_app_info() => pesa_core::info::get_app_info,

    get_account(id: u32) => pesa_core::accounts::ui::get_account,
    create_account(account_type: pesa_core::accounts::AccountType, initial_balance: i64) => pesa_core::accounts::ui::create_account,
}

#[tauri::command]
async fn scripts_list(state: State<'_, TauriAppState>) -> Result<Vec<String>, String> {
    let manager = state.script_manager.lock().await;
    manager.list_scripts().map_err(|e| e.to_string())
}

#[tauri::command]
async fn scripts_read(name: String, state: State<'_, TauriAppState>) -> Result<String, String> {
    let manager = state.script_manager.lock().await;
    manager.read_script(&name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scripts_save(
    name: String,
    content: String,
    state: State<'_, TauriAppState>,
) -> Result<(), String> {
    let manager = state.script_manager.lock().await;
    manager
        .save_script(&name, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scripts_delete(name: String, state: State<'_, TauriAppState>) -> Result<(), String> {
    let manager = state.script_manager.lock().await;
    manager.delete_script(&name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scripts_execute(
    content: String,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let manager = state.script_manager.lock().await;
    manager
        .execute_script(&content)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let app_dir = handle.path().app_data_dir().expect("failed to get app dir");
            let db_path = app_dir.join("database.sqlite");

            tauri::async_runtime::block_on(async move {
                let db = pesa_core::db::Database::new(&db_path)
                    .await
                    .expect("Failed to initialize database");

                if let Err(err) = db.init().await {
                    eprintln!("Database error: {:?}", err);
                }

                let (event_sender, _event_receiver) =
                    broadcast::channel(WEBSOCKET_CHANNEL_CAPACITY);
                let event_manager = Arc::new(TauriEventManager {
                    app_handle: handle.clone(),
                    sender: event_sender.clone(),
                });

                let context = AppContext {
                    db: db.conn.clone(),
                    event_manager: event_manager.clone(),
                    running: Arc::new(Mutex::new(HashMap::new())),
                };

                // Initialize ScriptManager
                let script_manager = ScriptManager::new(context.clone(), &app_dir)
                    .expect("Failed to initialize script manager");

                let script_manager_clone = script_manager.clone();
                let mut script_event_receiver = event_sender.subscribe();

                tauri::async_runtime::spawn(async move {
                    loop {
                        match script_event_receiver.recv().await {
                            Ok(event_payload) => {
                                if let (Some(event_name), Some(payload)) = (
                                    event_payload["event"].as_str(),
                                    event_payload["payload"].as_object(),
                                ) {
                                    let sm = script_manager_clone.lock().await;
                                    sm.emit_event(
                                        event_name,
                                        serde_json::Value::Object(payload.clone()),
                                    )
                                    .await;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error receiving event in script manager: {:?}", e);
                                break;
                            }
                        }
                    }
                });

                // Manage the combined state
                app.manage(TauriAppState {
                    context,
                    script_manager,
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            close_splashscreen,
            // Scripting Commands
            scripts_list,
            scripts_read,
            scripts_save,
            scripts_delete,
            scripts_execute,
            // Core Commands
            start_sandbox,
            stop_sandbox,
            sandbox_status,
            list_running_sandboxes,
            create_project,
            get_project,
            get_projects,
            get_projects_by_business_id,
            update_project,
            delete_project,
            create_business,
            get_business,
            get_businesses,
            update_business,
            delete_business,
            get_users,
            create_user,
            remove_user,
            get_user,
            generate_user,
            generate_users,
            get_user_by_phone,
            update_user,
            create_paybill_account,
            get_paybill_account,
            get_paybill_accounts,
            get_paybill_accounts_by_business_id,
            update_paybill_account,
            delete_paybill_account,
            create_till_account,
            get_till_account,
            get_till_accounts,
            get_till_accounts_by_business_id,
            update_till_account,
            delete_till_account,
            get_transaction,
            list_transactions,
            count_transactions,
            get_transaction_by_checkout_request,
            get_user_transactions,
            get_recent_transactions,
            get_transaction_stats,
            transfer,
            reverse,
            lipa,
            get_transaction_log,
            get_full_transaction_log,
            list_full_transaction_logs,
            list_accounts_full_transaction_logs,
            count_transaction_logs,
            get_api_log,
            update_api_log,
            delete_api_log,
            list_api_logs,
            count_api_logs,
            get_project_api_logs,
            get_api_logs_by_method,
            create_transaction_cost,
            list_transaction_costs,
            update_transaction_cost,
            delete_transaction_cost,
            calculate_transaction_fee,
            resolve_stk_prompt,
            get_app_info,
            get_account,
            create_account
        ]);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
