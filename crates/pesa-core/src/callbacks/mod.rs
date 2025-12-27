use chrono::Utc;
use sea_orm::prelude::DateTimeUtc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod db;
pub mod dispatch;
pub mod orchestrator;
pub mod response;
pub mod stk;

#[derive(
    Debug, Clone, strum::EnumString, strum::Display, Deserialize, Serialize, Default, PartialEq, Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum CallbackType {
    #[default]
    StkPush,
    B2cResult,
    C2bValidation,
    C2bConfirmation,
}

#[derive(
    Debug, Clone, strum::EnumString, strum::Display, Deserialize, Serialize, Default, PartialEq, Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum CallbackStatus {
    #[default]
    Pending,
    Delivered,
    Failed,
}

/// A struct representing a single callback log record from the database.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CallbackLog {
    pub id: u32,
    pub project_id: u32,
    pub conversation_id: String,
    pub originator_id: String,
    pub transaction_id: Option<String>,
    pub callback_url: String,
    pub callback_type: CallbackType,
    pub payload: Value,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub response_headers: Option<Value>,
    pub status: CallbackStatus,
    pub error: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

/// Parameters for creating a new callback log.
pub struct CreateCallbackParams {
    pub project_id: u32,
    pub callback_type: CallbackType,
    pub url: String,
    pub conversation_id: String,
    pub originator_id: String,
    pub payload: Value,
    pub transaction_id: Option<String>,
}

/// Represents the outcome of a dispatch attempt.
pub enum DispatchOutcome {
    Delivered {
        status_code: u16,
        headers: Value,
        body: String,
    },
    Failed {
        error_message: String,
    },
}

impl From<db::Model> for CallbackLog {
    fn from(value: db::Model) -> Self {
        Self {
            id: value.id,
            project_id: value.project_id,
            conversation_id: value.conversation_id,
            originator_id: value.originator_id,
            transaction_id: value.transaction_id,
            callback_url: value.callback_url,
            callback_type: value.callback_type.parse().unwrap_or_default(),
            payload: serde_json::from_str(&value.payload).unwrap_or_default(),
            response_status: value.response_status,
            response_body: value.response_body,
            response_headers: value
                .response_headers
                .and_then(|h| serde_json::from_str(&h).ok()),
            status: value.status.parse().unwrap_or_default(),
            error: value.error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

// Public API for interacting with Callback Logs
impl CallbackLog {
    /// Creates a new callback record in the database.
    pub async fn create<C: ConnectionTrait>(
        db: &C,
        params: CreateCallbackParams,
    ) -> Result<Self, DbErr> {
        let model = db::ActiveModel {
            project_id: Set(params.project_id),
            conversation_id: Set(params.conversation_id),
            originator_id: Set(params.originator_id),
            transaction_id: Set(params.transaction_id),
            callback_url: Set(params.url),
            callback_type: Set(params.callback_type.to_string()),
            payload: Set(serde_json::to_string(&params.payload).unwrap_or_default()),
            status: Set(CallbackStatus::Pending.to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok(model.into())
    }

    /// Finds a callback by its primary ID.
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: u32) -> Result<Option<Self>, DbErr> {
        db::Entity::find_by_id(id)
            .one(db)
            .await
            .map(|opt| opt.map(Into::into))
    }

    /// Finds all callbacks for a given project.
    pub async fn find_by_project<C: ConnectionTrait>(
        db: &C,
        project_id: u32,
    ) -> Result<Vec<Self>, DbErr> {
        db::Entity::find()
            .filter(db::Column::ProjectId.eq(project_id))
            .all(db)
            .await
            .map(|models| models.into_iter().map(Into::into).collect())
    }

    /// Updates the status of a callback after a dispatch attempt.
    pub async fn update_dispatch_status<C: ConnectionTrait>(
        &self,
        db: &C,
        outcome: DispatchOutcome,
    ) -> Result<Self, DbErr> {
        let mut model: db::ActiveModel = self.clone().into();

        match outcome {
            DispatchOutcome::Delivered {
                status_code,
                headers,
                body,
            } => {
                model.status = Set(CallbackStatus::Delivered.to_string());
                model.response_status = Set(Some(status_code as i32));
                model.response_headers =
                    Set(Some(serde_json::to_string(&headers).unwrap_or_default()));
                model.response_body = Set(Some(body));
            }
            DispatchOutcome::Failed { error_message } => {
                model.status = Set(CallbackStatus::Failed.to_string());
                model.error = Set(Some(error_message));
            }
        }
        model.updated_at = Set(Some(Utc::now().to_utc()));

        let updated_model = model.update(db).await?;
        Ok(updated_model.into())
    }
}

// Conversion from CallbackLog to ActiveModel for updates
impl From<CallbackLog> for db::ActiveModel {
    fn from(log: CallbackLog) -> Self {
        Self {
            id: Set(log.id),
            project_id: Set(log.project_id),
            conversation_id: Set(log.conversation_id),
            originator_id: Set(log.originator_id),
            transaction_id: Set(log.transaction_id),
            callback_url: Set(log.callback_url),
            callback_type: Set(log.callback_type.to_string()),
            payload: Set(serde_json::to_string(&log.payload).unwrap_or_default()),
            response_status: Set(log.response_status),
            response_body: Set(log.response_body),
            response_headers: Set(log
                .response_headers
                .map(|h| serde_json::to_string(&h).unwrap_or_default())),
            status: Set(log.status.to_string()),
            error: Set(log.error),
            created_at: Set(log.created_at),
            updated_at: Set(log.updated_at),
        }
    }
}
