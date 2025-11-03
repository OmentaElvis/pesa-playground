use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::server::api::c2b::ResponseType;
pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TillAccount {
    pub account_id: u32,
    pub business_id: u32,
    pub till_number: u32,
    pub location_description: Option<String>,
    pub response_type: Option<ResponseType>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
}

impl From<&db::Model> for TillAccount {
    fn from(value: &db::Model) -> Self {
        Self {
            account_id: value.account_id,
            business_id: value.business_id,
            till_number: value.till_number,
            location_description: value.location_description.clone(),
            response_type: None,
            validation_url: None,
            confirmation_url: None,
        }
    }
}

impl TillAccount {
    pub async fn get_by_account_id<C>(
        conn: &C,
        account_id: u32,
    ) -> Result<Option<TillAccount>, DbErr>
    where
        C: ConnectionTrait,
    {
        let till = db::Entity::find_by_id(account_id).one(conn).await?;
        Ok(till.as_ref().map(|t| t.into()))
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
    pub initial_balance: i64,
    pub location_description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromQueryResult)]
pub struct TillAccountDetails {
    pub account_id: u32,
    pub business_id: u32,
    pub till_number: u32,
    pub location_description: Option<String>,
    pub response_type: Option<String>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
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
