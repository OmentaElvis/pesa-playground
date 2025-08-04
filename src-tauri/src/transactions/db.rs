use sea_orm::entity::prelude::*;

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
    pub transaction_type: String,
    pub status: String,
    pub reversal_of: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Transaction {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            from: value.from,
            to: value.to,
            amount: value.amount,
            fee: value.fee,
            currency: value.currency,
            status: value
                .status
                .parse()
                .unwrap_or(super::TransactionStatus::Unknown(value.status.to_string())),
            reversal_of: value.reversal_of,
            transaction_type: value
                .transaction_type
                .parse()
                .unwrap_or(super::TransactionType::Unknown(value.status.to_string())),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
