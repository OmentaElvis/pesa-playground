use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub phone: String,
    pub pin: String,
}

impl From<crate::accounts::user_profiles::db::Model> for User {
    fn from(model: crate::accounts::user_profiles::db::Model) -> Self {
        User {
            id: model.account_id,
            name: model.name,
            phone: model.phone,
            pin: model.pin,
        }
    }
}

impl User {
    pub async fn get_user_by_phone(
        db: &DatabaseConnection,
        phone: String,
    ) -> anyhow::Result<Option<User>> {
        let user = crate::accounts::user_profiles::db::Entity::find()
            .filter(crate::accounts::user_profiles::db::Column::Phone.eq(phone))
            .one(db)
            .await?;

        Ok(user.map(|u| u.into()))
    }

    pub async fn update_by_id(
        db: &DatabaseConnection,
        user_id: u32,
        name: Option<String>,
        pin: Option<String>,
    ) -> anyhow::Result<()> {
        let user = crate::accounts::user_profiles::db::Entity::find_by_id(user_id)
            .one(db)
            .await?;

        if let Some(user) = user {
            let mut active_model: crate::accounts::user_profiles::db::ActiveModel = user.into();

            if let Some(name) = name {
                active_model.name = Set(name);
            }
            if let Some(pin) = pin {
                active_model.pin = Set(pin);
            }
            active_model.update(db).await?;
        }

        Ok(())
    }
}
