use std::str::FromStr;

use sea_orm::{
    prelude::DateTimeUtc, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
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
