use super::{JobResultPayload, TransactionJob as TransactionJobModel};
use crate::{
    AppContext,
    app::TransactionManagerStats,
    events::DomainEvent,
    transaction_jobs::JobPayload,
    transactions::{
        Ledger, Transaction, TransactionEngineError, TransactionNote, TransactionStatus,
        TransactionType,
    },
    utils::identifiers::Identifiers,
};
use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::ConnectionTrait;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

// In-memory job struct for the min-heap
pub struct ScheduledTransactionTask {
    /// The delay at which the transaction is to be processed.
    /// Useful for simulating slow transaction speed.
    /// If set to 0, then the request will be processed immediately instead of scheduling
    pub process_after: std::time::Instant,
    pub job_id: Uuid,
    pub identifiers: Identifiers,
    /// The source wallet/account
    pub source: Option<u32>,
    /// The destination wallet/account
    pub destination: u32,
    pub amount: i64,
    pub txn_type: TransactionType,
    pub notes: Option<TransactionNote>,
    /// Original requested delay
    pub delay: Duration,
    /// The reply path for the waiting caller
    pub reply_sender:
        oneshot::Sender<Result<(Transaction, Vec<DomainEvent>), TransactionEngineError>>,
}

// Implement PartialEq, Eq, PartialOrd, Ord for BinaryHeap to treat it as a min-heap based on process_after
impl PartialEq for ScheduledTransactionTask {
    fn eq(&self, other: &Self) -> bool {
        self.process_after == other.process_after
    }
}

impl Eq for ScheduledTransactionTask {}

impl PartialOrd for ScheduledTransactionTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTransactionTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.process_after.cmp(&other.process_after)
    }
}

// The Transaction Manager (the scheduler)
pub struct TransactionManager<C: ConnectionTrait> {
    conn: C,
    receiver: mpsc::Receiver<ScheduledTransactionTask>,
    pending_jobs: BinaryHeap<std::cmp::Reverse<ScheduledTransactionTask>>,
    stats: Arc<RwLock<TransactionManagerStats>>,
}

async fn process_single_transaction_job<C: ConnectionTrait>(
    conn: &C,
    job: ScheduledTransactionTask,
) {
    let job_id = job.job_id;
    let result_sender = job.reply_sender;

    let transaction_result = Ledger::transfer_with_id(
        conn,
        &job.identifiers.transaction_id,
        job.source,
        job.destination,
        job.amount,
        &job.txn_type,
        job.notes.as_ref(),
    )
    .await;

    let status;
    let result_payload;
    let final_result;

    match transaction_result {
        Ok((txn, events)) => {
            status = TransactionStatus::Completed;
            result_payload = Some(JobResultPayload::Completed(txn.clone()));
            final_result = Ok((txn, events));
        }
        Err(err) => {
            status = TransactionStatus::Failed;
            result_payload = Some(JobResultPayload::Failed {
                message: err.to_string(),
            });
            tracing::error!(
                target: "transactions",
                "Transaction job failed: {} - Error: {}",
                job.identifiers.transaction_id,
                err
            );
            final_result = Err(err);
        }
    };

    if let Err(e) = TransactionJobModel::update_status(conn, job_id, status, result_payload).await {
        tracing::error!(
            target: "transactions",
            "FATAL: Failed to update transaction job status in DB: {}",
            e
        );
    }

    // Send the result back to the original caller (API handler)
    if result_sender.send(final_result).is_err() {
        tracing::error!(
            target: "transactions",
            "Failed to send transaction result back to caller: channel closed"
        );
    }
}

impl<C: ConnectionTrait> TransactionManager<C> {
    pub fn new(
        conn: C,
        receiver: mpsc::Receiver<ScheduledTransactionTask>,
        stats: Arc<RwLock<TransactionManagerStats>>,
    ) -> Self {
        TransactionManager {
            conn,
            receiver,
            pending_jobs: BinaryHeap::new(),
            stats,
        }
    }

