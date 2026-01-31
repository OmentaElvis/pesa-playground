use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "request_ids")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub request_id: String,
    pub id_type: String,
    pub external_id: String,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    Request,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Request => Entity::belongs_to(super::super::db::Entity)
                .from(Column::RequestId)
                .to(super::super::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::super::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Request.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
