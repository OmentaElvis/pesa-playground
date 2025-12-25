use anyhow::Result;
use sea_orm::{ActiveValue::Set, entity::*, query::*};

use crate::{AppContext, business_operators};

// Input payload for creating an operator
#[derive(Debug, serde::Deserialize)]
pub struct CreateOperatorPayload {
    pub username: String,
    pub password: String,
    pub business_id: u32,
}

// Function to create a new operator for a specific business
pub async fn create_operator(
    context: &AppContext,
    payload: CreateOperatorPayload,
) -> anyhow::Result<business_operators::db::Model> {
    let new_operator = business_operators::db::ActiveModel {
        username: Set(payload.username),
        password: Set(payload.password),
        business_id: Set(payload.business_id),
        ..Default::default()
    };
    let operator = new_operator.insert(&context.db).await?;
    Ok(operator)
}

// Function to get all operators for a business
pub async fn get_operators_by_business(
    context: &AppContext,
    business_id: u32,
) -> Result<Vec<business_operators::db::Model>> {
    let operators = business_operators::db::Entity::find()
        .filter(business_operators::db::Column::BusinessId.eq(business_id))
        .all(&context.db)
        .await?;
    Ok(operators)
}

// Function to delete an operator by id
pub async fn delete_operator(context: &AppContext, operator_id: u32) -> Result<bool> {
    let result = business_operators::db::Entity::delete_by_id(operator_id)
        .exec(&context.db)
        .await?;
    Ok(result.rows_affected > 0)
}
