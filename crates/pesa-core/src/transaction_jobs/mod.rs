use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::{ConnectionTrait, DbErr, Set, entity::*, prelude::DateTimeUtc, query::*};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    transaction_jobs::db::Model,
    transactions::{Transaction, TransactionNote, TransactionStatus, TransactionType},
    utils::identifiers::Identifiers,
};

pub mod db;
pub mod task;
pub mod ui;

/// Transaction arguments stored in the payload.
#[derive(Serialize, Deserialize, Debug)]
pub struct JobPayload {
    pub source: Option<u32>,
    pub destination: u32,
    pub amount: i64,
    pub txn_type: TransactionType,
    pub notes: Option<TransactionNote>,
}

/// Transaction result stored in the result_payload.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", content = "data")]
pub enum JobResultPayload {
    Completed(Transaction),
    Failed { message: String },
}

/// High-level, serializable struct representing a transaction job.
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionJob {
    pub id: Uuid,
    pub identifiers: Identifiers,
    pub status: TransactionStatus,
    pub process_after: chrono::DateTime<chrono::Utc>,
    pub payload: JobPayload,
    pub result_payload: Option<JobResultPayload>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

impl TryFrom<Model> for TransactionJob {
    type Error = anyhow::Error;

    fn try_from(value: Model) -> std::result::Result<Self, Self::Error> {
        let payload: JobPayload =
            serde_json::from_value(value.payload).context("Failed to parse job payload")?;
        let result_payload = if let Some(payload) = value.result_payload {
            Some(serde_json::from_value(payload).context("Failed to parse result payload")?)
        } else {
            None
        };

        let id = Uuid::from_str(&value.id).context("Failed to parse job id")?;

        Ok(TransactionJob {
            id,
            identifiers: Identifiers {
                originator_conversation_id: value.originator_conversation_id,
                conversation_id: value.conversation_id,
                transaction_id: value.transaction_id,
                request_id: value.request_id,
            },
            status: value.status.parse()?,
            process_after: value.process_after,
            payload,
            result_payload,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

impl TransactionJob {
    /// Creates a new transaction job record in the database.
    pub async fn create<C: ConnectionTrait>(
        conn: &C,
        identifiers: &Identifiers,
        process_after: chrono::DateTime<chrono::Utc>,
        payload: JobPayload,
    ) -> Result<db::Model, DbErr> {
        let job = db::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            originator_conversation_id: Set(identifiers.originator_conversation_id.clone()),
            conversation_id: Set(identifiers.conversation_id.clone()),
            transaction_id: Set(identifiers.transaction_id.clone()),
            request_id: Set(identifiers.request_id.clone()),
            status: Set(TransactionStatus::Pending.to_string()),
            process_after: Set(process_after),
            payload: Set(serde_json::to_value(payload).map_err(|e| DbErr::Json(e.to_string()))?),
            result_payload: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(None),
        };
        job.insert(conn).await
    }

    /// Finds a job by one of its unique identifiers and returns the high-level struct.
    pub async fn get_by_id<C: ConnectionTrait>(conn: &C, id: &str) -> anyhow::Result<Option<Self>> {
        let model = db::Entity::find()
            .filter(
                Condition::any()
                    .add(db::Column::OriginatorConversationId.eq(id))
                    .add(db::Column::TransactionId.eq(id)),
            )
            .one(conn)
            .await?;

        if let Some(model) = model {
            Ok(Some(model.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// Updates the status and result of a finished job.
    pub async fn update_status<C: ConnectionTrait>(
        conn: &C,
        job_id: Uuid,
        status: TransactionStatus,
        result_payload: Option<JobResultPayload>,
    ) -> anyhow::Result<db::Model> {
        let Some(job_model) = db::Entity::find_by_id(job_id.to_string()).one(conn).await? else {
            anyhow::bail!(DbErr::RecordNotFound(format!(
                "Transaction job {} not found for update",
                job_id
            )));
        };
        let mut active_model: db::ActiveModel = job_model.into();

        active_model.status = Set(status.to_string());
        active_model.result_payload = Set(result_payload.map(|p| serde_json::to_value(p).unwrap()));
        active_model.updated_at = Set(Some(Utc::now()));

        active_model
            .update(conn)
            .await
            .context("Failed to update status for job_id")
    }
}
