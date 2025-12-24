use super::{
    db::{self, Entity, Model},
    get_fee,
};
use anyhow::{Context, Result, bail};
use sea_orm::entity::*;
use serde::{Deserialize, Serialize};

use crate::{AppContext, transactions::TransactionType};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionCostData {
    pub transaction_type: String,
    pub min_amount: i64,
    pub max_amount: i64,
    pub fee_fixed: Option<i64>,
    pub fee_percentage: Option<f64>,
}

pub async fn create_transaction_cost(ctx: &AppContext, data: TransactionCostData) -> Result<Model> {
    let db = &ctx.db;
    let new_cost = db::ActiveModel {
        transaction_type: Set(data.transaction_type),
        min_amount: Set(data.min_amount),
        max_amount: Set(data.max_amount),
        fee_fixed: Set(data.fee_fixed),
        fee_percentage: Set(data.fee_percentage),
        ..Default::default()
    };
    Ok(new_cost.insert(db).await?)
}

pub async fn list_transaction_costs(ctx: &AppContext) -> Result<Vec<Model>> {
    let db = &ctx.db;
    Ok(Entity::find().all(db).await?)
}

pub async fn update_transaction_cost(
    ctx: &AppContext,
    id: i32,
    data: TransactionCostData,
) -> Result<Model> {
    let db = &ctx.db;
    let cost = Entity::find_by_id(id).one(db).await?;
    if let Some(cost) = cost {
        let mut cost: db::ActiveModel = cost.into();
        cost.transaction_type = Set(data.transaction_type);
        cost.min_amount = Set(data.min_amount);
        cost.max_amount = Set(data.max_amount);
        cost.fee_fixed = Set(data.fee_fixed);
        cost.fee_percentage = Set(data.fee_percentage);
        Ok(cost.update(db).await?)
    } else {
        bail!("Transaction cost not found")
    }
}

pub async fn delete_transaction_cost(ctx: &AppContext, id: i32) -> Result<()> {
    let db = &ctx.db;
    let cost = Entity::find_by_id(id).one(db).await?;
    if let Some(cost) = cost {
        cost.delete(db).await?;
        Ok(())
    } else {
        bail!("Transaction cost not found")
    }
}

pub async fn calculate_transaction_fee(
    ctx: &AppContext,
    txn_type: TransactionType,
    amount: i64,
) -> Result<i64> {
    get_fee(&ctx.db, &txn_type, amount)
        .await
        .context("Failed to calculate transaction cost")
}
