use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transaction_costs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_type: String,
    pub min_amount: i64,
    pub max_amount: i64,
    pub fee_fixed: Option<i64>,
    pub fee_percentage: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
