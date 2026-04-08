use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::Transaction;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    pub from: Option<u32>,
    pub to: u32,
    pub amount: i64,
    pub fee: i64,
    pub currency: String,
    pub transaction_type: TransactionType,
    pub status: TransactionStatus,
    pub notes: Option<String>,
    pub reversal_of: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
    pub request_id: Option<String>,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    Request,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Request => Entity::belongs_to(crate::request_lifecycle::db::Entity)
                .from(Column::RequestId)
                .to(crate::request_lifecycle::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
        }
    }
}

impl Related<crate::request_lifecycle::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Request.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

use serde_json;

impl From<Model> for Transaction {
    fn from(value: Model) -> Self {
        let notes = value
            .notes
            .and_then(|notes_str| serde_json::from_str(&notes_str).ok());

        Self {
            id: value.id,
            from: value.from,
            to: value.to,
            amount: value.amount,
            fee: value.fee,
            currency: value.currency,
            status: value.status,
            reversal_of: value.reversal_of,
            transaction_type: value.transaction_type,
            created_at: value.created_at,
            updated_at: value.updated_at,
            notes,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, DeriveActiveEnum, EnumIter)]
#[serde(rename_all = "snake_case")]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum TransactionType {
    Paybill,
    BuyGoods,
    SendMoney,
    Airtime,
    Reversal,
    Withdraw,
    Deposit,
    ChargeSettlement,
    RevenueSweep,
    TopupUtility,
    Disbursment,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, DeriveActiveEnum, EnumIter)]
#[serde(rename_all = "snake_case")]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum TransactionStatus {
    Pending,
    Failed,
    Completed,
    Reversed,
}
