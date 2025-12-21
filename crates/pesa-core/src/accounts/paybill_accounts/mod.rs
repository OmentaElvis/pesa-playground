use std::str::FromStr;

use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter};
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
