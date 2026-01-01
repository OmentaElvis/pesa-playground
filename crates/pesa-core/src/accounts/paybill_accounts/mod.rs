use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

use crate::server::api::c2b::ResponseType;
pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaybillAccount {
    pub id: u32,
    pub business_id: u32,
    pub paybill_number: u32,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub response_type: Option<ResponseType>,
}

impl From<&db::Model> for PaybillAccount {
    fn from(value: &db::Model) -> Self {
        let response_type = if let Some(t) = &value.response_type {
            ResponseType::from_str(t).ok()
        } else {
            None
        };

        Self {
            id: value.id,
            business_id: value.business_id,
            paybill_number: value.paybill_number,
            validation_url: value.validation_url.clone(),
            confirmation_url: value.confirmation_url.clone(),
            response_type,
        }
    }
}

impl PaybillAccount {
    pub async fn create<C>(conn: &C, input: CreatePaybillAccount) -> Result<PaybillAccount>
    where
        C: ConnectionTrait,
    {
        let new_paybill = db::ActiveModel {
            business_id: Set(input.business_id),
            paybill_number: Set(input.paybill_number),
            response_type: Set(input.response_type.map(|res| res.to_string())),
            validation_url: Set(input.validation_url),
            confirmation_url: Set(input.confirmation_url),
            created_at: Set(Utc::now().to_utc()),
            ..Default::default()
        };

        let paybill_model = &new_paybill
            .insert(conn)
            .await
            .context("Failed to create new paybill account")?;

        let paybill: PaybillAccount = paybill_model.into();

        Ok(paybill)
    }

    pub async fn get_all<C>(conn: &C) -> Result<Vec<PaybillAccountDetails>>
    where
        C: ConnectionTrait,
    {
        let paybill_accounts = db::Entity::find()
            .into_model::<PaybillAccountDetails>()
            .all(conn)
            .await
            .context("Failed to fetch paybill accounts")?;

        Ok(paybill_accounts)
    }

    pub async fn get_by_id<C>(conn: &C, id: u32) -> Result<PaybillAccountDetails>
    where
        C: ConnectionTrait,
    {
        let paybill_account = db::Entity::find_by_id(id)
            .into_model::<PaybillAccountDetails>()
            .one(conn)
            .await
            .context(format!("Failed to fetch paybill account with ID {}", id))?
            .ok_or_else(|| anyhow::anyhow!("Paybill account with ID {} not found", id))?;

        Ok(paybill_account)
    }

    pub async fn get_by_business_id<C>(
        conn: &C,
        business_id: u32,
    ) -> Result<Vec<PaybillAccountDetails>>
    where
        C: ConnectionTrait,
    {
        let paybill_accounts = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .into_model::<PaybillAccountDetails>()
            .all(conn)
            .await
            .context(format!(
                "Failed to fetch paybill accounts for business {}",
                business_id
            ))?;

        Ok(paybill_accounts)
    }

    pub async fn update<C>(
        conn: &C,
        id: u32,
        input: UpdatePaybillAccount,
    ) -> Result<Option<PaybillAccount>>
    where
        C: ConnectionTrait,
    {
        let paybill_account = db::Entity::find_by_id(id)
            .one(conn)
            .await
            .context(format!("Failed to fetch paybill account with ID {}", id))?
            .ok_or_else(|| anyhow::anyhow!("Paybill account with ID {} not found", id))?;

        let mut active_model: db::ActiveModel = paybill_account.into();

        if let Some(business_id) = input.business_id {
            active_model.business_id = Set(business_id);
        }
        if let Some(paybill_number) = input.paybill_number {
            active_model.paybill_number = Set(paybill_number);
        }
        if let Some(validation_url) = input.validation_url {
            active_model.validation_url = Set(Some(validation_url));
        }
        if let Some(confirmation_url) = input.confirmation_url {
            active_model.confirmation_url = Set(Some(confirmation_url));
        }
        if let Some(response_type) = &input.response_type {
            active_model.response_type = Set(Some(response_type.to_string()));
        }

        let updated_paybill_account = active_model
            .update(conn)
            .await
            .context(format!("Failed to update paybill account {}", id))?;

        Ok(Some(PaybillAccount {
            id: updated_paybill_account.id,
            business_id: updated_paybill_account.business_id,
            paybill_number: updated_paybill_account.paybill_number,
            validation_url: updated_paybill_account.validation_url,
            confirmation_url: updated_paybill_account.confirmation_url,
            response_type: input.response_type,
        }))
    }
    pub async fn delete<C>(conn: &C, id: u32) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(id)
            .exec(conn)
            .await
            .context(format!("Failed to delete paybill account with ID {}", id))?;

        Ok(result.rows_affected > 0)
    }

    pub async fn get_by_paybill_number<C>(
        conn: &C,
        paybill_number: u32,
    ) -> Result<Option<PaybillAccount>, DbErr>
    where
        C: ConnectionTrait,
    {
        let paybill = db::Entity::find()
            .filter(db::Column::PaybillNumber.eq(paybill_number))
            .one(conn)
            .await?;
        Ok(paybill.as_ref().map(|p| p.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatePaybillAccount {
    pub business_id: u32,
    pub paybill_number: u32,
    pub response_type: Option<ResponseType>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct PaybillAccountDetails {
    pub id: u32,
    pub business_id: u32,
    pub paybill_number: u32,
    pub response_type: Option<String>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdatePaybillAccount {
    pub business_id: Option<u32>,
    pub paybill_number: Option<u32>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub response_type: Option<ResponseType>,
}
