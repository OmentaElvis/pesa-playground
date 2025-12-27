use std::time::Duration;

use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use serde_json::json;
use tokio::sync::oneshot;

use crate::accounts::{
    paybill_accounts::PaybillAccount, till_accounts::TillAccount, user_profiles::User,
    utility_accounts::UtilityAccount,
};
use crate::server::api::stkpush::ui::{STK_RESPONSE_REGISTRY, UserResponse};
use crate::server::{
    ApiError, ApiState, MpesaError,
    api::{
        auth::INVALID_CREDENTIALS,
        stkpush::{
            CallbackItem, StkCallback, StkCallbackBody, StkCallbackBodyWrapper, StkPushRequest,
            StkPushResponse, StkPushResultCode, TransactionType as StkTransactionType,
            generate_checkout_request_id, generate_merchant_request_id,
        },
    },
    async_handler::{IntoCallbackPayload, PpgAsyncRequest},
};
use crate::{
    api_keys::ApiKey,
    business::Business,
    events::DomainEventDispatcher,
    projects::Project,
    transactions::{Ledger, TransactionEngineError, TransactionNote, TransactionType},
};

pub struct Stkpush {
    pub business: Business,
    pub user: User,
    pub project: Project,
    pub utility_account: UtilityAccount,
    pub amount: i64,
    pub notes: TransactionNote,
    pub callback_url: String,
    pub merchant_id: String,
    pub checkout_id: String,
    pub transaction_type: TransactionType,
}

impl IntoCallbackPayload<Stkpush, StkCallbackBodyWrapper> for StkPushResultCode {
    fn get_payload(&self, ctx: &Stkpush) -> StkCallbackBodyWrapper {
        let message = self.to_string();
        let code = self.code();

        StkCallbackBodyWrapper {
            body: StkCallbackBody {
                callback: StkCallback {
                    merchant_request_id: ctx.merchant_id.clone(),
                    checkout_request_id: ctx.checkout_id.clone(),
                    result_code: code,
                    result_desc: message,
                    metadata: None,
                },
            },
        }
    }
}

impl PpgAsyncRequest for Stkpush {
    type RequestData = StkPushRequest;
    type SyncResponseData = StkPushResponse;
    type CallbackPayload = StkCallbackBodyWrapper;
    type Error = StkPushResultCode;

