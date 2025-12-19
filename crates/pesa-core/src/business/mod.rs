use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::accounts::{mmf_accounts::MmfAccount, utility_accounts::UtilityAccount};
pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Business {
    pub id: u32,
    pub name: String,
    pub short_code: String,
    pub charges_amount: i64,
}
impl From<&db::Model> for Business {
    fn from(value: &db::Model) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            short_code: value.short_code.clone(),
            charges_amount: value.charges_amount,
        }
    }
}

impl Business {
    pub async fn get_by_id<C>(conn: &C, id: u32) -> Result<Option<Business>, DbErr>
    where
        C: ConnectionTrait,
    {
        let business = db::Entity::find_by_id(id).one(conn).await?;
        Ok(business.as_ref().map(|b| b.into()))
    }

    pub async fn get_by_short_code<C>(conn: &C, short_code: &str) -> Result<Option<Business>, DbErr>
    where
        C: ConnectionTrait,
    {
        let business = db::Entity::find()
            .filter(db::Column::ShortCode.eq(short_code))
            .one(conn)
            .await?;
        Ok(business.as_ref().map(|b| b.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateBusiness {
    pub name: String,
    pub short_code: String,
    pub initial_working_balance: f64,
    pub initial_utility_balance: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateBusiness {
    pub name: Option<String>,
    pub short_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessDetails {
    pub id: u32,
    pub name: String,
    pub short_code: String,
    pub mmf_account_id: u32,
    pub utility_account_id: u32,
    pub charges_amount: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusinessSummary {
    pub id: u32,
    pub name: String,
    pub short_code: String,

    pub mmf_account: MmfAccount,
    pub utility_account: UtilityAccount,
    pub charges_amount: i64,
}
