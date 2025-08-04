use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "paybill_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: u32,
    pub business_id: u32,
    pub paybill_number: u32,
    pub account_validation_regex: Option<String>,
    pub validation_url: Option<String>,
    pub confirmation_url: Option<String>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::accounts::db::Entity",
        from = "Column::AccountId",
        to = "crate::accounts::db::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "crate::business::db::Entity",
        from = "Column::BusinessId",
        to = "crate::business::db::Column::Id"
    )]
    Business,
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
