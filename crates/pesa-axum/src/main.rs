use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    State,
};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
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
use pesa_macros::generate_axum_rpc_handler;
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const WEBSOCKET_CHANNEL_CAPACITY: usize = 100;

pub struct AxumEventManager {
    sender: broadcast::Sender<serde_json::Value>,
}

impl AppEventManager for AxumEventManager {
    fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()> {
        let event_payload = serde_json::json!({            "event": event,
            "payload": payload,
        });
        self.sender.send(event_payload)?;
        Ok(())
    }
}

// AxumAppState will hold the core context and the Axum-specific event manager
#[derive(Clone)]
pub struct AxumAppState {
    pub core_context: AppContext,
    pub event_manager: Arc<AxumEventManager>,
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AxumAppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state.event_manager))
}

async fn handle_socket(socket: WebSocket, event_manager: Arc<AxumEventManager>) {
    let mut receiver = event_manager.sender.subscribe();

    // Split the WebSocket into sender and receiver halves
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Task to send events from the broadcast channel to the WebSocket client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = receiver.recv().await {
            if ws_sender
                .send(Message::Text(msg.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Task to receive messages from the WebSocket client (e.g., pings, close messages)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
            // Optionally handle incoming messages from the client
            println!("Received from WebSocket: {}", text);
        }
    });

    // If either task completes, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

generate_axum_rpc_handler! {
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

    resolve_stk_prompt(checkout_id: String, result: UserResponse) => pesa_core::callbacks::stk::ui::resolve_stk_prompt
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Address to listen on
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Path to the SvelteKit build output (webroot)
    #[arg(short, long, default_value = ".")]
    webroot: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pesa_axum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli_args = CliArgs::parse();

    let db_path = std::path::Path::new("database.sqlite");
    let db = pesa_core::db::Database::new(db_path)
        .await
        .expect("Failed to initialize database");

    if let Err(err) = db.init().await {
        eprintln!("Database error: {:?}", err);
    }

    let (event_sender, _event_receiver) = broadcast::channel(WEBSOCKET_CHANNEL_CAPACITY);
    let axum_event_manager = Arc::new(AxumEventManager {
        sender: event_sender.clone(),
    });

    let core_context = AppContext {
        db: db.conn.clone(),
        event_manager: axum_event_manager.clone(),
        running: Arc::new(Mutex::new(HashMap::new())),
    };

    let app_state = AxumAppState {
        core_context,
        event_manager: axum_event_manager,
    };

    let app = Router::new()
        .route("/rpc", post(rpc_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .fallback_service(ServeDir::new(cli_args.webroot))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", cli_args.address, cli_args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
