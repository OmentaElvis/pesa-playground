use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub project_id: u32,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Clone, EnumIter)]
pub enum Relation {
    Project,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(crate::projects::db::Entity)
                .from(Column::ProjectId)
                .to(crate::projects::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<crate::projects::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
