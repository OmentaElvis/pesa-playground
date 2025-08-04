use super::db::{self, Entity, Model};
use sea_orm::entity::*;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Database;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionCostData {
    pub transaction_type: String,
    pub min_amount: i64,
    pub max_amount: i64,
    pub fee_fixed: Option<i64>,
    pub fee_percentage: Option<f64>,
}

#[tauri::command]
pub async fn create_transaction_cost(
    state: State<'_, Database>,
    data: TransactionCostData,
) -> Result<Model, String> {
    let db = &state.conn;
    let new_cost = db::ActiveModel {
        transaction_type: Set(data.transaction_type),
        min_amount: Set(data.min_amount),
        max_amount: Set(data.max_amount),
        fee_fixed: Set(data.fee_fixed),
        fee_percentage: Set(data.fee_percentage),
        ..Default::default()
    };
    new_cost.insert(db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_transaction_costs(state: State<'_, Database>) -> Result<Vec<Model>, String> {
    let db = &state.conn;
    Entity::find().all(db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_transaction_cost(
    state: State<'_, Database>,
    id: i32,
    data: TransactionCostData,
) -> Result<Model, String> {
    let db = &state.conn;
    let cost = Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(cost) = cost {
        let mut cost: db::ActiveModel = cost.into();
        cost.transaction_type = Set(data.transaction_type);
        cost.min_amount = Set(data.min_amount);
        cost.max_amount = Set(data.max_amount);
        cost.fee_fixed = Set(data.fee_fixed);
        cost.fee_percentage = Set(data.fee_percentage);
        cost.update(db).await.map_err(|e| e.to_string())
    } else {
        Err("Transaction cost not found".to_string())
    }
}

#[tauri::command]
pub async fn delete_transaction_cost(state: State<'_, Database>, id: i32) -> Result<(), String> {
    let db = &state.conn;
    let cost = Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(cost) = cost {
        cost.delete(db).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Transaction cost not found".to_string())
    }
}
