use chrono::Utc;
use rand::{distr::Alphanumeric, Rng};

use crate::{
    accounts::{user_profiles::User, utility_accounts::UtilityAccount},
    callbacks::{CallbackLog, CallbackStatus, CallbackType},
    projects::Project,
    server::ApiState,
    transactions::{TransactionNote, TransactionType},
};

use super::process::callback_execute;

pub struct StkpushInit {
    pub checkout_request_id: String,
    pub merchant_request_id: String,
    pub callback_url: String,
    pub user: User,
    pub business: UtilityAccount,
    pub amount: i64,
    pub notes: TransactionNote,
}

impl StkpushInit {
    pub fn new(
        url: String,
        user: User,
        business: UtilityAccount,
        amount: i64,
        notes: TransactionNote,
    ) -> Self {
        Self {
            checkout_request_id: Self::generate_checkout_request_id(),
            merchant_request_id: Self::generate_merchant_request_id(),
            callback_url: url,
            user,
            business,
            amount,
            notes,
        }
    }

    pub fn generate_merchant_request_id() -> String {
        format!(
            "{}-{}-{}",
            rand::rng().random_range(10000..99999),
            rand::rng().random_range(10000000..99999999),
            rand::rng().random_range(0..9)
        )
    }

    pub fn generate_checkout_request_id() -> String {
        let timestamp = Utc::now().format("%d%m%Y%H%M%S").to_string(); // e.g. 02072025143500
        let rand_suffix: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();

        format!("ws_CO_{}{}", timestamp, rand_suffix)
    }

    pub async fn start(self, state: ApiState, project: Project, txn_type: TransactionType) {
        let mut callback = CallbackLog::new(CallbackType::StkPush);

        callback
            .with_callback_url(&self.callback_url)
            .with_status(CallbackStatus::Pending)
            .with_checkout_request_id(&self.checkout_request_id)
            .with_merchant_request_id(&self.merchant_request_id);

        let _ = match callback_execute(&state, &self, &callback, project, txn_type).await {
            Ok(s) => s,
            Err(err) => {
                println!("[error: {}] {}", state.project_id, err);
                return;
            }
        };
    }
}
