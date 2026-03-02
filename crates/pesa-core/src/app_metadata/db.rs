use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "app_metadata")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No relations defined for AppMetadata");
    }
}

impl ActiveModelBehavior for ActiveModel {}
