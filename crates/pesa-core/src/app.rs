use crate::{
    AppContext, AppEventManager, TransactionChannel, settings::SettingsManager,
    transaction_jobs::task::TransactionManager,
};
use anyhow::Context;
use futures::future::FutureExt;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::{
    panic::AssertUnwindSafe,
    path::Path,
    sync::{Arc, RwLock},
};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug, Clone, Serialize)]
pub enum TransactionManagerStatus {
    Running,
    Panicked,
    Stopped,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionManagerStats {
    pub status: TransactionManagerStatus,
    pub pending_jobs_count: usize,
    pub processed_jobs_count: u64,
    pub last_error_message: Option<String>,
}

pub struct PesaApp {
    pub context: Arc<AppContext>,
    stats: Arc<RwLock<TransactionManagerStats>>,
    shutdown_tx: mpsc::Sender<()>,
    transaction_manager_handle: Option<JoinHandle<()>>,
}

impl PesaApp {
    pub async fn new(
        db_conn: DatabaseConnection,
        settings: SettingsManager,
        event_manager: Arc<dyn AppEventManager + Send + Sync>,
        app_root: &Path,
    ) -> anyhow::Result<Self> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let (transaction_tx, transaction_rx) = mpsc::channel(1024);

        let initial_stats = TransactionManagerStats {
            status: TransactionManagerStatus::Running,
            pending_jobs_count: 0,
            processed_jobs_count: 0,
            last_error_message: None,
        };
        let stats = Arc::new(RwLock::new(initial_stats));

        let request_lifecycle_manager = Arc::new(
            crate::request_lifecycle::RequestLifecycleManager::new(db_conn.clone()),
        );

        let context = Arc::new(AppContext {
            db: db_conn.clone(),
            settings,
            event_manager,
            running: Arc::new(crate::dashmap::DashMap::new()),
            app_root: app_root.to_path_buf(),
            stats: Arc::clone(&stats),
            txn_manager: TransactionChannel { transaction_tx },
            request_lifecycle_manager,
        });

        let mut transaction_manager =
            TransactionManager::new(db_conn, transaction_rx, Arc::clone(&stats));

        let stats_clone = Arc::clone(&stats);
        let manager_handle = tokio::spawn(async move {
            let run_future = AssertUnwindSafe(transaction_manager.run(shutdown_rx))
                .catch_unwind()
                .await;

            let mut stats_guard = stats_clone.write().unwrap();
            match run_future {
                Ok(Ok(())) => {
                    stats_guard.status = TransactionManagerStatus::Stopped;
                    tracing::info!("TransactionManager shut down gracefully.");
                }
                Ok(Err(e)) => {
                    stats_guard.status = TransactionManagerStatus::Panicked;
                    stats_guard.last_error_message =
                        Some(format!("Task exited with error: {:?}", e));
                    tracing::error!("{}", stats_guard.last_error_message.as_ref().unwrap());
                }
                Err(panic_payload) => {
                    stats_guard.status = TransactionManagerStatus::Panicked;
                    let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic payload".to_string()
                    };
                    stats_guard.last_error_message = Some(format!("Panic: {}", msg));
                    tracing::error!("{}", stats_guard.last_error_message.as_ref().unwrap());
                }
            }
        });

        Ok(Self {
            context,
            stats,
            shutdown_tx,
            transaction_manager_handle: Some(manager_handle),
        })
    }

    pub async fn get_transaction_manager_stats(&self) -> TransactionManagerStats {
        self.stats.read().unwrap().clone()
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        if self.shutdown_tx.is_closed() {
            return Ok(());
        }

        self.shutdown_tx
            .send(())
            .await
            .context("Failed to send shutdown signal to TransactionManager")?;

        if let Some(handle) = self.transaction_manager_handle.take() {
            handle
                .await
                .context("TransactionManager task failed to shut down cleanly")?;
        }

        Ok(())
    }
}
