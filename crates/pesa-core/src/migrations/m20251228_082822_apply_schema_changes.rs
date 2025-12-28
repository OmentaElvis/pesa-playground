use chrono::Utc;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, sea_query::Expr};
use sea_orm_migration::prelude::*;

use super::m20251227_183827_initial_schema::Migration as MigrationV1;

#[derive(Iden)]
enum UserProfiles {
    Table,
    AccountId,
    Imsi,
    RegisteredAt,
    LastSwapDate,
}

#[derive(Iden)]
enum CallbackLogs {
    Table,
    ProjectId,
    ConversationId,
    OriginatorId,
    ResponseHeaders,
    CheckoutRequestId,
    MerchantRequestId,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // === Apply changes to user_profiles table ===
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .add_column(
                        ColumnDef::new(UserProfiles::Imsi)
                            .string()
                            .not_null()
                            .default("UNSET"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .add_column(
                        ColumnDef::new(UserProfiles::RegisteredAt)
                            .timestamp_with_time_zone()
                            .default(Utc::now().to_rfc2822())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .add_column(
                        ColumnDef::new(UserProfiles::LastSwapDate)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Backfill the data for existing users
        let users_to_update: Vec<(i32,)> = crate::accounts::user_profiles::db::Entity::find()
            .select_only()
            .column(crate::accounts::user_profiles::db::Column::AccountId)
            .filter(Expr::col(UserProfiles::Imsi).eq("UNSET"))
            .into_tuple()
            .all(db)
            .await?;

        for (user_id,) in users_to_update {
            let new_imsi = crate::accounts::user_profiles::User::generate_test_imsi();
            let new_reg_date = crate::accounts::user_profiles::User::random_registration_date();
            crate::accounts::user_profiles::db::Entity::update_many()
                .col_expr(UserProfiles::Imsi, Expr::value(new_imsi))
                .col_expr(UserProfiles::RegisteredAt, Expr::value(new_reg_date))
                .filter(Expr::col(UserProfiles::AccountId).eq(user_id))
                .exec(db)
                .await?;
        }

        // === Apply changes to callback_logs table ===
        manager
            .get_connection()
            .execute_unprepared(&format!("DELETE FROM {};", CallbackLogs::Table.to_string()))
            .await?;

        manager
            .drop_table(Table::drop().table(CallbackLogs::Table).to_owned())
            .await?;

        // create the new callback_logs table
        let mut create = Table::create();
        create.table(CallbackLogs::Table);

        let callback_logs_create = MigrationV1::callback_logs_table();

        for column in callback_logs_create.get_columns() {
            let name = column.get_column_name();
            if name == CallbackLogs::CheckoutRequestId.to_string()
                || name == CallbackLogs::MerchantRequestId.to_string()
            {
                continue;
            }
            create.col(column.clone());
        }

        for foreign in callback_logs_create.get_foreign_key_create_stmts() {
            let mut foreign = foreign.clone();
            create.foreign_key(&mut foreign);
        }

        create.col(
            ColumnDef::new(CallbackLogs::ProjectId)
                .integer()
                .not_null()
                .default(0),
        );

        create.col(
            ColumnDef::new(CallbackLogs::ConversationId)
                .string()
                .not_null()
                .default(""),
        );

        create.col(
            ColumnDef::new(CallbackLogs::OriginatorId)
                .string()
                .not_null()
                .default(""),
        );

        create.col(
            ColumnDef::new(CallbackLogs::ResponseHeaders)
                .string()
                .null(),
        );

        create.foreign_key(
            ForeignKey::create()
                .name("FK_CallbackLogs_ProjectId")
                .from_col(CallbackLogs::ProjectId)
                .to_tbl(Projects::Table)
                .to_col(Projects::Id)
                .on_delete(ForeignKeyAction::Cascade),
        );

        manager.create_table(create).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .drop_column(UserProfiles::Imsi)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .drop_column(UserProfiles::RegisteredAt)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfiles::Table)
                    .drop_column(UserProfiles::LastSwapDate)
                    .to_owned(),
            )
            .await?;

        // Revert callback_logs changes
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_foreign_key("FK_CallbackLogs_ProjectId")
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_column(CallbackLogs::ProjectId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_column(CallbackLogs::ConversationId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_column(CallbackLogs::OriginatorId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .drop_column(CallbackLogs::ResponseHeaders)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .add_column(
                        ColumnDef::new(CallbackLogs::CheckoutRequestId)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(CallbackLogs::Table)
                    .add_column(
                        ColumnDef::new(CallbackLogs::MerchantRequestId)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
