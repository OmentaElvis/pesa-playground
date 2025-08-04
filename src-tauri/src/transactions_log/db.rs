use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transactions_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_id: String,
    pub account_id: i32,
    pub direction: Direction,
    pub new_balance: i64,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "direction")]
pub enum Direction {
    #[sea_orm(string_value = "DEBIT")]
    Debit,
    #[sea_orm(string_value = "CREDIT")]
    Credit,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::transactions::db::Entity",
        from = "Column::TransactionId",
        to = "super::super::transactions::db::Column::Id"
    )]
    Transaction,
    #[sea_orm(
        belongs_to = "super::super::accounts::db::Entity",
        from = "Column::AccountId",
        to = "super::super::accounts::db::Column::Id"
    )]
    Account,
}

impl ActiveModelBehavior for ActiveModel {}
