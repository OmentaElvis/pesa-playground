use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{accounts::utility_accounts::UtilityAccount, AppContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUtilityAccountInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUtilityAccountOutput {
    pub account_id: u32,
    pub name: String,
}

pub async fn get_utility_account(
    state: &AppContext,
    id: u32,
) -> anyhow::Result<Option<UtilityAccount>> {
    UtilityAccount::find_by_id(&state.db, id)
        .await
        .context("Failed to get Utility account")
}

pub async fn get_utility_account_by_business_id(
    state: &AppContext,
    business_id: u32,
) -> anyhow::Result<Option<UtilityAccount>> {
    UtilityAccount::find_by_business_id(&state.db, business_id)
        .await
        .context("Failed to get Utility account")
}
