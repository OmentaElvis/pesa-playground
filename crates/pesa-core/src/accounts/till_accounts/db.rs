use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "till_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub business_id: u32,
    pub till_number: u32,
    pub location_description: Option<String>,
    pub response_type: Option<String>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Relation {
    Business,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Business => Entity::belongs_to(crate::business::db::Entity)
                .from(Column::BusinessId)
                .to(crate::business::db::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::business::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
