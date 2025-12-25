use anyhow::Result;

use crate::{AppContext, business_operators::BusinessOperator};

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
) -> anyhow::Result<BusinessOperator> {
    let operator = BusinessOperator::create(
        &context.db,
        payload.username,
        payload.password,
        payload.business_id,
    )
    .await?;
    Ok(operator)
}

// Function to get all operators for a business
pub async fn get_operators_by_business(
    context: &AppContext,
    business_id: u32,
) -> Result<Vec<BusinessOperator>> {
    let operators = BusinessOperator::get_business_operators(&context.db, business_id).await?;
    Ok(operators)
}

pub async fn get_operator(
    context: &AppContext,
    operator_id: u32,
) -> Result<Option<BusinessOperator>> {
    let operator = BusinessOperator::get_operator(&context.db, operator_id).await?;
    Ok(operator)
}

// Function to delete an operator by id
pub async fn delete_operator(context: &AppContext, operator_id: u32) -> Result<bool> {
    BusinessOperator::delete_operator(&context.db, operator_id).await
}
