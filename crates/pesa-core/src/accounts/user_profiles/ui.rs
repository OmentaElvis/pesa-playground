use super::{User, db as user_profiles};
use crate::{AppContext, accounts::db as accounts};
use anyhow::{Context, Result};
use sea_orm::{ActiveValue::Set, TransactionTrait, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct UserDetails {
    pub id: u32,
    pub name: String,
    pub pin: String,
    pub phone: String,
    pub balance: f64,
}

impl UserDetails {}

pub async fn get_users(ctx: &AppContext) -> Result<Vec<UserDetails>, String> {
    let users = accounts::Entity::find()
        .filter(accounts::Column::AccountType.eq(crate::accounts::AccountType::User.to_string()))
        .find_also_related(user_profiles::Entity)
        .all(&ctx.db)
        .await
        .map_err(|err| format!("Failed to read user profiles {} ", err))?;

    let users_info = users
        .into_iter()
        .filter_map(|(acct, profiles)| {
            profiles.map(|p| UserDetails {
                id: acct.id,
                phone: p.phone,
                name: p.name,
                pin: p.pin,
                balance: acct.balance as f64 / 100.0,
            })
        })
        .collect();

    Ok(users_info)
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

pub async fn remove_user(ctx: &AppContext, user_id: u32) -> Result<(), String> {
    // just mark the user as deleted rather than actually yeating them.
    let account = accounts::Entity::find_by_id(user_id)
        .one(&ctx.db)
        .await
        .map_err(|err| format!("Failed to get user ({}): {}", user_id, err))?;

    let mut account: accounts::ActiveModel = account
        .ok_or_else(|| format!("User ({}) not found.", user_id))?
        .into();
    account.disabled = Set(true);
    account
        .update(&ctx.db)
        .await
        .map_err(|err| format!("Failed to update user account: {}", err))?;

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
