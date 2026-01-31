use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transaction_jobs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    #[sea_orm(indexed)]
    pub request_id: String,
    #[sea_orm(indexed)]
    pub originator_conversation_id: String,
    #[sea_orm(indexed)]
    pub conversation_id: String,
    #[sea_orm(indexed)]
    pub transaction_id: String,
    pub status: String,
    pub process_after: DateTimeUtc,
    pub payload: Json,
    pub result_payload: Option<Json>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    Request,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Request => Entity::belongs_to(crate::request_lifecycle::db::Entity)
                .from(Column::RequestId)
                .to(crate::request_lifecycle::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::SetNull)
                .into(),
        }
    }
}

impl Related<crate::request_lifecycle::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Request.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
