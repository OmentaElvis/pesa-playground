use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub balance: i64,
    pub account_type: String,
    pub created_at: DateTimeUtc,
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    UserProfile,
    MmfAccount,
    UtilityAccount,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::UserProfile => Entity::has_one(super::user_profiles::db::Entity)
                .from(Column::Id)
                .to(super::user_profiles::db::Column::AccountId)
                .into(),
            Self::MmfAccount => Entity::has_one(super::mmf_accounts::db::Entity)
                .from(Column::Id)
                .to(super::mmf_accounts::db::Column::AccountId)
                .into(),
            Self::UtilityAccount => Entity::has_one(super::utility_accounts::db::Entity)
                .from(Column::Id)
                .to(super::utility_accounts::db::Column::AccountId)
                .into(),
        }
    }
}

impl Related<super::user_profiles::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
