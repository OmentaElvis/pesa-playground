use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "callback_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub transaction_id: Option<String>,
    pub checkout_request_id: Option<String>,
    pub merchant_request_id: Option<String>,
    pub callback_url: String,
    pub callback_type: String, // e.g. "stkpush", "c2b"
    pub payload: String,       // raw JSON
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub status: String, // e.g. "delivered", "failed"
    pub error: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Transactions,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Transactions => Entity::belongs_to(crate::transactions::db::Entity)
                .from(Column::TransactionId)
                .to(crate::transactions::db::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::transactions::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
