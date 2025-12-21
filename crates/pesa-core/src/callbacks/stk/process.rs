use std::time::Duration;

use chrono::Utc;
use serde_json::json;
use tokio::sync::oneshot;

use crate::{
    accounts::{user_profiles::ui::UserDetails, Account},
    callbacks::{response::return_body, CallbackLog},
    events::DomainEventDispatcher,
    projects::Project,
    server::ApiState,
    transactions::{Ledger, TransactionEngineError},
    transactions_log::get_account_name,
};

use super::{init::StkpushInit, StkCodes, UserResponse, STK_RESPONSE_REGISTRY};

pub async fn callback_execute(
    state: &ApiState,
    init: &StkpushInit,
    callback: &CallbackLog,
    project: Project,
    transaction_type: crate::transactions::TransactionType,
) -> anyhow::Result<(StkCodes, Option<String>)> {
    let checkout_id = callback.checkout_request_id.clone().unwrap();
    let merchant_id = callback.merchant_request_id.clone().unwrap();
    let user = &init.user;
    let mut receipt = Ledger::generate_receipt();

    match project.simulation_mode {
        crate::projects::SimulationMode::AlwaysSuccess => {
            return_body(
                state,
                StkCodes::Success,
                callback.callback_url.to_string(),
                merchant_id,
                checkout_id,
                Some(json!({
                    "Amount": init.amount as f64 / 100.0,
                    "MpesaReceiptNumber": receipt,
                    "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                    "PhoneNumber": user.phone,
                })),
            )
            .await;

            return Ok((StkCodes::Success, Some(receipt)));
        }
        crate::projects::SimulationMode::AlwaysFail => {
            let status = StkCodes::random_failure();
            return_body(
                state,
                status.clone(),
                callback.callback_url.to_string(),
                merchant_id,
                checkout_id,
                None,
            )
            .await;
            return Ok((status, None));
        }
        crate::projects::SimulationMode::Random => {
            let status = StkCodes::random();
            return_body(
                state,
                status.clone(),
                callback.callback_url.to_string(),
                merchant_id,
                checkout_id,
                match status {
                    StkCodes::Success => Some(json!({
                        "Amount": init.amount as f64 / 100.0,
                        "MpesaReceiptNumber": receipt,
                        "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                        "PhoneNumber": user.phone.to_string(),
                    })),
                    _ => None,
                },
            )
            .await;
            return Ok((
                status.clone(),
                match status {
                    StkCodes::Success => Some(receipt),
                    _ => None,
                },
            ));
        }
        // next section is realistic
        crate::projects::SimulationMode::Realistic => {}
    }

    let account = match Account::get_account(&state.context.db, user.account_id).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            return_body(
                state,
                StkCodes::DSTimeout,
                callback.callback_url.to_string(),
                merchant_id,
                checkout_id,
                None,
            )
            .await;
            return Ok((StkCodes::DSTimeout, None));
        }
        Err(_) => {
            return_body(
                state,
                StkCodes::SystemError,
                callback.callback_url.to_string(),
                merchant_id,
                checkout_id,
                None,
            )
            .await;
            return Ok((StkCodes::SystemError, None));
        }
    };

    if account.disabled {
        return_body(
            state,
            StkCodes::DSTimeout,
            callback.callback_url.to_string(),
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::DSTimeout, None));
    }

    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    if reg.contains_key(&checkout_id) {
        // another task is handling the user, stop moving too fast
        return_body(
            state,
            StkCodes::UnableToObtainSubscriberLock,
            callback.callback_url.to_string(),
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::UnableToObtainSubscriberLock, None));
    }

    let (tx, rx) = oneshot::channel();
    reg.insert(checkout_id.clone(), tx);
    let business_name = get_account_name(&state.context.db, init.business.account_id).await?;
    if state
        .context
        .event_manager
        .emit_all(
            "stk_push",
            json!({
                "checkout_id": checkout_id,
                "project": project,
                "user": UserDetails {
                    id: account.id,
                    name: user.name.clone(),
                    pin: user.pin.clone(),
                    phone: user.phone.clone(),
                    balance: account.balance as f64 / 100.0,
                },
                "business_name": business_name,
                "callback": callback,
                "amount": init.amount as f64 / 100.0
            }),
        )
        .is_err()
    {
        return_body(
            state,
            StkCodes::ErrorSendingPushRequest,
            callback.callback_url.to_string(),
            merchant_id,
            checkout_id,
            None,
        )
        .await;
        return Ok((StkCodes::ErrorSendingPushRequest, None));
    }

    drop(reg);

    let status = match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(value)) => match value {
            UserResponse::Accepted { pin } => {
                if pin.eq(&user.pin) {
                    match Ledger::transfer(
                        &state.context.db,
                        Some(account.id),
                        init.business.account_id,
                        init.amount,
                        &transaction_type,
                        Some(&init.notes),
                    )
                    .await
                    {
                        Ok((transaction, events)) => {
                            DomainEventDispatcher::dispatch_events(&state.context, events)?;
                            receipt = transaction.id;
                            StkCodes::Success
                        }
                        Err(err) => match err {
                            TransactionEngineError::InsufficientFunds => {
                                StkCodes::InsufficientBalance
                            }
                            TransactionEngineError::Database(err) => {
                                println!("{}", err);
                                StkCodes::SystemError
                            }
                            TransactionEngineError::AccountNotFound(_) => StkCodes::DSTimeout,
                            _ => StkCodes::SystemError,
                        },
                    }
                } else {
                    StkCodes::InitiatorInformationInvalid
                }
            }
            UserResponse::Offline => StkCodes::DSTimeout,
            UserResponse::Timeout => StkCodes::NoResponseFromUser,
            UserResponse::Cancelled => StkCodes::RequestCancelledByUser,
            UserResponse::Failed(_) => StkCodes::ErrorSendingPushRequest1037,
        },
        Ok(Err(_)) => StkCodes::SystemError,
        Err(_) => StkCodes::NoResponseFromUser,
    };
    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    reg.remove(&checkout_id);
    drop(reg);

    return_body(
        state,
        status.clone(),
        callback.callback_url.to_string(),
        merchant_id,
        checkout_id,
        match status {
            StkCodes::Success => Some(json!({
                "Amount": init.amount as f64 / 100.0,
                "MpesaReceiptNumber": receipt,
                "TransactionDate": Utc::now().format("%Y%m%d%H%M%S").to_string(),
                "PhoneNumber": user.phone,
            })),
            _ => None,
        },
    )
    .await;
    Ok((
        status.clone(),
        match status {
            StkCodes::Success => Some(receipt),
            _ => None,
        },
    ))
}
