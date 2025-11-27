use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    pub business_id: u32,
    pub callback_url: Option<String>,
    pub simulation_mode: String,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Clone, Debug, EnumIter)]
pub enum Relation {
    Business,
}

impl Related<crate::business::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Business => Entity::belongs_to(crate::business::db::Entity)
                .from(Column::BusinessId)
                .to(crate::business::db::Column::Id)
                .on_delete(sea_query::ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
