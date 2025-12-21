use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{accounts::mmf_accounts::MmfAccount, AppContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMmfAccountInput {
    pub name: String,
    pub initial_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMmfAccountOutput {
    pub account_id: u32,
    pub name: String,
}

pub async fn get_mmf_account(state: &AppContext, id: u32) -> anyhow::Result<Option<MmfAccount>> {
    MmfAccount::find_by_id(&state.db, id)
        .await
        .context("Failed to get mmf account")
}

pub async fn get_mmf_account_by_business_id(
    state: &AppContext,
    business_id: u32,
) -> anyhow::Result<Option<MmfAccount>> {
    MmfAccount::find_by_business_id(&state.db, business_id)
        .await
        .context("Failed to get mmf account")
}
