use crate::types::*;
use anyhow::Result;
use mlua::{
    AnyUserData, Function, Lua, LuaSerdeExt, RegistryKey, Result as LuaResult, UserData,
    UserDataMethods,
};
use pesa_core::AppContext;
use pesa_macros::generate_lua_bindings;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::Mutex as TokioMutex;
use tracing::error;

// A wrapper for AppContext to be stored in Lua registry
#[derive(Clone)]
struct LuaAppContext(AppContext);

impl UserData for LuaAppContext {
    fn add_methods<M: UserDataMethods<Self>>(_methods: &mut M) {
        // No methods exposed directly to Lua for AppContext itself
    }
}

// Helper to get AppContext from Lua registry
fn get_app_context_from_lua(lua: &Lua) -> mlua::Result<AppContext> {
    let app_context_ref: AnyUserData = lua.named_registry_value("app_context")?;
    let app_context = app_context_ref.borrow::<LuaAppContext>()?;
    Ok(app_context.0.clone())
}

generate_lua_bindings! {
    start_sandbox(project_id: u32) => pesa_core::sandboxes::ui::start_sandbox,
    stop_sandbox(project_id: u32) => pesa_core::sandboxes::ui::stop_sandbox,
    sandbox_status(project_id: u32) => pesa_core::sandboxes::ui::sandbox_status,
    list_running_sandboxes() => pesa_core::sandboxes::ui::list_running_sandboxes,

    create_project(#[wrap] input: CreateProject) => pesa_core::projects::ui::create_project,
    get_project(id: u32) => pesa_core::projects::ui::get_project,
    get_projects() => pesa_core::projects::ui::get_projects,
    get_projects_by_business_id(business_id: u32) => pesa_core::projects::ui::get_projects_by_business_id,
    update_project(id: u32, #[wrap] input: UpdateProject) => pesa_core::projects::ui::update_project,
    delete_project(id: u32) => pesa_core::projects::ui::delete_project,

    create_business(#[wrap] input: CreateBusiness) => pesa_core::business::ui::create_business,
    get_business(id: u32) => pesa_core::business::ui::get_business,
    get_businesses() => pesa_core::business::ui::get_businesses,
    update_business(id: u32, #[wrap] input: UpdateBusiness) => pesa_core::business::ui::update_business,
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

    create_paybill_account(#[wrap] input: CreatePaybillAccount) => pesa_core::accounts::paybill_accounts::ui::create_paybill_account,
    get_paybill_account(id: u32) => pesa_core::accounts::paybill_accounts::ui::get_paybill_account,
    get_paybill_accounts() => pesa_core::accounts::paybill_accounts::ui::get_paybill_accounts,
    get_paybill_accounts_by_business_id(business_id: u32) => pesa_core::accounts::paybill_accounts::ui::get_paybill_accounts_by_business_id,
    update_paybill_account(id: u32, #[wrap] input: UpdatePaybillAccount) => pesa_core::accounts::paybill_accounts::ui::update_paybill_account,
    delete_paybill_account(id: u32) => pesa_core::accounts::paybill_accounts::ui::delete_paybill_account,

    create_till_account(#[wrap] input: CreateTillAccount) => pesa_core::accounts::till_accounts::ui::create_till_account,
    get_till_account(id: u32) => pesa_core::accounts::till_accounts::ui::get_till_account,
    get_till_accounts() => pesa_core::accounts::till_accounts::ui::get_till_accounts,
    get_till_accounts_by_business_id(business_id: u32) => pesa_core::accounts::till_accounts::ui::get_till_accounts_by_business_id,
    update_till_account(id: u32, #[wrap] input: UpdateTillAccount) => pesa_core::accounts::till_accounts::ui::update_till_account,
    delete_till_account(id: u32) => pesa_core::accounts::till_accounts::ui::delete_till_account,

    get_transaction(transaction_id: String) => pesa_core::transactions::ui::get_transaction,
    list_transactions(#[wrap] filter: TransactionFilter) => pesa_core::transactions::ui::list_transactions,
    list_system_transactions(limit: Option<u32>, offset: Option<u32>) => pesa_core::transactions::ui::list_system_transactions,
    count_transactions(#[wrap] filter: TransactionFilter) => pesa_core::transactions::ui::count_transactions,
    get_transaction_by_checkout_request(checkout_request_id: String) => pesa_core::transactions::ui::get_transaction_by_checkout_request,
    get_user_transactions(user_id: u32, limit: Option<u32>, offset: Option<u32>) => pesa_core::transactions::ui::get_user_transactions,
    get_recent_transactions(limit: Option<u32>) => pesa_core::transactions::ui::get_recent_transactions,
    get_transaction_stats() => pesa_core::transactions::ui::get_transaction_stats,
    // transfer(source: Option<u32>, destination: u32, amount: i64, #[wrap] txn_type: TransactionType, #[wrap] notes: Option<TransactionNote>) => pesa_core::transactions::ui::transfer,
    reverse(id: String) => pesa_core::transactions::ui::reverse,
    lipa(#[wrap] args: LipaArgs) => pesa_core::transactions::ui::lipa,

    get_transaction_log(transaction_id: u32) => pesa_core::transactions_log::ui::get_transaction_log,
    get_full_transaction_log(transaction_log_id: u32) => pesa_core::transactions_log::ui::get_full_transaction_log,
    list_full_transaction_logs(account_id: i32, limit: Option<u64>, offset: Option<u64>) => pesa_core::transactions_log::ui::list_full_transaction_logs,
    list_accounts_full_transaction_logs(accounts: Vec<u32>, limit: Option<u64>, offset: Option<u64>) => pesa_core::transactions_log::ui::list_accounts_full_transaction_logs,
    count_transaction_logs(accounts: Vec<u32>) => pesa_core::transactions_log::ui::count_transaction_logs,

    get_api_log(log_id: String) => pesa_core::api_logs::ui::get_api_log,
    update_api_log(log_id: String, #[wrap] request: UpdateApiLogRequest) => pesa_core::api_logs::ui::update_api_log,
    delete_api_log(log_id: String) => pesa_core::api_logs::ui::delete_api_log,
    list_api_logs(#[wrap] filter: ApiLogFilter) => pesa_core::api_logs::ui::list_api_logs,
    count_api_logs(project_id: Option<i64>, method: Option<String>, path: Option<String>, status_code: Option<i32>) => pesa_core::api_logs::ui::count_api_logs,
    get_project_api_logs(project_id: u32, #[wrap] filter: ApiLogFilter) => pesa_core::api_logs::ui::get_project_api_logs,
    get_api_logs_by_method(project_id: u32, method: String, limit: Option<u64>) => pesa_core::api_logs::ui::get_api_logs_by_method,

    create_transaction_cost(#[wrap] data: TransactionCostData) => pesa_core::transaction_costs::ui::create_transaction_cost,
    list_transaction_costs() => pesa_core::transaction_costs::ui::list_transaction_costs,
    update_transaction_cost(id: i32, #[wrap] data: TransactionCostData) => pesa_core::transaction_costs::ui::update_transaction_cost,
    delete_transaction_cost(id: i32) => pesa_core::transaction_costs::ui::delete_transaction_cost,
    calculate_transaction_fee(#[wrap] txn_type: TransactionType, amount: i64) => pesa_core::transaction_costs::ui::calculate_transaction_fee,

    resolve_stk_prompt(checkout_id: String, #[wrap] result: UserResponse) => pesa_core::callbacks::stk::ui::resolve_stk_prompt,
    #[no_context]
    get_app_info() => pesa_core::info::get_app_info,

    get_account(id: u32) => pesa_core::accounts::ui::get_account,
    create_account(#[wrap] account_type: AccountType, initial_balance: i64) => pesa_core::accounts::ui::create_account,

    get_utility_account(id: u32) => pesa_core::accounts::utility_accounts::ui::get_utility_account,
    get_mmf_account(id: u32) => pesa_core::accounts::mmf_accounts::ui::get_mmf_account,

    revenue_settlement(business_id: u32) => pesa_core::business::ui::revenue_settlement
}

type ListenerMap = Arc<Mutex<HashMap<String, Vec<RegistryKey>>>>;

pub struct ScriptManager {
    lua: Arc<Lua>,
    scripts_dir: PathBuf,
    listeners: ListenerMap,
    has_listeners: Arc<Mutex<bool>>,
}

// Helper function to normalize script names
fn normalize_name(name: &str) -> String {
    // Simple normalization: lowercase, replace spaces with hyphens, and add .lua extension
    // A more robust solution might use a slugify library.
    format!("{}.lua", name.to_lowercase().replace(' ', "-"))
}

impl ScriptManager {
    pub fn new(app_context: AppContext, data_dir: &Path) -> Result<Arc<TokioMutex<Self>>> {
        let lua = Arc::new(Lua::new());
        let app_context_clone = app_context.clone();
        let scripts_dir = data_dir.join("scripts");
        fs::create_dir_all(&scripts_dir)?;

        let listeners: ListenerMap = Arc::new(Mutex::new(HashMap::new()));
        let has_listeners = Arc::new(Mutex::new(false));

        let res: Result<(), mlua::Error> = (|| {
            // Store AppContext in the registry
            let lua_app_context = LuaAppContext(app_context_clone);
            lua.set_named_registry_value("app_context", lua_app_context)?;

            // Create the global 'pesa' table
            let pesa_table = lua.create_table()?;
            lua.globals().set("pesa", pesa_table.clone())?;

            // Create 'pesa.event' table
            let event_table = lua.create_table()?;
            let listen_listeners = listeners.clone();
            let listen_has_listeners = has_listeners.clone();

            let listen_fn =
                lua.create_function(move |lua, (event_name, handler): (String, Function)| {
                    let registry_key = lua.create_registry_value(handler)?;
                    let mut listeners_lock = listen_listeners.lock().unwrap();
                    listeners_lock
                        .entry(event_name)
                        .or_default()
                        .push(registry_key);
                    *listen_has_listeners.lock().unwrap() = true;
                    Ok(())
                })?;
            event_table.set("listen", listen_fn)?;
            pesa_table.set("event", event_table)?;

            // Call the function generated by the macro to populate the 'pesa' table
            register_lua_bindings(lua.as_ref())?;

            Ok(())
        })();

        if let Err(e) = res {
            error!("Failed to initialize Lua context: {}", e);
            // Using anyhow::Result, so convert the error
            return Err(anyhow::anyhow!("Failed to initialize Lua context: {}", e));
        }

        let manager = Self {
            lua,
            scripts_dir,
            listeners,
            has_listeners,
        };

        Ok(Arc::new(TokioMutex::new(manager)))
    }

    pub fn has_listeners(&self) -> bool {
        *self.has_listeners.lock().unwrap()
    }

    pub async fn execute_script(&self, script_code: &str) -> Result<String> {
        let result: LuaResult<String> = self.lua.load(script_code).eval_async().await;

        match result {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("Lua script execution error: {:?}", e);
                Err(anyhow::anyhow!("Lua script error: {}", e))
            }
        }
    }

    pub fn list_scripts(&self) -> Result<Vec<String>> {
        let mut scripts = Vec::new();
        for entry in fs::read_dir(&self.scripts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    scripts.push(filename.to_string());
                }
            }
        }
        Ok(scripts)
    }

    pub fn read_script(&self, name: &str) -> Result<String> {
        let path = self.scripts_dir.join(normalize_name(name));
        Ok(fs::read_to_string(path)?)
    }

    pub fn save_script(&self, name: &str, content: &str) -> Result<()> {
        let path = self.scripts_dir.join(normalize_name(name));
        Ok(fs::write(path, content)?)
    }

    pub fn delete_script(&self, name: &str) -> Result<()> {
        let path = self.scripts_dir.join(normalize_name(name));
        Ok(fs::remove_file(path)?)
    }

    pub async fn emit_event(&self, event_name: &str, payload: serde_json::Value) {
        let listeners_lock = self.listeners.lock().unwrap();
        if let Some(handlers_ref) = listeners_lock.get(event_name) {
            for handler_key in handlers_ref {
                match self.lua.registry_value::<Function>(handler_key) {
                    Ok(func) => match self.lua.to_value(&payload) {
                        Ok(lua_payload) => {
                            let event_name_clone = event_name.to_string();
                            tokio::spawn(async move {
                                if let Err(e) = func.call_async::<()>(lua_payload).await {
                                    error!(
                                        "Error executing Lua event handler for '{}': {:?}",
                                        event_name_clone, e
                                    );
                                }
                            });
                        }
                        Err(e) => {
                            error!(
                                "Error converting payload for Lua event handler '{}': {:?}",
                                event_name, e
                            );
                        }
                    },
                    Err(e) => {
                        error!(
                            "Error retrieving Lua event handler for '{}': {:?}",
                            event_name, e
                        );
                    }
                }
            }
        }
    }
}
