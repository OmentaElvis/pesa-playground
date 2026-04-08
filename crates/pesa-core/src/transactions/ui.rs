use std::time::Duration;

use super::{Ledger, Transaction, TransactionEngineError, TransactionNote, TransactionType};
use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};

use crate::AppContext;
use crate::accounts::Account;
use crate::accounts::paybill_accounts::PaybillAccount;
use crate::accounts::till_accounts::TillAccount;
use crate::accounts::user_profiles::User;
use crate::accounts::utility_accounts;
use crate::accounts::utility_accounts::UtilityAccount;
use crate::api_logs::ApiLog;
use crate::business::Business;
use crate::events::DomainEventDispatcher;
use crate::projects;
use crate::server::api::c2b::C2bTransactionType;
use crate::server::api::c2b::ResponseType;
use crate::server::api::c2b::ValidationRequest;
use crate::server::api::c2b::ValidationResponse;
use crate::transaction_costs::get_fee;
use crate::transaction_jobs::task::TransferConfig;
use crate::transactions::TransactionFilter;
use crate::utils::identifiers::Identifiers;

pub async fn get_transaction(
    ctx: &AppContext,
    transaction_id: String,
) -> Result<Option<Transaction>> {
    Transaction::get(&ctx.db, &transaction_id)
        .await
        .context("Failed to get transaction")
}

