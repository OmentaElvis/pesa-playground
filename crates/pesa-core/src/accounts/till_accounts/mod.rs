use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, Set,
    prelude::DateTimeUtc,
};
use serde::{Deserialize, Serialize};

use crate::server::api::c2b::ResponseType;
pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TillAccount {
    pub id: u32,
    pub business_id: u32,
    pub till_number: u32,
    pub location_description: Option<String>,
    pub response_type: Option<ResponseType>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub created_at: DateTimeUtc,
}

impl From<&db::Model> for TillAccount {
    fn from(value: &db::Model) -> Self {
        let response_type = if let Some(t) = &value.response_type {
            ResponseType::from_str(t).ok()
        } else {
            None
        };

        Self {
            id: value.id,
            business_id: value.business_id,
            till_number: value.till_number,
            location_description: value.location_description.clone(),
            response_type,
            validation_url: value.validation_url.clone(),
            confirmation_url: value.confirmation_url.clone(),
            created_at: value.created_at,
        }
    }
}

impl TillAccount {
    pub async fn create<C>(conn: &C, input: CreateTillAccount) -> Result<TillAccount>
    where
        C: ConnectionTrait,
    {
        let new_till = db::ActiveModel {
            business_id: Set(input.business_id),
            till_number: Set(input.till_number),
            location_description: Set(input.location_description.clone()),
            response_type: Set(input.response_type.map(|res| res.to_string())),
            validation_url: Set(input.validation_url),
            confirmation_url: Set(input.confirmation_url),
            created_at: Set(Utc::now().to_utc()),
            ..Default::default()
        };

        let till_model = &new_till
            .insert(conn)
            .await
            .context("Failed to create new till number")?;
        let till: TillAccount = till_model.into();
        Ok(till)
    }
    pub async fn get_by_id<C>(conn: &C, id: u32) -> Result<TillAccount>
    where
        C: ConnectionTrait,
    {
        let till_account = &db::Entity::find_by_id(id)
            .one(conn)
            .await
            .context(format!("Failed to fetch till account with ID {}", id))?
            .ok_or_else(|| anyhow!("Till account with ID {} not found", id))?;
        Ok(till_account.into())
    }
    pub async fn get_all<C>(conn: &C) -> Result<Vec<TillAccount>>
    where
        C: ConnectionTrait,
    {
        let till_accounts = db::Entity::find()
            .all(conn)
            .await
            .context("Failed to fetch till accounts")?
            .into_iter()
            .map(|model| (&model).into())
            .collect();
        Ok(till_accounts)
    }
    pub async fn get_by_business_id<C>(conn: &C, business_id: u32) -> Result<Vec<TillAccount>>
    where
        C: ConnectionTrait,
    {
        let till_accounts = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .all(conn)
            .await
            .context(format!(
                "Failed to fetch till accounts for business {}",
                business_id
            ))?
            .into_iter()
            .map(|model| (&model).into())
            .collect();
        Ok(till_accounts)
    }
    pub async fn update<C>(
        conn: &C,
        id: u32,
        input: UpdateTillAccount,
    ) -> Result<Option<TillAccount>>
    where
        C: ConnectionTrait,
    {
        let till_account = db::Entity::find_by_id(id)
            .one(conn)
            .await
            .context(format!("Failed to fetch till account with ID {}", id))?
            .ok_or_else(|| anyhow!("Till account with ID {} not found", id))?;
        let mut active_model: db::ActiveModel = till_account.into();
        if let Some(business_id) = input.business_id {
            active_model.business_id = Set(business_id);
        }
        if let Some(till_number) = input.till_number {
            active_model.till_number = Set(till_number);
        }
        if let Some(location_description) = input.location_description {
            active_model.location_description = Set(Some(location_description.clone()));
        }
        if let Some(response_type) = input.response_type {
            active_model.response_type = Set(Some(response_type.to_string()));
        }
        if let Some(validation_url) = input.validation_url {
            active_model.validation_url = Set(Some(validation_url));
        }
        if let Some(confirmation_url) = input.confirmation_url {
            active_model.confirmation_url = Set(Some(confirmation_url));
        }
        let updated_till_account = active_model
            .update(conn)
            .await
            .context(format!("Failed to update till account {}", id))?;
        Ok(Some(TillAccount {
            id: updated_till_account.id,
            business_id: updated_till_account.business_id,
            till_number: updated_till_account.till_number,
            location_description: updated_till_account.location_description,
            response_type: updated_till_account
                .response_type
                .map(|r| r.parse().unwrap_or(ResponseType::Cancelled)),
            validation_url: updated_till_account.validation_url,
            confirmation_url: updated_till_account.confirmation_url,
            created_at: updated_till_account.created_at,
        }))
    }
    pub async fn delete<C>(conn: &C, id: u32) -> Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(id)
            .exec(conn)
            .await
            .context(format!("Failed to delete till account with ID {}", id))?;
        Ok(result.rows_affected > 0)
    }

    pub async fn get_by_till_number<C>(
        conn: &C,
        till_number: u32,
    ) -> Result<Option<TillAccount>, DbErr>
    where
        C: ConnectionTrait,
    {
        let till = db::Entity::find()
            .filter(db::Column::TillNumber.eq(till_number))
            .one(conn)
            .await?;
        Ok(till.as_ref().map(|t| t.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTillAccount {
    pub business_id: u32,
    pub till_number: u32,
    pub response_type: Option<ResponseType>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub location_description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateTillAccount {
    pub business_id: Option<u32>,
    pub till_number: Option<u32>,
    pub location_description: Option<String>,
    pub response_type: Option<ResponseType>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
}
