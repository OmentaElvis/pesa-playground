use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "api_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub project_id: u32,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub created_at: DateTimeUtc,
    pub error_desc: Option<String>,
    pub duration: u64,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(crate::projects::db::Entity)
                .from(Column::ProjectId)
                .to(crate::projects::db::Column::Id)
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
