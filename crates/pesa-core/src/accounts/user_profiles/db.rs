use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: u32,
    pub name: String,
    pub phone: String,
    pub pin: String, // we will not be hashing pins since we need clear text pins for ui
}

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum Relation {
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(crate::accounts::db::Entity)
                .from(Column::AccountId)
                .to(crate::accounts::db::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::accounts::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