pub async fn list_system_transactions(
    ctx: &AppContext,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Transaction>> {
    Transaction::list_system(&ctx.db, limit.map(|l| l as u64), offset.map(|o| o as u64))
        .await
        .context("Failed to list system transactions")
}

pub async fn list_transactions(
    ctx: &AppContext,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>> {
    Transaction::list(&ctx.db, filter)
        .await
        .context("Failed to list transactions")
}

pub async fn count_transactions(ctx: &AppContext, filter: TransactionFilter) -> Result<u64> {
    Transaction::count(&ctx.db, filter)
        .await
        .context("Failed to count transactions")
}

async fn total_transaction_volume(ctx: &AppContext) -> Result<i64> {
    Transaction::total_volume(&ctx.db)
        .await
        .context("Failed to get total transaction volume")
}

async fn total_transaction_fees(ctx: &AppContext) -> Result<i64> {
    Transaction::total_fees(&ctx.db)
        .await
        .context("Failed to get total transaction fees")
}

pub async fn get_transaction_by_checkout_request(
    ctx: &AppContext,
    checkout_request_id: String,
) -> Result<Option<Transaction>> {
    Transaction::get_by_checkout_request_id(&ctx.db, &checkout_request_id)
        .await
        .context("Failed to get transaction by checkout request ID")
}

pub async fn get_user_transactions(
    ctx: &AppContext,
    user_id: u32,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<Transaction>> {
    let filter = TransactionFilter {
        from: Some(user_id),
        to: None,
        transaction_type: None,
        status: None,
        limit: limit.or(Some(20)),
        offset: offset.or(Some(0)),
    };

    list_transactions(ctx, filter).await
}

pub async fn get_recent_transactions(
    ctx: &AppContext,
    limit: Option<u64>,
) -> Result<Vec<Transaction>> {
    let filter = TransactionFilter {
        from: None,
        to: None,
        transaction_type: None,
        status: None,
        limit: limit.or(Some(10)),
        offset: Some(0),
    };

    list_transactions(ctx, filter).await
}

pub async fn get_transaction_stats(ctx: &AppContext) -> Result<TransactionStats> {
    let filter = TransactionFilter {
        from: None,
        to: None,
        transaction_type: None,
        status: None,
        limit: None,
        offset: None,
    };

    let total_count = count_transactions(ctx, filter.clone())
        .await
        .context("Failed to get total count")?;

    let successful_filter = TransactionFilter {
        status: Some(super::TransactionStatus::Completed),
        ..filter.clone()
    };

    let successful_count = count_transactions(ctx, successful_filter)
        .await
        .context("Failed to get successful count")?;

    let pending_filter = TransactionFilter {
        status: Some(super::TransactionStatus::Pending),
        ..filter.clone()
    };

    let pending_count = count_transactions(ctx, pending_filter)
        .await
        .context("Failed to get pending count")?;

    let failed_filter = TransactionFilter {
        status: Some(super::TransactionStatus::Failed),
        ..filter
    };

    let failed_count = count_transactions(ctx, failed_filter)
        .await
        .context("Failed to get failed count")?;

    let total_volume = total_transaction_volume(ctx).await?;
    let total_fees = total_transaction_fees(ctx).await?;

    Ok(TransactionStats {
        total_count,
        successful_count,
        pending_count,
        failed_count,
        total_volume,
        total_fees,
    })
}

pub async fn transfer(
    ctx: &AppContext,
    source: Option<u32>,
    destination: u32,
    amount: i64,
    txn_type: TransactionType,
    notes: Option<TransactionNote>,
) -> Result<Transaction> {
    let (txn, events) = Ledger::transfer(
        &ctx.db,
        source,
        destination,
        amount,
        &txn_type,
        notes.as_ref(),
    )
    .await
    .context("Transfer Error")?;

    DomainEventDispatcher::dispatch_events(ctx, events)?;

    Ok(txn)
}

pub async fn reverse(ctx: &AppContext, id: String) -> Result<Transaction> {
    let (txn, events) = Ledger::reverse(&ctx.db, &id)
        .await
        .context("Transfer Error")?;

    DomainEventDispatcher::dispatch_events(ctx, events)?;

    Ok(txn)
}

#[derive(serde::Serialize)]
pub struct TransactionStats {
    pub total_count: u64,
    pub successful_count: u64,
    pub pending_count: u64,
    pub failed_count: u64,
    pub total_volume: i64,
    pub total_fees: i64,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum LipaPaymentType {
    Paybill,
    Till,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct LipaArgs {
    pub user_phone: String,
    pub amount: i64,
    pub payment_type: LipaPaymentType,
    pub business_number: u32,
    pub account_number: Option<String>,
}

pub async fn c2b_lipa_logic(ctx: &AppContext, args: LipaArgs) -> Result<()> {
    let conn = &ctx.db;
    match args.payment_type {
        LipaPaymentType::Paybill => {
            if args.account_number.is_none() {
                bail!("Account number is required for paybill payments.");
            }
        }
        LipaPaymentType::Till => {}
    }
    let user = User::get_user_by_phone(conn, &args.user_phone)
        .await
        .context(format!(
            "Failed to get user with phone number: {}",
            args.user_phone
        ))?;

    if user.is_none() {
        bail!("User with phone number {} not found.", args.user_phone);
    }

    let user = user.unwrap();

    let (validation_url, confirmation_url, response_type, business_id, notes) =
        match args.payment_type {
            LipaPaymentType::Paybill => {
                let paybill = PaybillAccount::get_by_paybill_number(conn, args.business_number)
                    .await
                    .context({
                        format!(
                            "Failed to get paybill with business number: {}",
                            args.business_number
                        )
                    })?;

                if paybill.is_none() {
                    bail!(
                        "Paybill with business number {} not found.",
                        args.business_number
                    );
                }

                let paybill = paybill.unwrap();

                (
                    paybill.validation_url,
                    paybill.confirmation_url,
                    paybill.response_type,
                    paybill.business_id,
                    TransactionNote::PaybillPayment {
                        paybill_number: paybill.paybill_number,
                        bill_ref_number: args.account_number.clone().unwrap_or_default(),
                    },
                )
            }
            LipaPaymentType::Till => {
                let till = TillAccount::get_by_till_number(conn, args.business_number)
                    .await
                    .context(format!(
                        "Failed to get till will number {}",
                        args.business_number
                    ))?;

                if till.is_none() {
                    bail!(
                        "Till account with business number {} not found.",
                        args.business_number
                    );
                }

                let till = till.unwrap();

                (
                    till.validation_url,
                    till.confirmation_url,
                    till.response_type,
                    till.business_id,
                    TransactionNote::TillPayment {
                        till_number: till.till_number,
                    },
                )
            }
        };

    let destination = utility_accounts::UtilityAccount::find_by_business_id(conn, business_id)
        .await?
        .context(format!(
            "Failed to get utility account for business id: {}",
            business_id
        ))?;

    let business = Business::get_by_id(conn, business_id)
        .await?
        .context(format!("Failed to get business by id: {}", business_id))?;

    let user_account = Account::get_account(conn, user.account_id)
        .await
        .context("Failed to get user account.")?;

    if user_account.is_none() {
        bail!("User account not found");
    }

    let source = user_account.unwrap();

    let fee = get_fee(
        conn,
        match args.payment_type {
            LipaPaymentType::Paybill => &TransactionType::Paybill,
            LipaPaymentType::Till => &TransactionType::BuyGoods,
        },
        args.amount,
    )
    .await
    .context("Failed to compute transaction fee")?;

    let total = args.amount + fee;
    if source.balance < total {
        bail!(TransactionEngineError::InsufficientFunds);
    }

    tokio::spawn(process_lipa(
        conn.clone(),
        ProcessLipaArgs {
            user,
            source,
            destination,
            amount: args.amount,
            confirmation_url,
            validation_url,
            response_type,
            payment_type: args.payment_type,
            bill_ref_number: args.account_number,
            business_id,
            notes,
            business,
        },
        ctx.clone(),
    ));

    Ok(())
}

pub async fn lipa(ctx: &AppContext, args: LipaArgs) -> Result<()> {
    c2b_lipa_logic(ctx, args).await
}

struct ProcessLipaArgs {
    user: User,
    source: Account,
    destination: UtilityAccount,
    amount: i64,
    confirmation_url: Option<String>,
    validation_url: Option<String>,
    response_type: Option<ResponseType>,
    payment_type: LipaPaymentType,
    bill_ref_number: Option<String>,
    business_id: u32,
    notes: TransactionNote,
    business: Business,
}

async fn process_lipa<C: sea_orm::ConnectionTrait>(
    conn: C,
    args: ProcessLipaArgs,
    ctx: AppContext,
) {
    let ids = Identifiers::new();

    if let Err(e) = ctx
        .request_lifecycle_manager
        .create_system_request(
            &ids,
            "c2b_lipa",
            crate::request_lifecycle::RequestType::C2bLipa,
            None,
            Some(args.business_id),
            Some(args.user.account_id),
        )
        .await
    {
        tracing::error!("Failed to create system request for c2b_lipa: {}", e);
    }

    let parts: Vec<&str> = args.user.name.split_whitespace().collect();
    let first_name;
    let mut middle_name = "";
    let mut last_name = "";

    if parts.len() >= 3 {
        first_name = parts[0];
        middle_name = parts[1];
        last_name = parts[2];
    } else if parts.len() == 2 {
        first_name = parts[0];
        last_name = parts[1];
    } else {
        first_name = parts[0]
    }

    let msisdn = mask_msisdn_ke(&args.user.phone);
    let mut third_party_transaction_id = String::new();
    let validation_url = args.validation_url.filter(|url| !url.is_empty());

    if let Some(validation_url) = validation_url {
        let req: reqwest::RequestBuilder = reqwest::Client::new()
            .post(validation_url.to_string())
            .json(&ValidationRequest {
                transaction_type: match args.payment_type {
                    LipaPaymentType::Paybill => C2bTransactionType::PayBill,
                    LipaPaymentType::Till => C2bTransactionType::Till,
                },
                transaction_id: ids.transaction_id.to_string(),
                transaction_amount: format!("{:.2}", args.amount as f64 / 100.0),
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                middle_name: middle_name.to_string(),
                third_party_transaction_id: third_party_transaction_id.to_string(),
                transaction_time: timestamp(),
                business_shortcode: args.business.short_code.clone(),
                bill_ref_number: args.bill_ref_number.clone().unwrap_or_default(),
                invoice_number: String::new(),
                org_account_balance: format!("{:.2}", args.destination.balance as f64 / 100.0),
                msisdn: msisdn.to_string(),
            });

        let res = tokio::time::timeout(Duration::from_secs(8), req.send()).await;

        match res {
            Err(_) => {
                if let Some(ResponseType::Cancelled) = args.response_type {
                    return;
                }
            }
            Ok(res) => match res {
                Ok(res) => {
                    let response_text = res.text().await.unwrap_or_default();
                    match serde_json::from_str::<ValidationResponse>(&response_text) {
                        Ok(response) => {
                            third_party_transaction_id =
                                response.third_party_trans_id.unwrap_or_default();
                        }
                        Err(err) => {
                            if let Some(project) = projects::db::Entity::find()
                                .filter(projects::db::Column::BusinessId.eq(args.business_id))
                                .one(&conn)
                                .await
                                .unwrap_or_default()
                            {
                                let _ = ApiLog::builder()
                                    .project_id(project.id)
                                    .method("POST".to_string())
                                    .path(validation_url.clone())
                                    .status_code(422)
                                    .error_desc(format!(
                                        "Failed to deserialize validation response: {}. Body: {}",
                                        err, response_text
                                    ))
                                    .duration(0)
                                    .save(&conn, &ids)
                                    .await;
                            }

                            return;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error validation URL ({}): {}", validation_url, err);
                    if let Some(ResponseType::Cancelled) = args.response_type {
                        return;
                    }
                }
            },
        }
    }

    let txn_res = match ctx
        .transfer(TransferConfig {
            identifiers: ids.clone(),
            delay: Duration::ZERO,
            source: Some(args.source.id),
            destination: args.destination.account_id,
            amount: args.amount,
            txn_type: match args.payment_type {
                LipaPaymentType::Paybill => TransactionType::Paybill,
                LipaPaymentType::Till => TransactionType::BuyGoods,
            },
            notes: Some(args.notes),
        })
        .await
    {
        Ok(Ok((txn, events))) => {
            let _ = DomainEventDispatcher::dispatch_events(&ctx, events);
            txn
        }
        Ok(Err(err)) => {
            eprintln!("Transaction error: {err}");
            return;
        }
        Err(err) => {
            eprintln!("Failed to initiate transaction: {err}");
            return;
        }
    };
    let confirmation_url = args.confirmation_url.filter(|url| !url.is_empty());

    if let Some(confirmation_url) = &confirmation_url {
        let destination = Account::get_account(&conn, args.destination.account_id)
            .await
            .expect("Failed to fetch business utility account")
            .expect(
                "Expected business account to exist during the entire lifetime of the transaction",
            );

        let req: reqwest::RequestBuilder = reqwest::Client::new()
            .post(confirmation_url.to_string())
            .json(&ValidationRequest {
                transaction_type: match args.payment_type {
                    LipaPaymentType::Paybill => C2bTransactionType::PayBill,
                    LipaPaymentType::Till => C2bTransactionType::Till,
                },
                transaction_id: txn_res.id.to_string(),
                transaction_amount: format!("{:.2}", args.amount as f64 / 100.0),
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                middle_name: middle_name.to_string(),
                third_party_transaction_id,
                transaction_time: timestamp(),
                business_shortcode: args.business.short_code,
                bill_ref_number: args.bill_ref_number.unwrap_or_default(),
                invoice_number: String::new(),
                org_account_balance: format!("{:.2}", destination.balance as f64 / 100.0),
                msisdn,
            });

        let _ = tokio::time::timeout(Duration::from_secs(8), req.send()).await;
    }
}

pub fn mask_middle(s: &str, keep_prefix: usize, keep_suffix: usize, mask_char: char) -> String {
    let len = s.chars().count();
    if len <= keep_prefix + keep_suffix {
        return s.to_string();
    }

    let prefix: String = s.chars().take(keep_prefix).collect();
    let suffix: String = s
        .chars()
        .rev()
        .take(keep_suffix)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let mid_len = len - keep_prefix - keep_suffix;

    let masked_mid: String = std::iter::repeat_n(mask_char, mid_len).collect();
    format!("{prefix}{masked_mid}{suffix}")
}

pub fn mask_msisdn_ke(msisdn: &str) -> String {
    mask_middle(msisdn, 5, 3, '*')
}

fn timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y%m%d%H%M%S").to_string()
}
