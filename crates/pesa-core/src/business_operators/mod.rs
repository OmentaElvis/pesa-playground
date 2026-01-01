use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult,
    QueryFilter,
};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod ui;

#[derive(FromQueryResult, Debug, Serialize, Deserialize, Clone)]
pub struct BusinessOperator {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub business_id: u32,
}

impl From<db::Model> for BusinessOperator {
    fn from(value: db::Model) -> Self {
        BusinessOperator {
            id: value.id,
            username: value.username,
            password: value.password,
            business_id: value.business_id,
        }
    }
}

impl BusinessOperator {
    pub async fn create<C>(
        db: &C,
        username: String,
        password: String,
        business_id: u32,
    ) -> anyhow::Result<Self>
    where
        C: ConnectionTrait,
    {
        let new_operator = db::ActiveModel {
            username: Set(username),
            password: Set(password),
            business_id: Set(business_id),
            ..Default::default()
        };
        let operator = new_operator.insert(db).await?;

        Ok(operator.into())
    }

    pub async fn find_by_business<C>(
        db: &C,
        username: String,
        business_id: u32,
    ) -> anyhow::Result<Option<Self>>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .filter(db::Column::Username.eq(username))
            .into_model::<BusinessOperator>()
            .one(db)
            .await?;

        Ok(model)
    }

    pub async fn get_business_operators<C>(
        db: &C,
        business_id: u32,
    ) -> anyhow::Result<Vec<BusinessOperator>>
    where
        C: ConnectionTrait,
    {
        let models = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .into_model::<BusinessOperator>()
            .all(db)
            .await?;

        Ok(models)
    }

    pub async fn get_operator<C>(db: &C, operator: u32) -> anyhow::Result<Option<BusinessOperator>>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find_by_id(operator)
            .into_model::<BusinessOperator>()
            .one(db)
            .await?;

        Ok(model)
    }

    pub async fn delete_operator<C>(db: &C, operator: u32) -> anyhow::Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(operator).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}
