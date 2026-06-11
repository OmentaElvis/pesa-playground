use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub source_type: SourceType,
    pub source_api_key_id: Option<u32>,
    pub source_operator_id: Option<u32>,
    pub source_component: Option<String>,
    pub request_type: RequestType,
    pub request_status: RequestStatus,
    pub created_at: DateTimeUtc,
    pub started_at: Option<DateTimeUtc>,
    pub completed_at: Option<DateTimeUtc>,
    pub project_id: Option<u32>,
    pub business_id: Option<u32>,
    pub user_id: Option<u32>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    SourceApiKey,
    SourceOperator,
    Project,
    Business,
    User,
    RequestIds,
    ApiLogs,
    Transactions,
    CallbackLogs,
    TransactionJobs,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::SourceApiKey => Entity::belongs_to(crate::api_keys::db::Entity)
                .from(Column::SourceApiKeyId)
                .to(crate::api_keys::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
            Self::SourceOperator => Entity::belongs_to(crate::business_operators::db::Entity)
                .from(Column::SourceOperatorId)
                .to(crate::business_operators::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
            Self::Project => Entity::belongs_to(crate::projects::db::Entity)
                .from(Column::ProjectId)
                .to(crate::projects::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
            Self::Business => Entity::belongs_to(crate::business::db::Entity)
                .from(Column::BusinessId)
                .to(crate::business::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
            Self::User => Entity::belongs_to(crate::accounts::user_profiles::db::Entity)
                .from(Column::UserId)
                .to(crate::accounts::user_profiles::db::Column::AccountId)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
            Self::RequestIds => Entity::has_many(super::request_ids::db::Entity).into(),
            Self::ApiLogs => Entity::has_many(crate::api_logs::db::Entity).into(),
            Self::Transactions => Entity::has_many(crate::transactions::db::Entity).into(),
            Self::CallbackLogs => Entity::has_many(crate::callbacks::db::Entity).into(),
            Self::TransactionJobs => Entity::has_many(crate::transaction_jobs::db::Entity).into(),
        }
    }
}

impl Related<crate::api_keys::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SourceApiKey.def()
    }
}

impl Related<crate::business_operators::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SourceOperator.def()
    }
}

impl Related<crate::projects::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<crate::business::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl Related<crate::accounts::user_profiles::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::request_ids::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RequestIds.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum RequestStatus {
    Completed,
    Failed,
    Pending,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum RequestType {
    StkPush,
    C2bRegisterUrl,
    B2cPayment,
    BalanceQuery,
    C2bLipa,
    Oauth,
    TransactionStatus,
    DynamicQr,
    Reversal,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, DeriveActiveEnum, EnumIter)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum SourceType {
    Api,
    System,
    Ui,
}
