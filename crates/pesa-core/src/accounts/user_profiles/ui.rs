use super::{db as user_profiles, User};
use crate::{
    accounts::{db as accounts, AccountType},
    transactions::Ledger,
    AppContext,
};
use anyhow::{Context, Result};
use chrono::Utc;
use fake::{faker::name::en::Name, Fake};
use rand::seq::IndexedRandom;
use sea_orm::{prelude::*, ActiveValue::Set, TransactionTrait};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct UserDetails {
    pub id: u32,
    pub name: String,
    pub pin: String,
    pub phone: String,
    pub balance: f64,
}

impl UserDetails {
    fn generate_fake_phone(existing: &mut std::collections::HashSet<String>) -> String {
        loop {
            let suffix: u64 = (10_000_000..=99_999_999).fake(); // 9-digit Safaricom-like
            let phone = format!("2547{}", suffix);
            if !existing.contains(&phone) {
                existing.insert(phone.clone());
                return phone;
            }
        }
    }

    fn generate_fake_pin() -> String {
        format!("{:04}", (0..=9999).fake::<u16>()) // 4-digit PIN
    }

    pub fn generate() -> UserDetails {
        let mut set = std::collections::HashSet::new();

        let name: String = Name().fake();
        let phone = Self::generate_fake_phone(&mut set);
        let pin = Self::generate_fake_pin();
        let balance = [1000.0, 0.0, 200.0, 420.69, 14.0]
            .choose(&mut rand::rng())
            .unwrap();

        UserDetails {
            phone,
            name,
            pin,
            id: 0,
            balance: *balance as f64,
        }
    }

    pub fn generate_users(count: u32) -> Vec<UserDetails> {
        (0..count).map(|_| Self::generate()).collect()
    }
}

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
) -> Result<u32, String> {
    let txn = ctx
        .db
        .begin()
        .await
        .map_err(|err| format!("Failed to start transaction: {}", err))?;

    let balance = (balance * 100.0).round() as i64;

    let new_account = accounts::ActiveModel {
        account_type: Set(AccountType::User.to_string()),
        balance: Set(0),
        disabled: Set(false),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let account_model = new_account
        .insert(&txn)
        .await
        .map_err(|err| format!("Failed to create new account: {}", err))?;

    let new_profile = user_profiles::ActiveModel {
        account_id: Set(account_model.id),
        name: Set(name),
        pin: Set(pin),
        phone: Set(phone),
    };

    let user_model = new_profile
        .insert(&txn)
        .await
        .map_err(|err| format!("Failed to create new profile: {}", err))?;

    Ledger::transfer(
        &txn,
        None,
        user_model.account_id,
        balance,
        &crate::transactions::TransactionType::Deposit,
    )
    .await
    .map_err(|err| {
        format!(
            "Failed to deposit funds to new account({}): {}",
            user_model.account_id, err
        )
    })?;

    txn.commit()
        .await
        .map_err(|err| format!("Failed to complete transaction: {}", err))?;

    let _ = ctx.event_manager.emit_all(
        "new_user",
        serde_json::to_value(UserDetails {
            id: user_model.account_id,
            name: user_model.name,
            pin: user_model.pin,
            phone: user_model.phone,
            balance: balance as f64 / 100.0,
        })
        .expect("Failed to convert UserDetails to serde_json value"),
    );
    Ok(account_model.id)
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
pub async fn get_user(ctx: &AppContext, user_id: u32) -> Result<Option<UserDetails>, String> {
    let user = accounts::Entity::find_by_id(user_id)
        .find_also_related(user_profiles::Entity)
        .one(&ctx.db)
        .await
        .map_err(|err| format!("Failed to read user profile {} ", err))?;

    let user_info: Option<UserDetails> = if let Some((acct, profiles)) = user {
        profiles.map(|p| UserDetails {
            id: acct.id,
            phone: p.phone,
            name: p.name,
            pin: p.pin,
            balance: acct.balance as f64 / 100.0,
        })
    } else {
        None
    };

    Ok(user_info)
}
pub async fn generate_user() -> Result<UserDetails, String> {
    let user = UserDetails::generate();
    Ok(user)
}

pub async fn generate_users(count: u32) -> Result<Vec<UserDetails>, String> {
    let users = UserDetails::generate_users(count);
    Ok(users)
}

pub async fn get_user_by_phone(ctx: &AppContext, phone: String) -> Result<Option<UserDetails>> {
    let user = User::get_user_by_phone(&ctx.db, &phone)
        .await
        .context("Failed to read account from db")?;

    if user.is_none() {
        return Ok(None);
    }

    let user = user.unwrap();

    let account = accounts::Entity::find_by_id(user.account_id)
        .one(&ctx.db)
        .await
        .context("Failed to read user profile")?;

    if let Some(account) = account {
        Ok(Some(UserDetails {
            id: account.id,
            name: user.name,
            pin: user.pin,
            phone: user.phone,
            balance: account.balance as f64 / 100.0,
        }))
    } else {
        Ok(None)
    }
}

pub async fn update_user(
    ctx: &AppContext,
    user_id: u32,
    name: Option<String>,
    balance: Option<i64>,
    pin: Option<String>,
) -> Result<()> {
    let user = accounts::Entity::find_by_id(user_id)
        .find_also_related(user_profiles::Entity)
        .one(&ctx.db)
        .await
        .context("Failed to read user profile")?;

    if user.is_none() {
        return Ok(());
    }
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let (account, profile) = user.unwrap();
    let mut account: accounts::ActiveModel = account.into();
    if let Some(bal) = balance {
        account.balance = Set(bal);
    }

    account
        .update(&txn)
        .await
        .context("Failed to update account")?;

    let profile: Option<user_profiles::ActiveModel> = profile.map(|p| p.into());
    if let Some(mut profile) = profile {
        if let Some(name) = name {
            profile.name = Set(name);
        }

        if let Some(pin) = pin {
            profile.pin = Set(pin);
        }
        profile
            .update(&txn)
            .await
            .context("Failed to update user profile")?;
    }

    txn.commit()
        .await
        .context("Failed to complete transaction")?;
    Ok(())
}
