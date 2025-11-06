use std::{collections::HashMap, sync::Arc};

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

use crate::sandboxes::RunningSandbox;

pub mod accounts;
pub mod api_keys;
pub mod api_logs;
pub mod business;
pub mod callbacks;
pub mod db;
pub mod events;
pub mod projects;
pub mod sandboxes;
pub mod server;
pub mod transaction_costs;
pub mod transactions;
pub mod transactions_log;
pub mod user;
pub mod info;

pub trait AppEventManager {
    fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct AppContext {
    pub db: DatabaseConnection,
    pub event_manager: Arc<dyn AppEventManager + Send + Sync>,
    pub running: Arc<Mutex<HashMap<u32, RunningSandbox>>>,
}

// sandbox::ui::start_sandbox,
// sandbox::ui::stop_sandbox,
// sandbox::ui::sandbox_status,
// sandbox::ui::list_running_sandboxes,
// projects::ui::create_project,
// projects::ui::get_project,
// projects::ui::get_projects,
// projects::ui::update_project,
// projects::ui::delete_project,
// projects::ui::get_projects_by_business_id,
// // businesses
// business::ui::create_business,
// business::ui::get_business,
// business::ui::get_businesses,
// business::ui::update_business,
// business::ui::delete_business,
// // accounts
// accounts::ui::get_account,
// accounts::ui::create_account,
// accounts::paybill_accounts::ui::create_paybill_account,
// accounts::paybill_accounts::ui::get_paybill_account,
// accounts::paybill_accounts::ui::get_paybill_accounts,
// accounts::paybill_accounts::ui::get_paybill_accounts_by_business_id,
// accounts::paybill_accounts::ui::update_paybill_account,
// accounts::paybill_accounts::ui::delete_paybill_account,
// accounts::till_accounts::ui::create_till_account,
// accounts::till_accounts::ui::get_till_account,
// accounts::till_accounts::ui::get_till_accounts,
// accounts::till_accounts::ui::get_till_accounts_by_business_id,
// accounts::till_accounts::ui::update_till_account,
// accounts::till_accounts::ui::delete_till_account,
// // users
// accounts::user_profiles::ui::get_users,
// accounts::user_profiles::ui::create_user,
// accounts::user_profiles::ui::remove_user,
// accounts::user_profiles::ui::get_user,
// accounts::user_profiles::ui::update_user,
// accounts::user_profiles::ui::generate_user,
// accounts::user_profiles::ui::generate_users,
// accounts::user_profiles::ui::get_user_by_phone,
// // transactions
// transactions::ui::lipa,
// transactions::ui::transfer,
// transactions::ui::reverse,
// transactions::ui::get_transaction,
// transactions::ui::list_transactions,
// transactions::ui::count_transactions,
// transactions::ui::get_transaction_by_checkout_request,
// transactions::ui::get_user_transactions,
// transactions::ui::get_recent_transactions,
// transactions::ui::get_transaction_stats,
// transactions_log::ui::get_transaction_log,
// transactions_log::ui::get_full_transaction_log,
// transactions_log::ui::list_full_transaction_logs,
// transactions_log::ui::list_accounts_full_transaction_logs,
// transactions_log::ui::count_transaction_logs,
// // logs
// api_logs::ui::get_api_log,
// api_logs::ui::update_api_log,
// api_logs::ui::delete_api_log,
// api_logs::ui::count_api_logs,
// api_logs::ui::get_project_api_logs,
// api_logs::ui::get_api_logs_by_method,
// api_logs::ui::list_api_logs,
// // Callbacks
// callbacks::stk::ui::resolve_stk_prompt,
// // Transaction Costs
// transaction_costs::ui::create_transaction_cost,
// transaction_costs::ui::list_transaction_costs,
// transaction_costs::ui::update_transaction_cost,
// transaction_costs::ui::delete_transaction_cost,
// transaction_costs::ui::calculate_transaction_fee,
