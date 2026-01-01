use super::User;
use crate::AppContext;
use anyhow::{Context, Result};
use sea_orm::TransactionTrait;

pub async fn get_users(ctx: &AppContext) -> anyhow::Result<Vec<User>> {
    let users = User::get_users(&ctx.db).await?;
    Ok(users)
}

pub async fn create_user(
    ctx: &AppContext,
    name: String,
    phone: String,
    balance: f64,
    pin: String,
) -> anyhow::Result<User> {
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let balance = (balance * 100.0).round() as i64;
    let user = User::create_from(&txn, phone, name, pin, balance).await?;

    txn.commit()
        .await
        .context("Failed to complete transaction")?;

    let _ = ctx.event_manager.emit_all(
        "user_created",
        serde_json::to_value(&user).context("Failed to convert UserDetails to serde_json value")?,
    );

    Ok(user)
}

pub async fn remove_user(ctx: &AppContext, user_id: u32) -> Result<()> {
    // just mark the user as deleted rather than actually yeating them.
    User::disable_user(&ctx.db, user_id).await?;
    Ok(())
}
pub async fn get_user(ctx: &AppContext, user_id: u32) -> anyhow::Result<Option<User>> {
    let user = User::find_by_id(&ctx.db, user_id).await?;
    Ok(user)
}
pub async fn generate_user() -> anyhow::Result<User> {
    let user = User::generate();
    Ok(user)
}

pub async fn generate_users(count: u32) -> anyhow::Result<Vec<User>> {
    let users = User::generate_users(count);
    Ok(users)
}

pub async fn get_user_by_phone(ctx: &AppContext, phone: String) -> Result<Option<User>> {
    let user = User::get_user_by_phone(&ctx.db, &phone)
        .await
        .context("Failed to read account from db")?;
    Ok(user)
}

pub async fn update_user(
    ctx: &AppContext,
    user_id: u32,
    name: Option<String>,
    pin: Option<String>,
    phone: Option<String>,
) -> Result<()> {
    User::update_by_id(&ctx.db, user_id, name, pin, phone).await?;
    Ok(())
}