    fn api_name() -> &'static str {
        "stkpush"
    }

    async fn init(
        state: &ApiState,
        req: Self::RequestData,
        _conversation_id: &str,
        api_key: ApiKey,
    ) -> Result<(Self::SyncResponseData, Self), ApiError> {
        let passkey = api_key.passkey;
        let timestamp = req.timestamp;

        let (business_id, notes) = match req.transaction_type {
            StkTransactionType::CustomerBuyGoodsOnline => {
                let till = match TillAccount::get_by_till_number(
                    &state.context.db,
                    req.party_b.parse().unwrap_or_default(),
                )
                .await
                {
                    Ok(Some(till)) => till,
                    Ok(None) => {
                        return Err(ApiError::new(
                            MpesaError::InvalidShortcode,
                            "Invalid Till Number",
                        ));
                    }
                    Err(err) => {
                        return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
                    }
                };
                (
                    till.business_id,
                    TransactionNote::TillPayment {
                        till_number: till.till_number,
                    },
                )
            }
            StkTransactionType::CustomerPayBillOnline => {
                let paybill = match PaybillAccount::get_by_paybill_number(
                    &state.context.db,
                    req.party_b.parse().unwrap_or_default(),
                )
                .await
                {
                    Ok(Some(paybill)) => paybill,
                    Ok(None) => {
                        return Err(ApiError::new(
                            MpesaError::InvalidShortcode,
                            "Invalid Paybill Number",
                        ));
                    }
                    Err(err) => {
                        return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
                    }
                };

                (
                    paybill.business_id,
                    TransactionNote::PaybillPayment {
                        paybill_number: paybill.paybill_number,
                        bill_ref_number: req.account_reference,
                    },
                )
            }
        };

        let business = match Business::get_by_id(&state.context.db, business_id).await {
            Ok(Some(business)) => business,
            Ok(None) => {
                return Err(ApiError::new(
                    MpesaError::InvalidShortcode,
                    "Invalid business shortcode.",
                ));
            }
            Err(err) => {
                return Err(ApiError::new(MpesaError::InternalError, err.to_string()));
            }
        };

        let utility_account =
            match UtilityAccount::find_by_business_id(&state.context.db, business_id).await {
                Ok(Some(account)) => account,
                Ok(None) => {
                    return Err(ApiError::new(
                        crate::server::MpesaError::InternalError,
                        format!("Failed to read acount for business {}", business_id),
                    ));
                }
                Err(err) => {
                    return Err(ApiError::new(
                        crate::server::MpesaError::InternalError,
                        err.to_string(),
                    ));
                }
            };

        let short_code = business.short_code.clone();

        let password =
            general_purpose::STANDARD.encode(format!("{}{}{}", short_code, passkey, timestamp));

        if !password.eq(&req.password) {
            return Err(ApiError::new(
                MpesaError::InvalidCredentials,
                "Invalid password",
            ));
        }

        let amount = req.amount.parse::<f64>().map_err(|err| {
            ApiError::new(
                crate::server::MpesaError::InternalError,
                format!(
                    "Failed to parse amount as number: {}, you provided: {} ",
                    err, req.amount
                ),
            )
        })?;

        let user = match User::get_user_by_phone(&state.context.db, &req.phone_number)
            .await
            .map_err(|err| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An error occured while trying to get user: {}", err),
                )
            })? {
            Some(user) => user,
            None => {
                return Err(ApiError::new(
                    MpesaError::InvalidPhoneNumber,
                    "Invalid phone number",
                ));
            }
        };

        let project = match Project::get_by_id(&state.context.db, state.project_id).await {
            Ok(Some(project)) => project,
            Ok(None) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InvalidCredentials,
                    INVALID_CREDENTIALS,
                ));
            }
            Err(err) => {
                return Err(ApiError::new(
                    crate::server::MpesaError::InternalError,
                    err.to_string(),
                ));
            }
        };

        let amount = (amount * 100.0).round() as i64;
        let merchant_id = generate_merchant_request_id();
        let checkout_id = generate_checkout_request_id();

        Ok((
            StkPushResponse {
                merchant_request_id: merchant_id.clone(),
                checkout_request_id: checkout_id.clone(),
                response_code: 0,
                response_description: String::from(
                    "The service request has been accepted successfully.",
                ),
                customer_message: String::from("Success"),
            },
            Self {
                business,
                user,
                utility_account,
                amount,
                notes,
                callback_url: req.call_back_u_r_l,
                merchant_id,
                checkout_id,
                project,
                transaction_type: match req.transaction_type {
                    StkTransactionType::CustomerPayBillOnline => TransactionType::Paybill,
                    StkTransactionType::CustomerBuyGoodsOnline => TransactionType::BuyGoods,
                },
            },
        ))
    }

    async fn execute(&mut self, state: &ApiState) -> Result<Self::CallbackPayload, Self::Error> {
        let checkout_id = &self.checkout_id;
        let user = &self.user;
        let project = &self.project;

        let mut receipt = Ledger::generate_receipt();

        match project.simulation_mode {
            crate::projects::SimulationMode::AlwaysSuccess => {
                return Ok(self.create_body(StkPushResultCode::Success, Some(receipt)));
            }
            crate::projects::SimulationMode::AlwaysFail => {
                let status = StkPushResultCode::random_failure();
                return Ok(self.create_body(status, None));
            }
            crate::projects::SimulationMode::Random => {
                let status = StkPushResultCode::random();
                return Ok(self.create_body(status, Some(receipt)));
            }
            // next section is realistic
            crate::projects::SimulationMode::Realistic => {}
        }

        if user.disabled {
            return Ok(self.create_body(StkPushResultCode::DSTimeout, None));
        }

        if STK_RESPONSE_REGISTRY.contains_key(&self.checkout_id) {
            // another task is handling the user, stop moving too fast
            return Ok(self.create_body(StkPushResultCode::UnableToObtainSubscriberLock, None));
        }

        let (tx, rx) = oneshot::channel();
        STK_RESPONSE_REGISTRY.insert(checkout_id.clone(), tx);

        if state
            .context
            .event_manager
            .emit_all(
                "stk_push",
                json!({
                    "checkout_id": checkout_id,
                    "project": project,
                    "user": user,
                    "business_name": self.business.name,
                    "amount": self.amount as f64 / 100.0
                }),
            )
            .is_err()
        {
            return Ok(self.create_body(StkPushResultCode::ErrorSendingPushRequest, None));
        }

        let status = match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(value)) => match value {
                UserResponse::Accepted { pin } => {
                    if pin.eq(&user.pin) {
                        match Ledger::transfer(
                            &state.context.db,
                            Some(user.account_id),
                            self.utility_account.account_id,
                            self.amount,
                            &self.transaction_type,
                            Some(&self.notes),
                        )
                        .await
                        {
                            Ok((transaction, events)) => {
                                DomainEventDispatcher::dispatch_events(&state.context, events)?;
                                receipt = transaction.id;

                                return Ok(
                                    self.create_body(StkPushResultCode::Success, Some(receipt))
                                );
                            }
                            Err(err) => match err {
                                TransactionEngineError::InsufficientFunds => {
                                    StkPushResultCode::InsufficientBalance
                                }
                                TransactionEngineError::Database(err) => {
                                    tracing::error!(target: "stkpush", "Transaction engine db error: {err}");
                                    StkPushResultCode::SystemError
                                }
                                TransactionEngineError::AccountNotFound(_) => {
                                    StkPushResultCode::DSTimeout
                                }
                                _ => StkPushResultCode::SystemError,
                            },
                        }
                    } else {
                        StkPushResultCode::InitiatorInformationInvalid
                    }
                }
                UserResponse::Offline => StkPushResultCode::DSTimeout,
                UserResponse::Timeout => StkPushResultCode::NoResponseFromUser,
                UserResponse::Cancelled => StkPushResultCode::RequestCancelledByUser,
                UserResponse::Failed(_) => StkPushResultCode::ErrorSendingPushRequest1037,
            },
            Ok(Err(_)) => StkPushResultCode::SystemError,
            Err(_) => StkPushResultCode::NoResponseFromUser,
        };
        STK_RESPONSE_REGISTRY.remove(checkout_id.as_str());

        Ok(self.create_body(status, None))
    }

    fn get_originator_id(&self) -> &str {
        &self.checkout_id
    }
    fn get_callback_url(&self) -> Option<&str> {
        Some(&self.callback_url)
    }
}

impl Stkpush {
    pub fn create_body(
        &self,
        result_code: StkPushResultCode,
        receipt: Option<String>,
    ) -> StkCallbackBodyWrapper {
        let metadata = receipt.map(|receipt| super::CallbackMetadata {
            item: vec![
                CallbackItem {
                    name: "Amount".to_string(),
                    value: json!(self.amount as f64 / 100.0),
                },
                CallbackItem {
                    name: "MpesaReceiptNumber".to_string(),
                    value: receipt.into(),
                },
                CallbackItem {
                    name: "TransactionDate".to_string(),
                    value: Utc::now().format("%Y%m%d%H%M%S").to_string().into(),
                },
                CallbackItem {
                    name: "PhoneNumber".to_string(),
                    value: self.user.phone.to_string().into(),
                },
            ],
        });

        StkCallbackBodyWrapper {
            body: StkCallbackBody {
                callback: StkCallback {
                    merchant_request_id: self.merchant_id.clone(),
                    checkout_request_id: self.checkout_id.clone(),
                    result_code: result_code.code(),
                    result_desc: result_code.to_string(),
                    metadata,
                },
            },
        }
    }
}
