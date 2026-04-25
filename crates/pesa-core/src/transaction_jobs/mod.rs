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
                is_tracked: true,
            },
            status: value.status,
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
    ) -> Result<TransactionJob, DbErr> {
        let id = Uuid::new_v4().to_string();
        let job = db::ActiveModel {
            id: Set(id.clone()),
            originator_conversation_id: Set(identifiers.originator_conversation_id.clone()),
            conversation_id: Set(identifiers.conversation_id.clone()),
            transaction_id: Set(identifiers.transaction_id.clone()),
            request_id: Set(identifiers.request_id.clone()),
            status: Set(TransactionStatus::Pending),
            process_after: Set(process_after),
            payload: Set(serde_json::to_value(payload).map_err(|e| DbErr::Json(e.to_string()))?),
            result_payload: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(None),
        };
        let _result = db::Entity::insert(job).exec(conn).await?;
        if let Some(data) = db::Entity::find_by_id(id).one(conn).await? {
            TransactionJob::try_from(data).map_err(|err| DbErr::Custom(err.to_string()))
        } else {
            Err(DbErr::RecordNotFound(
                "Failed to retrieve created transaction job".to_string(),
            ))
        }
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

    /// Finds a job by one of its unique identifiers and returns the high-level struct.
    pub async fn get_by_request_id<C: ConnectionTrait>(
        conn: &C,
        id: &str,
    ) -> anyhow::Result<Vec<Self>> {
        let models = db::Entity::find()
            .filter(crate::transaction_jobs::db::Column::RequestId.eq(id))
            .all(conn)
            .await?;

        let mut jobs = Vec::new();

        for model in models {
            jobs.push(model.try_into()?);
        }

        Ok(jobs)
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

        active_model.status = Set(status);
        active_model.result_payload = Set(result_payload.map(|p| serde_json::to_value(p).unwrap()));
        active_model.updated_at = Set(Some(Utc::now()));

        active_model
            .update(conn)
            .await
            .context("Failed to update status for job_id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request_lifecycle::{RequestLifecycleManager, RequestType};
    use crate::tests::TestDb;
    use crate::transactions::TransactionType;
    use crate::utils::identifiers::Identifiers;
    use chrono::Utc;

    async fn setup_request(db: &sea_orm::DatabaseConnection) -> String {
        let manager = RequestLifecycleManager::new(db.clone());
        let identifiers = Identifiers {
            originator_conversation_id: "orig_123".to_string(),
            conversation_id: "conv_456".to_string(),
            transaction_id: "txn_789".to_string(),
            request_id: "req_012".to_string(),
            ..Default::default()
        };

        manager
            .create_system_request(
                &identifiers,
                "test_component",
                RequestType::StkPush,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        identifiers.request_id
    }

    #[tokio::test]
    async fn test_create_job() {
        let db = TestDb::in_memory().await.unwrap();
        let request_id = setup_request(&db.conn).await;
        dbg!(&request_id);

        let identifiers = Identifiers {
            originator_conversation_id: "orig_123".to_string(),
            conversation_id: "conv_456".to_string(),
            transaction_id: "txn_789".to_string(),
            request_id,
            ..Default::default()
        };

        let payload = JobPayload {
            source: Some(1),
            destination: 2,
            amount: 1000,
            txn_type: TransactionType::SendMoney,
            notes: None,
        };

        let process_after = Utc::now();
        let result = TransactionJob::create(&db.conn, &identifiers, process_after, payload).await;

        dbg!(&result);

        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.identifiers.originator_conversation_id, "orig_123");
        assert_eq!(job.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_get_job_by_id() {
        let db = TestDb::in_memory().await.unwrap();
        let request_id = setup_request(&db.conn).await;

        let identifiers = Identifiers {
            originator_conversation_id: "orig_123".to_string(),
            conversation_id: "conv_456".to_string(),
            transaction_id: "txn_789".to_string(),
            request_id,
            ..Default::default()
        };

        let payload = JobPayload {
            source: Some(1),
            destination: 2,
            amount: 1000,
            txn_type: TransactionType::SendMoney,
            notes: None,
        };

        let process_after = Utc::now();
        let _created = TransactionJob::create(&db.conn, &identifiers, process_after, payload)
            .await
            .unwrap();

        let found = TransactionJob::get_by_id(&db.conn, "orig_123")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().identifiers.conversation_id, "conv_456");
    }

    #[tokio::test]
    async fn test_get_job_by_id_not_found() {
        let db = TestDb::in_memory().await.unwrap();

        let found = TransactionJob::get_by_id(&db.conn, "nonexistent")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_job_status() {
        let db = TestDb::in_memory().await.unwrap();
        let request_id = setup_request(&db.conn).await;

        let identifiers = Identifiers {
            originator_conversation_id: "orig_123".to_string(),
            conversation_id: "conv_456".to_string(),
            transaction_id: "txn_789".to_string(),
            request_id,
            ..Default::default()
        };

        let payload = JobPayload {
            source: Some(1),
            destination: 2,
            amount: 1000,
            txn_type: TransactionType::SendMoney,
            notes: None,
        };

        let process_after = Utc::now();
        let _created = TransactionJob::create(&db.conn, &identifiers, process_after, payload)
            .await
            .unwrap();

        let job_id = Uuid::from_str(&_created.id.to_string()).unwrap();
        let updated =
            TransactionJob::update_status(&db.conn, job_id, TransactionStatus::Completed, None)
                .await;

        assert!(updated.is_ok());
        assert_eq!(updated.unwrap().status, TransactionStatus::Completed);
    }
}
