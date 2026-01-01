use anyhow::Result;
use sea_orm::TransactionTrait;

use crate::AppContext;

use crate::business::{Business, BusinessSummary, CreateBusiness, UpdateBusiness};

pub async fn create_business(ctx: &AppContext, input: CreateBusiness) -> Result<Business> {
    let txn = ctx.db.begin().await?;

    let created_business = Business::create(&txn, input).await?;

    txn.commit().await?;

    Ok(created_business)
}
pub async fn get_business(ctx: &AppContext, id: u32) -> Result<BusinessSummary> {
    Business::get_summary(&ctx.db, id).await
}
pub async fn get_businesses(ctx: &AppContext) -> Result<Vec<Business>> {
    Business::get_all(&ctx.db).await
}
pub async fn update_business(
    ctx: &AppContext,
    id: u32,
    input: UpdateBusiness,
) -> Result<Option<Business>> {
    Business::update(&ctx.db, id, input).await
}
pub async fn delete_business(ctx: &AppContext, id: u32) -> Result<bool> {
    Business::delete(&ctx.db, id).await
}

pub async fn revenue_settlement(ctx: &AppContext, business_id: u32) -> Result<()> {
    let txn = ctx.db.begin().await?;
    Business::settle_revenue(&txn, business_id).await?;
    txn.commit().await?;
    Ok(())
}
