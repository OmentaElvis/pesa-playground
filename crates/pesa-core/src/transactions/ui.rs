use std::time::Duration;

use super::db;
use super::Ledger;
use super::Transaction;
use super::TransactionEngineError;
use super::TransactionType;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_query::ExprTrait;
use serde::Deserialize;

use crate::accounts::paybill_accounts::PaybillAccount;
use crate::accounts::till_accounts::TillAccount;
use crate::accounts::user_profiles::User;
use crate::accounts::Account;
use crate::api_logs::ApiLog;
use crate::events::DomainEventDispatcher;
use crate::projects;
use crate::server::api::c2b::C2bTransactionType;
use crate::server::api::c2b::ResponseType;
use crate::server::api::c2b::ValidationRequest;
use crate::server::api::c2b::ValidationResponse;
use crate::transaction_costs::get_fee;
use crate::AppContext;

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionFilter {
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for TransactionFilter {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            transaction_type: None,
            status: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

pub async fn get_transaction(
    ctx: &AppContext,
    transaction_id: String,
) -> Result<Option<Transaction>> {
    let db = &ctx.db;
    let transaction = db::Entity::find_by_id(transaction_id)
        .one(db)
        .await
        .context("Failed to get transaction")?;

    Ok(transaction.map(|t| t.into()))
}

pub async fn list_system_transactions(
    ctx: &AppContext,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Transaction>> {
    let db = &ctx.db;
    let mut query = crate::transactions::db::Entity::find();

    if let Some(limit) = limit {
        query = query.limit(limit as u64);
    }

    if let Some(offset) = offset {
        query = query.offset(offset as u64);
    }

    query = query.filter(Condition::any().and(crate::transactions::db::Column::From.is_null()));

    let transactions = query.all(db).await.context("Failed to list transactions")?;

    Ok(transactions.into_iter().map(|t| t.into()).collect())
}

pub async fn list_transactions(
    ctx: &AppContext,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>> {
    let db = &ctx.db;
    let mut query = crate::transactions::db::Entity::find();

    match (filter.from, filter.to) {
        (Some(from), Some(to)) => {
            query = query.filter(
                Condition::any()
                    .add(crate::transactions::db::Column::From.eq(from))
                    .add(crate::transactions::db::Column::To.eq(to)),
            )
        }
        (Some(from), None) => {
            query = query.filter(crate::transactions::db::Column::From.eq(from));
        }
        (None, Some(to)) => {
            query = query.filter(crate::transactions::db::Column::To.eq(to));
        }
        (None, None) => {}
    }

    if let Some(transaction_type) = filter.transaction_type {
        query = query.filter(crate::transactions::db::Column::TransactionType.eq(transaction_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(crate::transactions::db::Column::Status.eq(status));
    }

    if let Some(limit) = filter.limit {
        query = query.limit(limit as u64);
    }

    if let Some(offset) = filter.offset {
        query = query.offset(offset as u64);
    }

    let transactions = query.all(db).await.context("Failed to list transactions")?;

    Ok(transactions.into_iter().map(|t| t.into()).collect())
}

pub async fn count_transactions(ctx: &AppContext, filter: TransactionFilter) -> Result<u64> {
    let db = &ctx.db;
    let mut query = crate::transactions::db::Entity::find();

    if let Some(from) = filter.from {
        query = query.filter(crate::transactions::db::Column::From.eq(from));
    }
    if let Some(to) = filter.to {
        query = query.filter(crate::transactions::db::Column::To.eq(to));
    }
    if let Some(transaction_type) = filter.transaction_type {
        query = query.filter(crate::transactions::db::Column::TransactionType.eq(transaction_type));
    }
    if let Some(status) = filter.status {
        query = query.filter(crate::transactions::db::Column::Status.eq(status));
    }

    query
        .count(db)
        .await
        .context("Failed to count transactions")
}

async fn total_transaction_volume(ctx: &AppContext) -> Result<i64> {
    let db = &ctx.db;
    let res: i64 = crate::transactions::db::Entity::find()
        .select_only()
        .column_as(crate::transactions::db::Column::Amount.sum(), "sum")
        .into_tuple()
        .one(db)
        .await?
        .map(|val: (Option<i64>,)| val.0)
        .unwrap_or_default()
        .unwrap_or_default();

    Ok(res)
}

async fn total_transaction_fees(ctx: &AppContext) -> Result<i64> {
    let db = &ctx.db;
    let res: i64 = crate::transactions::db::Entity::find()
        .select_only()
        .column_as(crate::transactions::db::Column::Fee.sum(), "sum")
        .into_tuple()
        .one(db)
        .await?
        .map(|val: (Option<i64>,)| val.0)
        .unwrap_or_default()
        .unwrap_or_default();

    Ok(res)
}

pub async fn get_transaction_by_checkout_request(
    ctx: &AppContext,
    checkout_request_id: String,
) -> Result<Option<Transaction>> {
    let db = &ctx.db;
    let transaction = crate::transactions::db::Entity::find()
        .filter(crate::transactions::db::Column::Id.eq(checkout_request_id))
        .one(db)
        .await
        .context("Failed to get transaction by checkout request ID")?;

    Ok(transaction.map(|t| t.into()))
}

pub async fn get_user_transactions(
    ctx: &AppContext,
    user_id: u32,
    limit: Option<u32>,
    offset: Option<u32>,
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
    limit: Option<u32>,
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
        status: Some("SUCCESS".to_string()),
        ..filter.clone()
    };

    let successful_count = count_transactions(ctx, successful_filter)
        .await
        .context("Failed to get successful count")?;

    let pending_filter = TransactionFilter {
        status: Some("PENDING".to_string()),
        ..filter.clone()
    };

    let pending_count = count_transactions(ctx, pending_filter)
        .await
        .context("Failed to get pending count")?;

    let failed_filter = TransactionFilter {
        status: Some("FAILED".to_string()),
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
) -> Result<Transaction> {
    let (txn, events) = Ledger::transfer(&ctx.db, source, destination, amount, &txn_type)
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

#[derive(Deserialize)]
pub enum LipaPaymentType {
    Paybill,
    Till,
}

#[derive(Deserialize)]
pub struct LipaArgs {
    pub user_phone: String,
    pub amount: i64,
    pub payment_type: LipaPaymentType,
    pub business_number: u32,
    pub account_number: Option<String>,
}

pub async fn c2b_lipa_logic(ctx: &AppContext, args: LipaArgs) -> Result<()> {
    let conn = &ctx.db;
    // validate the different paths of payment.
    match args.payment_type {
        LipaPaymentType::Paybill => {
            // must have business and account number
            if args.account_number.is_none() {
                bail!("Account number is required for paybill payments.");
            }
        }
        LipaPaymentType::Till => {}
    }
    // get user
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

    let (account_id, validation_url, confirmation_url, response_type, business_id) =
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
                    paybill.account_id,
                    paybill.validation_url,
                    paybill.confirmation_url,
                    paybill.response_type,
                    paybill.business_id,
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
                    till.account_id,
                    till.validation_url,
                    till.confirmation_url,
                    till.response_type,
                    till.business_id,
                )
            }
        };
    let account = Account::get_account(conn, account_id)
        .await
        .context(format!(
            "An error occured gettint account with id {}.",
            account_id
        ))?;

    if account.is_none() {
        bail!("Account {} not found", account_id);
    }

    let destination = account.unwrap();

    let user_account = Account::get_account(conn, user.account_id)
        .await
        .context("Failed to get user account.")?;

    if user_account.is_none() {
        bail!("User account not found");
    }

    let source = user_account.unwrap();

    // pre calculate amount and balance
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

    // from here we can handle the payment from a background thread.
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
    destination: Account,
    amount: i64,
    confirmation_url: Option<String>,
    validation_url: Option<String>,
    response_type: Option<ResponseType>,
    payment_type: LipaPaymentType,
    bill_ref_number: Option<String>,
    business_id: u32,
}

async fn process_lipa<C: ConnectionTrait>(conn: C, args: ProcessLipaArgs, ctx: AppContext) {
    let trasaction_id = Ledger::generate_receipt();

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

    if let Some(validation_url) = args.validation_url {
        // send the validation request
        let req: reqwest::RequestBuilder = reqwest::Client::new()
            .post(validation_url.to_string())
            .json(&ValidationRequest {
                // TODO confirm that the actual mpesa prod environment sends an empty transaction type
                transaction_type: match args.payment_type {
                    LipaPaymentType::Paybill => C2bTransactionType::PayBill,
                    LipaPaymentType::Till => C2bTransactionType::Till,
                },
                // TODO confirm that a transaction_id is different for validation and confirmation request.
                transaction_id: trasaction_id.to_string(),
                transaction_amount: format!("{}", args.amount as f64 / 100.0),
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                middle_name: middle_name.to_string(),
                third_party_transaction_id: third_party_transaction_id.to_string(),
                transaction_time: timestamp(),
                business_shortcode: String::new(),
                bill_ref_number: args.bill_ref_number.clone().unwrap_or_default(),
                invoice_number: String::new(),
                // TODO org balance does not seem to be in validation request in sandbox, but we will send it anyway
                org_account_balance: format!("{}", args.destination.balance as f64 / 100.0),
                msisdn: msisdn.to_string(),
            });

        // do the validation timeout of 8 seconds
        let res = tokio::time::timeout(Duration::from_secs(8), req.send()).await;

        match res {
            Err(_) => {
                // The request timed out.
                // do the default request
                if let Some(ResponseType::Cancelled) = args.response_type {
                    return;
                }
            }
            Ok(res) => {
                // Response from the external server
                match res {
                    Ok(res) => {
                        let response_text = res.text().await.unwrap_or_default();
                        match serde_json::from_str::<ValidationResponse>(&response_text) {
                            Ok(response) => {
                                third_party_transaction_id =
                                    response.third_party_trans_id.unwrap_or_default();
                            }
                            Err(err) => {
                                // TODO: Find a better method to show errors like this to the ui directly.
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
                                            err,
                                            response_text
                                        ))
                                        .duration(0) // We dont have this yet
                                        .save(&conn)
                                        .await;
                                }

                                // Cancel the transaction
                                return;
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error validation URL ({}): {}", validation_url, err);
                        // treat as the timeout error
                        if let Some(ResponseType::Cancelled) = args.response_type {
                            return;
                        }
                    }
                }
            }
        }
    }

    let txn_res = match Ledger::transfer(
        &conn,
        Some(args.source.id),
        args.destination.id,
        args.amount,
        match args.payment_type {
            LipaPaymentType::Paybill => &TransactionType::Paybill,
            LipaPaymentType::Till => &TransactionType::BuyGoods,
        },
    )
    .await
    {
        Ok((txn, events)) => {
            let _ = DomainEventDispatcher::dispatch_events(&ctx, events);
            txn
        }
        Err(err) => {
            eprintln!("Transaction error: {err}");
            return;
        }
    };

    // send the confirmation request.
    if let Some(confirmation_url) = &args.confirmation_url {
        let req: reqwest::RequestBuilder = reqwest::Client::new()
            .post(confirmation_url.to_string())
            .json(&ValidationRequest {
                transaction_type: match args.payment_type {
                    LipaPaymentType::Paybill => C2bTransactionType::PayBill,
                    LipaPaymentType::Till => C2bTransactionType::Till,
                },
                transaction_id: txn_res.id.to_string(),
                transaction_amount: format!("{}", args.amount as f64 / 100.0),
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                middle_name: middle_name.to_string(),
                third_party_transaction_id,
                transaction_time: timestamp(),
                business_shortcode: String::new(),
                bill_ref_number: args.bill_ref_number.unwrap_or_default(),
                invoice_number: String::new(),
                org_account_balance: format!("{}", args.destination.balance as f64 / 100.0),
                msisdn,
            });

        let _ = tokio::time::timeout(Duration::from_secs(8), req.send()).await;
        // we will just stop here since the confirmation response doesnt matter at this point
    }
}

/// Mask the middle of a string (e.g., phone/MSISDN), keeping a prefix and suffix visible.
/// If the string is too short to have a middle section, it’s returned unchanged.
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
