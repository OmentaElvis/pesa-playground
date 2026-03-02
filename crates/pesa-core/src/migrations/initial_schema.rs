use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum Accounts {
    Table,
    Id,
    Balance,
    AccountType,
    CreatedAt,
    Disabled,
}

#[derive(Iden)]
enum Businesses {
    Table,
    Id,
    Name,
    ShortCode,
    ChargesAmount,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
    Name,
    BusinessId,
    CallbackUrl,
    SimulationMode,
    StkDelay,
    Prefix,
    CreatedAt,
}

#[derive(Iden)]
enum ApiKeys {
    Table,
    Id,
    ProjectId,
    ConsumerKey,
    ConsumerSecret,
    Passkey,
    CreatedAt,
}

#[derive(Iden)]
enum ApiLogs {
    Table,
    Id,
    ProjectId,
    Method,
    Path,
    StatusCode,
    RequestBody,
    ResponseBody,
    CreatedAt,
    ErrorDesc,
    Duration,
    RequestId,
}

#[derive(Iden)]
enum AccessTokens {
    Table,
    Token,
    ProjectId,
    ExpiresAt,
    CreatedAt,
}

#[derive(Iden)]
enum BusinessOperators {
    Table,
    Id,
    Username,
    Password,
    BusinessId,
}

#[derive(Iden)]
enum UserProfiles {
    Table,
    AccountId,
    Name,
    Phone,
    Pin,
    Imsi,
    RegisteredAt,
    LastSwapDate,
}

#[derive(Iden)]
enum MmfAccounts {
    Table,
    AccountId,
    BusinessId,
}

#[derive(Iden)]
enum PaybillAccounts {
    Table,
    Id,
    BusinessId,
    PaybillNumber,
    ResponseType,
    ValidationUrl,
    ConfirmationUrl,
    CreatedAt,
}

#[derive(Iden)]
enum TillAccounts {
    Table,
    Id,
    BusinessId,
    TillNumber,
    LocationDescription,
    ResponseType,
    ValidationUrl,
    ConfirmationUrl,
    CreatedAt,
}

#[derive(Iden)]
enum UtilityAccounts {
    Table,
    AccountId,
    BusinessId,
}

#[derive(Iden)]
enum Transactions {
    Table,
    Id,
    From,
    To,
    Amount,
    Fee,
    Currency,
    TransactionType,
    Status,
    Notes,
    ReversalOf,
    CreatedAt,
    UpdatedAt,
    RequestId,
}

#[derive(Iden)]
enum TransactionCosts {
    Table,
    Id,
    TransactionType,
    MinAmount,
    MaxAmount,
    FeeFixed,
    FeePercentage,
}

#[derive(Iden)]
enum CallbackLogs {
    Table,
    Id,
    ProjectId,
    ConversationId,
    OriginatorId,
    TransactionId,
    CallbackUrl,
    CallbackType,
    Payload,
    ResponseStatus,
    ResponseBody,
    ResponseHeaders,
    Status,
    Error,
    CreatedAt,
    UpdatedAt,
    RequestId,
}

#[derive(Iden)]
enum TransactionsLog {
    Table,
    Id,
    TransactionId,
    AccountId,
    Direction,
    NewBalance,
}

#[derive(Iden)]
enum TransactionJobs {
    Table,
    Id,
    RequestId,
    OriginatorConversationId,
    ConversationId,
    TransactionId,
    Status,
    ProcessAfter,
    Payload,
    ResultPayload,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Requests {
    Table,
    Id,
    SourceType,
    SourceApiKeyId,
    SourceOperatorId,
    SourceComponent,
    RequestType,
    RequestStatus,
    CreatedAt,
    StartedAt,
    CompletedAt,
    ProjectId,
    BusinessId,
    UserId,
    RequestBody,
    ResponseBody,
    ErrorMessage,
}

#[derive(Iden)]
enum RequestIds {
    Table,
    Id,
    RequestId,
    IdType,
    ExternalId,
}

#[derive(Iden)]
enum AppMetadata {
    Table,
    Key,
    Value,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    pub fn accounts_table() -> TableCreateStatement {
        Table::create()
            .table(Accounts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Accounts::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Accounts::Balance).big_integer().not_null())
            .col(ColumnDef::new(Accounts::AccountType).string().not_null())
            .col(
                ColumnDef::new(Accounts::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(Accounts::Disabled).boolean().not_null())
            .to_owned()
    }

