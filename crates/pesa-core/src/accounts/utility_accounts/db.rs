use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "utility_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: u32,
    pub business_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
    Business,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(crate::accounts::db::Entity)
                .from(Column::AccountId)
                .to(crate::accounts::db::Column::Id)
                .into(),
            Self::Business => Entity::belongs_to(crate::business::db::Entity)
                .from(Column::BusinessId)
                .to(crate::business::db::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::accounts::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<crate::business::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Business.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
