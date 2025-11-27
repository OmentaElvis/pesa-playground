use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "businesses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    pub short_code: String, // M-Pesa short code
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "crate::accounts::paybill_accounts::db::Entity",
        on_delete = "Cascade"
    )]
    PaybillAccounts,
    #[sea_orm(
        has_many = "crate::accounts::till_accounts::db::Entity",
        on_delete = "Cascade"
    )]
    TillAccounts,
}

impl Related<crate::accounts::paybill_accounts::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PaybillAccounts.def()
    }
}

impl Related<crate::accounts::till_accounts::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TillAccounts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
