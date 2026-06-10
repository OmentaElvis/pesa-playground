use anyhow::Context;
use sea_orm::EntityTrait;

use crate::{
    accounts::db::Entity as AccountsEntity,
    accounts::user_profiles::db::Entity as UserProfilesEntity,
    api_keys::db::Entity as ApiKeysEntity,
    business::db::Entity as BusinessEntity,
    business_operators::db::Entity as BusinessOperatorsEntity,
    projects::db::Entity as ProjectsEntity,
    self_test::{callback::CallbackManager, context::TestContext, runner::TestStep},
    system::ui::clear_all_data,
    transaction_costs::db::Entity as TransactionCostsEntity,
};

pub struct SystemPurgeTest;

impl TestStep for SystemPurgeTest {
    async fn run(
        &self,
        context: &mut TestContext,
        _callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running System Purge Test ==").await;

        let initial_counts = get_table_counts(context).await?;
        context
            .log(&format!(
                ">> Initial table counts: businesses={}, users={}, projects={}, accounts={}, api_keys={}, operators={}, txn_costs={}",
                initial_counts.businesses,
                initial_counts.user_profiles,
                initial_counts.projects,
                initial_counts.accounts,
                initial_counts.api_keys,
                initial_counts.business_operators,
                initial_counts.transaction_costs,
            ))
            .await;

        context.log(">> Calling clear_all_data...").await;
        clear_all_data(&context.app_context)
            .await
            .context("Failed to clear all data")?;

        let final_counts = get_table_counts(context).await?;
        context
            .log(&format!(
                ">> Final table counts: businesses={}, users={}, projects={}, accounts={}, api_keys={}, operators={}, txn_costs={}",
                final_counts.businesses,
                final_counts.user_profiles,
                final_counts.projects,
                final_counts.accounts,
                final_counts.api_keys,
                final_counts.business_operators,
                final_counts.transaction_costs,
            ))
            .await;

        assert_eq!(
            final_counts.businesses, 0,
            "Expected businesses table to be empty after purge"
        );
        assert_eq!(
            final_counts.user_profiles, 0,
            "Expected user_profiles table to be empty after purge"
        );
        assert_eq!(
            final_counts.projects, 0,
            "Expected projects table to be empty after purge"
        );
        assert_eq!(
            final_counts.accounts, 0,
            "Expected accounts table to be empty after purge"
        );
        assert_eq!(
            final_counts.api_keys, 0,
            "Expected api_keys table to be empty after purge"
        );
        assert_eq!(
            final_counts.business_operators, 0,
            "Expected business_operators table to be empty after purge"
        );

        assert!(
            final_counts.transaction_costs > 0,
            "Expected transaction_costs table to have seed data after purge"
        );

        context.log("== System Purge Test Passed ==").await;
        Ok(())
    }
}

struct TableCounts {
    businesses: usize,
    user_profiles: usize,
    projects: usize,
    accounts: usize,
    api_keys: usize,
    business_operators: usize,
    transaction_costs: usize,
}

async fn get_table_counts(context: &TestContext) -> anyhow::Result<TableCounts> {
    let db = &context.app_context.db;

    let businesses = BusinessEntity::find().all(db).await?.len();
    let user_profiles = UserProfilesEntity::find().all(db).await?.len();
    let projects = ProjectsEntity::find().all(db).await?.len();
    let accounts = AccountsEntity::find().all(db).await?.len();
    let api_keys = ApiKeysEntity::find().all(db).await?.len();
    let business_operators = BusinessOperatorsEntity::find().all(db).await?.len();
    let transaction_costs = TransactionCostsEntity::find().all(db).await?.len();

    Ok(TableCounts {
        businesses,
        user_profiles,
        projects,
        accounts,
        api_keys,
        business_operators,
        transaction_costs,
    })
}