use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter,
};
use serde::Serialize;

pub mod db;
pub mod ui;

#[derive(Serialize)]
pub struct User {
    pub account_id: u32,
    pub phone: String,
    pub name: String,
    pub pin: String,
}

impl From<&db::Model> for User {
    fn from(value: &db::Model) -> Self {
        User {
            account_id: value.account_id,
            phone: value.phone.clone(),
            name: value.name.clone(),
            pin: value.pin.clone(),
        }
    }
}

impl User {
    pub async fn update_by_id<C>(
        conn: &C,
        user_id: u32,
        name: Option<String>,
        pin: Option<String>,
    ) -> anyhow::Result<()>
    where
        C: ConnectionTrait,
    {
        let user = db::Entity::find_by_id(user_id).one(conn).await?;

        if let Some(user) = user {
            let mut active_model: crate::accounts::user_profiles::db::ActiveModel = user.into();

            if let Some(name) = name {
                active_model.name = Set(name);
            }
            if let Some(pin) = pin {
                active_model.pin = Set(pin);
            }
            active_model.update(conn).await?;
        }

        Ok(())
    }

    pub async fn get_user_by_phone<C>(conn: &C, phone: &str) -> Result<Option<User>, DbErr>
    where
        C: ConnectionTrait,
    {
        let users = db::Entity::find()
            .filter(db::Column::Phone.eq(phone))
            .all(conn)
            .await?;

        if users.is_empty() {
            return Ok(None);
        }

        let user = users.first().unwrap();

        Ok(Some(user.into()))
    }
}