    pub fn businesses_table() -> TableCreateStatement {
        Table::create()
            .table(Businesses::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Businesses::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Businesses::Name).string().not_null())
            .col(ColumnDef::new(Businesses::ShortCode).string().not_null())
            .col(
                ColumnDef::new(Businesses::ChargesAmount)
                    .big_integer()
                    .not_null(),
            )
            .to_owned()
    }

    pub fn businesses_short_code_unique_index() -> IndexCreateStatement {
        Index::create()
            .unique()
            .name("idx-businesses-short_code")
            .table(Businesses::Table)
            .col(Businesses::ShortCode)
            .to_owned()
    }

    pub fn projects_table() -> TableCreateStatement {
        Table::create()
            .table(Projects::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Projects::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Projects::Name).string().not_null())
            .col(ColumnDef::new(Projects::BusinessId).integer().not_null())
            .col(ColumnDef::new(Projects::CallbackUrl).string().null())
            .col(ColumnDef::new(Projects::SimulationMode).string().not_null())
            .col(ColumnDef::new(Projects::StkDelay).integer().not_null())
            .col(ColumnDef::new(Projects::Prefix).string().null())
            .col(
                ColumnDef::new(Projects::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Projects::Table, Projects::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn api_keys_table() -> TableCreateStatement {
        Table::create()
            .table(ApiKeys::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ApiKeys::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(ApiKeys::ProjectId).integer().not_null())
            .col(ColumnDef::new(ApiKeys::ConsumerKey).string().not_null())
            .col(ColumnDef::new(ApiKeys::ConsumerSecret).string().not_null())
            .col(ColumnDef::new(ApiKeys::Passkey).string().not_null())
            .col(
                ColumnDef::new(ApiKeys::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ApiKeys::Table, ApiKeys::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn api_logs_table() -> TableCreateStatement {
        Table::create()
            .table(ApiLogs::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ApiLogs::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(ApiLogs::ProjectId).integer().not_null())
            .col(ColumnDef::new(ApiLogs::Method).string().not_null())
            .col(ColumnDef::new(ApiLogs::Path).string().not_null())
            .col(
                ColumnDef::new(ApiLogs::StatusCode)
                    .small_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(ApiLogs::RequestBody).string().null())
            .col(ColumnDef::new(ApiLogs::ResponseBody).string().null())
            .col(
                ColumnDef::new(ApiLogs::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(ApiLogs::ErrorDesc).string().null())
            .col(ColumnDef::new(ApiLogs::Duration).integer().not_null())
            .col(ColumnDef::new(ApiLogs::RequestId).string().null())
            .foreign_key(
                ForeignKey::create()
                    .from(ApiLogs::Table, ApiLogs::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ApiLogs::Table, ApiLogs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn api_logs_request_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-api-logs-request-id")
            .table(ApiLogs::Table)
            .col(ApiLogs::RequestId)
            .to_owned()
    }

    pub fn access_tokens_table() -> TableCreateStatement {
        Table::create()
            .table(AccessTokens::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(AccessTokens::Token)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(AccessTokens::ProjectId).integer().not_null())
            .col(
                ColumnDef::new(AccessTokens::ExpiresAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(AccessTokens::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(AccessTokens::Table, AccessTokens::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn operators_table() -> TableCreateStatement {
        Table::create()
            .table(BusinessOperators::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(BusinessOperators::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(BusinessOperators::Username)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BusinessOperators::Password)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BusinessOperators::BusinessId)
                    .integer()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(BusinessOperators::Table, BusinessOperators::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn user_profiles_table() -> TableCreateStatement {
        Table::create()
            .table(UserProfiles::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UserProfiles::AccountId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(UserProfiles::Name).string().not_null())
            .col(ColumnDef::new(UserProfiles::Phone).string().not_null())
            .col(ColumnDef::new(UserProfiles::Pin).string().not_null())
            .col(ColumnDef::new(UserProfiles::Imsi).string().not_null())
            .col(
                ColumnDef::new(UserProfiles::RegisteredAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserProfiles::LastSwapDate)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(UserProfiles::Table, UserProfiles::AccountId)
                    .to(Accounts::Table, Accounts::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
            .to_owned()
    }

    pub fn mmf_account_table() -> TableCreateStatement {
        Table::create()
            .table(MmfAccounts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(MmfAccounts::AccountId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(MmfAccounts::BusinessId).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(MmfAccounts::Table, MmfAccounts::AccountId)
                    .to(Accounts::Table, Accounts::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(MmfAccounts::Table, MmfAccounts::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn paybill_table() -> TableCreateStatement {
        Table::create()
            .table(PaybillAccounts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(PaybillAccounts::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::BusinessId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::PaybillNumber)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::ResponseType)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::ValidationUrl)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::ConfirmationUrl)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(PaybillAccounts::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(PaybillAccounts::Table, PaybillAccounts::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn till_account_table() -> TableCreateStatement {
        Table::create()
            .table(TillAccounts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TillAccounts::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(TillAccounts::BusinessId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TillAccounts::TillNumber)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TillAccounts::LocationDescription)
                    .string()
                    .null(),
            )
            .col(ColumnDef::new(TillAccounts::ResponseType).string().null())
            .col(ColumnDef::new(TillAccounts::ValidationUrl).string().null())
            .col(
                ColumnDef::new(TillAccounts::ConfirmationUrl)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(TillAccounts::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(TillAccounts::Table, TillAccounts::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn utility_account_table() -> TableCreateStatement {
        Table::create()
            .table(UtilityAccounts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UtilityAccounts::AccountId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UtilityAccounts::BusinessId)
                    .integer()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(UtilityAccounts::Table, UtilityAccounts::AccountId)
                    .to(Accounts::Table, Accounts::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(UtilityAccounts::Table, UtilityAccounts::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn transactions_table() -> TableCreateStatement {
        Table::create()
            .table(Transactions::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Transactions::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Transactions::From).integer().null())
            .col(ColumnDef::new(Transactions::To).integer().not_null())
            .col(
                ColumnDef::new(Transactions::Amount)
                    .big_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(Transactions::Fee).big_integer().not_null())
            .col(ColumnDef::new(Transactions::Currency).string().not_null())
            .col(
                ColumnDef::new(Transactions::TransactionType)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(Transactions::Status).string().not_null())
            .col(ColumnDef::new(Transactions::Notes).string().null())
            .col(ColumnDef::new(Transactions::ReversalOf).string().null())
            .col(
                ColumnDef::new(Transactions::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Transactions::UpdatedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(ColumnDef::new(Transactions::RequestId).string().null())
            .foreign_key(
                ForeignKey::create()
                    .from(Transactions::Table, Transactions::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn transactions_request_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-transactions-request-id")
            .table(Transactions::Table)
            .col(Transactions::RequestId)
            .to_owned()
    }

    pub fn transaction_costs_table() -> TableCreateStatement {
        Table::create()
            .table(TransactionCosts::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TransactionCosts::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(TransactionCosts::TransactionType)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionCosts::MinAmount)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionCosts::MaxAmount)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionCosts::FeeFixed)
                    .big_integer()
                    .null(),
            )
            .col(
                ColumnDef::new(TransactionCosts::FeePercentage)
                    .double()
                    .null(),
            )
            .to_owned()
    }

    pub fn callback_logs_table() -> TableCreateStatement {
        Table::create()
            .table(CallbackLogs::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(CallbackLogs::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(CallbackLogs::ProjectId)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(CallbackLogs::ConversationId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(CallbackLogs::OriginatorId)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(CallbackLogs::TransactionId).string().null())
            .col(ColumnDef::new(CallbackLogs::CallbackUrl).string().not_null())
            .col(ColumnDef::new(CallbackLogs::CallbackType).string().not_null())
            .col(ColumnDef::new(CallbackLogs::Payload).string().not_null())
            .col(ColumnDef::new(CallbackLogs::ResponseStatus).integer().null())
            .col(ColumnDef::new(CallbackLogs::ResponseBody).string().null())
            .col(ColumnDef::new(CallbackLogs::ResponseHeaders).string().null())
            .col(ColumnDef::new(CallbackLogs::Status).string().not_null())
            .col(ColumnDef::new(CallbackLogs::Error).string().null())
            .col(
                ColumnDef::new(CallbackLogs::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(CallbackLogs::UpdatedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(ColumnDef::new(CallbackLogs::RequestId).string().null())
            .foreign_key(
                ForeignKey::create()
                    .name("FK_CallbackLogs_ProjectId")
                    .from_col(CallbackLogs::ProjectId)
                    .to_tbl(Projects::Table)
                    .to_col(Projects::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(CallbackLogs::Table, CallbackLogs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn callback_logs_request_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-callback-logs-request-id")
            .table(CallbackLogs::Table)
            .col(CallbackLogs::RequestId)
            .to_owned()
    }

    pub fn transactions_log_table() -> TableCreateStatement {
        Table::create()
            .table(TransactionsLog::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TransactionsLog::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(TransactionsLog::TransactionId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionsLog::AccountId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionsLog::Direction)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionsLog::NewBalance)
                    .big_integer()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(TransactionsLog::Table, TransactionsLog::AccountId)
                    .to(Accounts::Table, Accounts::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(TransactionsLog::Table, TransactionsLog::TransactionId)
                    .to(Transactions::Table, Transactions::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
            .to_owned()
    }

    pub fn transaction_jobs_table() -> TableCreateStatement {
        Table::create()
            .table(TransactionJobs::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TransactionJobs::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(TransactionJobs::RequestId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::OriginatorConversationId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::ConversationId)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::TransactionId)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(TransactionJobs::Status).string().not_null())
            .col(
                ColumnDef::new(TransactionJobs::ProcessAfter)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(TransactionJobs::Payload).json().not_null())
            .col(ColumnDef::new(TransactionJobs::ResultPayload).json().null())
            .col(
                ColumnDef::new(TransactionJobs::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(TransactionJobs::UpdatedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(TransactionJobs::Table, TransactionJobs::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn transaction_jobs_request_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-transaction-jobs-request-id")
            .table(TransactionJobs::Table)
            .col(TransactionJobs::RequestId)
            .to_owned()
    }

    pub fn requests_table() -> TableCreateStatement {
        Table::create()
            .table(Requests::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Requests::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Requests::SourceType).string().not_null())
            .col(ColumnDef::new(Requests::SourceApiKeyId).integer().null())
            .col(ColumnDef::new(Requests::SourceOperatorId).integer().null())
            .col(ColumnDef::new(Requests::SourceComponent).string().null())
            .col(ColumnDef::new(Requests::RequestType).string().not_null())
            .col(
                ColumnDef::new(Requests::RequestStatus)
                    .string()
                    .not_null()
                    .default("pending"),
            )
            .col(
                ColumnDef::new(Requests::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Requests::StartedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(
                ColumnDef::new(Requests::CompletedAt)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(ColumnDef::new(Requests::ProjectId).integer().null())
            .col(ColumnDef::new(Requests::BusinessId).integer().null())
            .col(ColumnDef::new(Requests::UserId).integer().null())
            .col(ColumnDef::new(Requests::RequestBody).text().null())
            .col(ColumnDef::new(Requests::ResponseBody).text().null())
            .col(ColumnDef::new(Requests::ErrorMessage).text().null())
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::SourceApiKeyId)
                    .to(ApiKeys::Table, ApiKeys::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::SourceOperatorId)
                    .to(BusinessOperators::Table, BusinessOperators::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::BusinessId)
                    .to(Businesses::Table, Businesses::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(Requests::Table, Requests::UserId)
                    .to(UserProfiles::Table, UserProfiles::AccountId)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull),
            )
            .to_owned()
    }

    pub fn requests_project_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-project-id")
            .table(Requests::Table)
            .col(Requests::ProjectId)
            .to_owned()
    }

    pub fn requests_business_id_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-business-id")
            .table(Requests::Table)
            .col(Requests::BusinessId)
            .to_owned()
    }

    pub fn requests_source_type_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-source-type")
            .table(Requests::Table)
            .col(Requests::SourceType)
            .to_owned()
    }

    pub fn requests_created_at_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-requests-created-at")
            .table(Requests::Table)
            .col(Requests::CreatedAt)
            .to_owned()
    }

    pub fn request_ids_table() -> TableCreateStatement {
        Table::create()
            .table(RequestIds::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(RequestIds::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(RequestIds::RequestId).string().not_null())
            .col(ColumnDef::new(RequestIds::IdType).string().not_null())
            .col(ColumnDef::new(RequestIds::ExternalId).string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(RequestIds::Table, RequestIds::RequestId)
                    .to(Requests::Table, Requests::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned()
    }

    pub fn request_ids_lookup_index() -> IndexCreateStatement {
        Index::create()
            .name("idx-request-ids-lookup")
            .table(RequestIds::Table)
            .col(RequestIds::IdType)
            .col(RequestIds::ExternalId)
            .to_owned()
    }

    pub fn request_ids_unique_constraint() -> IndexCreateStatement {
        Index::create()
            .name("unique-request-id-type")
            .table(RequestIds::Table)
            .col(RequestIds::RequestId)
            .col(RequestIds::IdType)
            .unique()
            .to_owned()
    }

    pub fn app_metadata_table() -> TableCreateStatement {
        Table::create()
            .table(AppMetadata::Table)
            .if_not_exists()
            .col(ColumnDef::new(AppMetadata::Key).string().not_null().primary_key())
            .col(ColumnDef::new(AppMetadata::Value).string().not_null())
            .to_owned()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Self::accounts_table()).await?;
        manager.create_table(Self::businesses_table()).await?;
        manager.create_index(Self::businesses_short_code_unique_index()).await?;
        manager.create_table(Self::projects_table()).await?;
        manager.create_table(Self::api_keys_table()).await?;
        manager.create_table(Self::requests_table()).await?;
        manager.create_index(Self::requests_project_id_index()).await?;
        manager.create_index(Self::requests_business_id_index()).await?;
        manager.create_index(Self::requests_source_type_index()).await?;
        manager.create_index(Self::requests_created_at_index()).await?;
        manager.create_table(Self::api_logs_table()).await?;
        manager.create_index(Self::api_logs_request_id_index()).await?;
        manager.create_table(Self::access_tokens_table()).await?;
        manager.create_table(Self::operators_table()).await?;
        manager.create_table(Self::user_profiles_table()).await?;
        manager.create_table(Self::mmf_account_table()).await?;
        manager.create_table(Self::paybill_table()).await?;
        manager.create_table(Self::till_account_table()).await?;
        manager.create_table(Self::utility_account_table()).await?;
        manager.create_table(Self::transactions_table()).await?;
        manager.create_index(Self::transactions_request_id_index()).await?;
        manager.create_table(Self::transaction_costs_table()).await?;
        manager.create_table(Self::callback_logs_table()).await?;
        manager.create_index(Self::callback_logs_request_id_index()).await?;
        manager.create_table(Self::transactions_log_table()).await?;
        manager.create_table(Self::transaction_jobs_table()).await?;
        manager.create_index(Self::transaction_jobs_request_id_index()).await?;
        manager.create_table(Self::request_ids_table()).await?;
        manager.create_index(Self::request_ids_lookup_index()).await?;
        manager.create_index(Self::request_ids_unique_constraint()).await?;
        manager.create_table(Self::app_metadata_table()).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(AppMetadata::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("unique-request-id-type").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-request-ids-lookup").to_owned()).await?;
        manager.drop_table(Table::drop().table(RequestIds::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-transaction-jobs-request-id").to_owned()).await?;
        manager.drop_table(Table::drop().table(TransactionJobs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(TransactionsLog::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-callback-logs-request-id").to_owned()).await?;
        manager.drop_table(Table::drop().table(CallbackLogs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(TransactionCosts::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-transactions-request-id").to_owned()).await?;
        manager.drop_table(Table::drop().table(Transactions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UtilityAccounts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(TillAccounts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PaybillAccounts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(MmfAccounts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UserProfiles::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(BusinessOperators::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AccessTokens::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-api-logs-request-id").to_owned()).await?;
        manager.drop_table(Table::drop().table(ApiLogs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ApiKeys::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-requests-created-at").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-requests-source-type").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-requests-business-id").to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-requests-project-id").to_owned()).await?;
        manager.drop_table(Table::drop().table(Requests::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Projects::Table).to_owned()).await?;
        manager.drop_index(Index::drop().name("idx-businesses-short_code").to_owned()).await?;
        manager.drop_table(Table::drop().table(Businesses::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Accounts::Table).to_owned()).await?;

        Ok(())
    }
}
