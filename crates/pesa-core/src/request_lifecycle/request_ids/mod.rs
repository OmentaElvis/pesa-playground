use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod ui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId {
    pub id: u32,
    pub request_id: String,
    pub id_type: String,
    pub external_id: String,
}

impl From<db::Model> for RequestId {
    fn from(value: db::Model) -> Self {
        Self {
            id: value.id,
            request_id: value.request_id,
            id_type: value.id_type,
            external_id: value.external_id,
        }
    }
}

impl RequestId {
    pub async fn create<C: ConnectionTrait>(
        db: &C,
        request_id: String,
        id_type: String,
        external_id: String,
    ) -> anyhow::Result<Self> {
        let model = db::ActiveModel {
            request_id: Set(request_id),
            id_type: Set(id_type),
            external_id: Set(external_id),
            ..Default::default()
        };

        let inserted = model.insert(db).await?;
        Ok(inserted.into())
    }

    pub async fn find_by_request_id<C: ConnectionTrait>(
        db: &C,
        request_id: String,
    ) -> anyhow::Result<Vec<Self>> {
        let records = db::Entity::find()
            .filter(db::Column::RequestId.eq(request_id))
            .all(db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(records)
    }
}
