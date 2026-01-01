use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "callback_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    /// The project this callback belongs to.
    pub project_id: u32,
    /// Our internally generated unique ID for the entire transaction flow.
    #[sea_orm(indexed)]
    pub conversation_id: String,
    /// The externally provided unique ID from the initial request (e.g., OriginatorConversationID).
    #[sea_orm(indexed)]
    pub originator_id: String,
    /// Optional M-Pesa transaction ID, if applicable.
    #[sea_orm(indexed)]
    pub transaction_id: Option<String>,
    /// The URL the callback was sent to.
    pub callback_url: String,
    /// The type of callback (e.g., "B2cResult", "StkPush").
    pub callback_type: String,
    /// The JSON payload sent in the request.
    pub payload: String,
    /// The HTTP status code received from the callback recipient.
    pub response_status: Option<i32>,
    /// The HTTP response body received from the callback recipient.
    pub response_body: Option<String>,
    /// The HTTP response headers received from the callback recipient.
    pub response_headers: Option<String>,
    /// The delivery status of the callback.
    pub status: String, // e.g., "Pending", "Delivered", "Failed"
    /// Any error message from a failed dispatch attempt.
    pub error: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::projects::db::Entity",
        from = "Column::ProjectId",
        to = "crate::projects::db::Column::Id"
    )]
    Project,
}

impl Related<crate::projects::db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