    pub async fn run(&mut self, mut shutdown_rx: mpsc::Receiver<()>) -> Result<()> {
        loop {
            // Process any jobs that are currently due
            while let Some(Reverse(job)) = self.pending_jobs.peek() {
                if job.process_after <= std::time::Instant::now() {
                    let Reverse(job) = self.pending_jobs.pop().unwrap(); // Pop it from the heap
                    process_single_transaction_job(&self.conn, job).await;

                    // Update stats
                    let mut stats = self.stats.write().unwrap();
                    stats.processed_jobs_count += 1;
                    stats.pending_jobs_count = self.pending_jobs.len();
                } else {
                    break; // Top job is not yet due, break and wait
                }
            }

            // Determine when the next job is due or if we should wait for a new job
            let sleep_duration = if let Some(Reverse(next_job)) = self.pending_jobs.peek() {
                next_job
                    .process_after
                    .duration_since(std::time::Instant::now())
            } else {
                // sleep for some time: 5 minutes
                Duration::from_secs(5 * 60)
            };

            tokio::select! {
                // Listen for shutdown signal
                _ = shutdown_rx.recv() => {
                    tracing::info!("TransactionManager received shutdown signal. Exiting.");
                    // TODO: Persist pending_jobs to DB before shutting down
                    break;
                }

                // Wait for a new job to be pushed onto the channel
                Some(job) = self.receiver.recv() => {
                    // Check for immediate processing if delay is zero
                    if job.delay == Duration::ZERO {
                        // process immediately so that we dont waste cpu cycles scheduling only to be woken up immediately
                        process_single_transaction_job(&self.conn, job).await;
                        // Update stats
                        let mut stats = self.stats.write().unwrap();
                        stats.processed_jobs_count += 1;
                    } else {
                        // Schedule
                        self.pending_jobs.push(std::cmp::Reverse(job));
                        // Update stats
                        let mut stats = self.stats.write().unwrap();
                        stats.pending_jobs_count = self.pending_jobs.len();
                    }
                }
                // Or sleep until the next job is due
                _ = tokio::time::sleep(sleep_duration) => {
                    // This branch will trigger when sleep_duration elapses.
                    // The loop will then re-evaluate pending_jobs.
                    tracing::trace!(target: "transactions", "Transaction processing woke up after: {} ms", sleep_duration.as_millis());
                }
            }
        }
        Ok(())
    }
}

impl AppContext {
    /// Submits a new transaction job for scheduled processing
    pub async fn transfer(
        &self,
        identifiers: Identifiers,
        delay: Duration,
        source: Option<u32>,
        destination: u32,
        amount: i64,
        txn_type: TransactionType,
        notes: Option<TransactionNote>,
    ) -> Result<Result<(Transaction, Vec<DomainEvent>), TransactionEngineError>> {
        self.transfer_atomic(
            &self.db,
            identifiers,
            delay,
            source,
            destination,
            amount,
            txn_type,
            notes,
        )
        .await
    }

    /// Atomic version of transfer that is supposed to be passed a db transaction instead of bare connection
    pub async fn transfer_atomic<C: ConnectionTrait>(
        &self,
        conn: &C,
        identifiers: Identifiers,
        delay: Duration,
        source: Option<u32>,
        destination: u32,
        amount: i64,
        txn_type: TransactionType,
        notes: Option<TransactionNote>,
    ) -> Result<Result<(Transaction, Vec<DomainEvent>), TransactionEngineError>> {
        let process_after_utc = Utc::now() + chrono::Duration::from_std(delay).unwrap();

        let payload = JobPayload {
            source,
            destination,
            amount,
            txn_type: txn_type.clone(),
            notes: notes.clone(),
        };

        let db_job: TransactionJobModel =
            TransactionJobModel::create(conn, &identifiers, process_after_utc, payload)
                .await
                .context("Failed to create initial transaction job DB record")?
                .try_into()?;

        // Create the oneshot channel for the reply
        let (reply_sender, reply_receiver) = oneshot::channel();

        // Create the in-memory job to send to the manager
        let job = ScheduledTransactionTask {
            process_after: std::time::Instant::now() + delay,
            job_id: db_job.id,
            identifiers,
            source,
            destination,
            amount,
            txn_type,
            notes,
            delay,
            reply_sender,
        };

        // Send the job to the manager
        self.transaction_tx
            .send(job)
            .await
            .context("Failed to send transaction job to manager")?;

        // Await the reply and return it
        reply_receiver
            .await
            .context("Transaction manager dropped the reply channel.")
    }
}
