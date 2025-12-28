use sea_orm_migration::prelude::*;

// Local Aliases (Iden enums) for all tables

#[derive(Iden)]
enum AccessTokens {
    Table,
    Token,
    ProjectId,
    ExpiresAt,
    CreatedAt,
}

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
enum BusinessOperators {
    Table,
    Id,
    Username,
    Password,
    BusinessId,
}

#[derive(Iden)]
enum CallbackLogs {
    Table,
    Id,
    TransactionId,
    CheckoutRequestId,
    MerchantRequestId,
    CallbackUrl,
    CallbackType,
    Payload,
    ResponseStatus,
    ResponseBody,
    Status,
    Error,
    CreatedAt,
    UpdatedAt,
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
enum UserProfiles {
    Table,
    AccountId,
    Name,
    Phone,
    Pin,
}

#[derive(Iden)]
enum UtilityAccounts {
    Table,
    AccountId,
    BusinessId,
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
    pub fn businesses_table_index() -> IndexCreateStatement {
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
            .foreign_key(
                ForeignKey::create()
                    .from(ApiLogs::Table, ApiLogs::ProjectId)
                    .to(Projects::Table, Projects::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade),
            )
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
            .col(ColumnDef::new(CallbackLogs::TransactionId).string().null())
            .col(
                ColumnDef::new(CallbackLogs::CheckoutRequestId)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(CallbackLogs::MerchantRequestId)
                    .string()
                    .null(),
            )
            .col(
                ColumnDef::new(CallbackLogs::CallbackUrl)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(CallbackLogs::CallbackType)
                    .string()
                    .not_null(),
            )
            .col(ColumnDef::new(CallbackLogs::Payload).string().not_null())
            .col(
                ColumnDef::new(CallbackLogs::ResponseStatus)
                    .integer()
                    .null(),
            )
            .col(ColumnDef::new(CallbackLogs::ResponseBody).string().null())
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
            .foreign_key(
                ForeignKey::create()
                    .from(CallbackLogs::Table, CallbackLogs::TransactionId)
                    .to(Transactions::Table, Transactions::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction),
            )
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
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tables in an order that respects foreign key constraints

        // 1. Accounts
        manager.create_table(Self::accounts_table()).await?;

        // 2. Businesses
        manager.create_table(Self::businesses_table()).await?;

        // Add unique index for business short_code
        manager.create_index(Self::businesses_table_index()).await?;

        // 3. Projects (depends on Businesses)
        manager.create_table(Self::projects_table()).await?;

        // 4. API Keys (depends on Projects)
        manager.create_table(Self::api_keys_table()).await?;

        // 5. API Logs (depends on Projects)
        manager.create_table(Self::api_logs_table()).await?;

        // 6. Access Tokens (depends on Projects)
        manager.create_table(Self::access_tokens_table()).await?;

        // 7. Business Operators (depends on Businesses)
        manager.create_table(Self::operators_table()).await?;

        // 8. User Profiles (depends on Accounts)
        manager.create_table(Self::user_profiles_table()).await?;

        // 9. Mmf Accounts (depends on Accounts, Businesses)
        manager.create_table(Self::mmf_account_table()).await?;

        // 10. Paybill Accounts (depends on Businesses)
        manager.create_table(Self::paybill_table()).await?;

        // 11. Till Accounts (depends on Businesses)
        manager.create_table(Self::till_account_table()).await?;

        // 12. Utility Accounts (depends on Accounts, Businesses)
        manager.create_table(Self::utility_account_table()).await?;

        // 13. Transactions
        manager.create_table(Self::transactions_table()).await?;

        // 14. Transaction Costs
        manager
            .create_table(Self::transaction_costs_table())
            .await?;

        // 15. Callback Logs (depends on Transactions)
        manager.create_table(Self::callback_logs_table()).await?;

        // 16. Transactions Log (depends on Accounts, Transactions)
        manager.create_table(Self::transactions_log_table()).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation due to foreign key constraints
        manager
            .drop_table(Table::drop().table(TransactionsLog::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CallbackLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TransactionCosts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UtilityAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TillAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaybillAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MmfAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserProfiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BusinessOperators::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AccessTokens::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ApiLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ApiKeys::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Businesses::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;

        Ok(())
    }
}
