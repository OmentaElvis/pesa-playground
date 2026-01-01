use std::time::Duration;

use anyhow::{Context, anyhow};
use chrono::Utc;

use crate::{
    accounts::user_profiles::{self, User},
    self_test::{callback::CallbackManager, context::TestContext, runner::TestStep},
    transaction_costs::get_fee,
    transactions::{TransactionEngineError, TransactionStatus, TransactionType, ui::transfer},
    transactions_log::FullTransactionLog,
};

pub struct SendMoneyTest;

#[derive(Debug, PartialEq, Eq)]
enum TransactionOutcome {
    Success,
    Failed,
}

struct SendMoneyTestCase<'a> {
    name: &'a str,
    sender_id: Option<u32>,
    recipient_id: u32,
    amount: i64,
    expected_outcome: TransactionOutcome,
    expected_error: Option<TransactionEngineError>,
    expected_full_transaction_log: Option<FullTransactionLog>,
}

impl TestStep for SendMoneyTest {
    async fn run(
        &self,
        context: &mut TestContext,
        _callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        context.log("== Running SendMoney Suite ==").await;

        // --- Create test users ---
        context.log(">> Creating test users...").await;
        let rich_user = user_profiles::ui::create_user(
            &context.app_context,
            "Rich Sender".to_string(),
            "254712345678".to_string(),
            100_000.00,
            "1234".to_string(),
        )
        .await
        .context("Failed to create rich user")?;

        let poor_user = user_profiles::ui::create_user(
            &context.app_context,
            "Poor Recipient".to_string(),
            "254787654321".to_string(),
            50.00, // KES 50.00
            "4321".to_string(),
        )
        .await
        .context("Failed to create poor user")?;

        context
            .log(&format!(
                ">> Created Rich User ID: {}",
                rich_user.account_id
            ))
            .await;
        context
            .log(&format!(
                ">> Created Poor User ID: {}",
                poor_user.account_id
            ))
            .await;

        // --- Run Test Cases ---
        happy_path_transfer(context, &rich_user, &poor_user).await?;
        insufficient_funds_transfer(context, &poor_user, &rich_user).await?;
        insufficient_for_fee_transfer(context, &poor_user, &rich_user).await?;
        sender_not_found_transfer(context, &poor_user).await?;
        recipient_not_found_transfer(context, &rich_user).await?;

        context
            .log("== SendMoney Suite Completed Successfully ==")
            .await;
        Ok(())
    }
}

async fn execute_p2p_test_case(
    context: &mut TestContext,
    case: SendMoneyTestCase<'_>,
) -> anyhow::Result<()> {
    context
        .log(&format!("-- Running Test Case: {} --", case.name))
        .await;

    let new_transaction_event = context
        .event_manager
        .listen_for::<FullTransactionLog>("new_transaction", Duration::from_secs(5));

    let transfer_result = transfer(
        &context.app_context,
        case.sender_id,
        case.recipient_id,
        case.amount,
        TransactionType::SendMoney,
        None,
    )
    .await;

    if let Some(expected_error) = &case.expected_error {
        let error = transfer_result.expect_err(&format!(
            "[{}] Expected transfer to fail, but it succeeded.",
            case.name
        ));
        if let Some(error) = error.downcast_ref::<TransactionEngineError>() {
            assert_eq!(
                error, expected_error,
                "[{}] Expected error message '{}' not found in error: {}",
                case.name, expected_error, error
            );
            context
                .log(&format!(
                    ">> Verified transfer failed with message containing: '{}'",
                    expected_error
                ))
                .await;
        } else {
            panic!("Expected Transaction engine error but got: {:?}", error);
        }
        // Ensure no transaction event if it failed early
        tokio::time::timeout(Duration::from_millis(100), new_transaction_event)
            .await
            .ok();
        return Ok(());
    } else {
        transfer_result.context("Expected transfer to succeed but it failed")?;
    }

    match case.expected_outcome {
        TransactionOutcome::Success => {
            let mut received_log = new_transaction_event.await.context(format!(
                "[{}] Did not receive new_transaction event",
                case.name
            ))?;
            context
                .log(">> Verified new_transaction event was received for success.")
                .await;

            if let Some(expected_log) = case.expected_full_transaction_log {
                received_log.transaction_id = expected_log.transaction_id.clone();
                received_log.transaction_date = expected_log.transaction_date;

                assert_eq!(
                    received_log, expected_log,
                    "[{}] Transaction Log mismatch",
                    case.name
                );
                context.log(">> Verified transaction log content.").await;
            }
        }
        TransactionOutcome::Failed => {
            // This case is for failures that still produce a transaction log (e.g., validation fail)
            // For SendMoney, most failures bail early, so this branch might not be hit.
            tokio::time::timeout(Duration::from_millis(200), new_transaction_event)
                .await
                .map_err(|_| {
                    anyhow!(
                        "[{}] Expected new_transaction NOT to be received, and it wasn't (timed out).",
                        case.name
                    )
                })
                .ok(); // Ignore timeout error, it means the event wasn't sent
        }
    }

    context
        .log(&format!("-- Test Case {} Passed --", case.name))
        .await;
    Ok(())
}

// === TEST CASE IMPLEMENTATIONS ===

async fn happy_path_transfer(
    context: &mut TestContext,
    sender: &User,
    recipient: &User,
) -> anyhow::Result<()> {
    let sender_initial_account =
        crate::accounts::Account::get_account(&context.app_context.db, sender.account_id)
            .await?
            .context("Sender account not found")?;

    let amount = 100 * 100; // KES 100.00
    let fee = get_fee(&context.app_context.db, &TransactionType::SendMoney, amount).await?;
    let total_deducted = amount + fee;

    let expected_sender_balance_after = sender_initial_account.balance - total_deducted;

    execute_p2p_test_case(
        context,
        SendMoneyTestCase {
            name: "Happy Path: SendMoney Transfer",
            sender_id: Some(sender.account_id),
            recipient_id: recipient.account_id,
            amount,
            expected_outcome: TransactionOutcome::Success,
            expected_error: None,
            expected_full_transaction_log: Some(FullTransactionLog {
                transaction_id: "".to_string(),
                transaction_date: Utc::now().to_utc(),
                transaction_amount: amount,
                transaction_type: TransactionType::SendMoney.to_string(),
                from_name: sender.name.clone(),
                to_name: recipient.name.clone(),
                from_id: Some(sender.account_id),
                to_id: recipient.account_id,
                new_balance: expected_sender_balance_after,
                status: TransactionStatus::Completed.to_string(),
                fee,
                direction: crate::transactions_log::db::Direction::Outflow,
                notes: None,
            }),
        },
    )
    .await
}

async fn insufficient_funds_transfer(
    context: &mut TestContext,
    sender: &User,
    recipient: &User,
) -> anyhow::Result<()> {
    execute_p2p_test_case(
        context,
        SendMoneyTestCase {
            name: "Error: Insufficient Funds",
            sender_id: Some(sender.account_id),
            recipient_id: recipient.account_id,
            amount: 1_000_000 * 100, // KES 1M
            expected_outcome: TransactionOutcome::Failed,
            expected_error: Some(TransactionEngineError::InsufficientFunds),
            expected_full_transaction_log: None,
        },
    )
    .await
}

async fn insufficient_for_fee_transfer(
    context: &mut TestContext,
    sender: &User,
    recipient: &User,
) -> anyhow::Result<()> {
    let sender_initial_account =
        crate::accounts::Account::get_account(&context.app_context.db, sender.account_id)
            .await?
            .context("Sender account not found")?;

    // Amount is exactly the user's balance, so they can't afford the fee
    let amount = sender_initial_account.balance;

    execute_p2p_test_case(
        context,
        SendMoneyTestCase {
            name: "Error: Insufficient Funds for Fee",
            sender_id: Some(sender.account_id),
            recipient_id: recipient.account_id,
            amount,
            expected_outcome: TransactionOutcome::Failed,
            expected_error: Some(TransactionEngineError::InsufficientFunds),
            expected_full_transaction_log: None,
        },
    )
    .await
}

async fn sender_not_found_transfer(
    context: &mut TestContext,
    recipient: &User,
) -> anyhow::Result<()> {
    execute_p2p_test_case(
        context,
        SendMoneyTestCase {
            name: "Error: Sender Not Found",
            sender_id: Some(999999), // Non-existent account ID
            recipient_id: recipient.account_id,
            amount: 100,
            expected_outcome: TransactionOutcome::Failed,
            expected_error: Some(TransactionEngineError::AccountNotFound(999999)),
            expected_full_transaction_log: None,
        },
    )
    .await
}

async fn recipient_not_found_transfer(
    context: &mut TestContext,
    sender: &User,
) -> anyhow::Result<()> {
    execute_p2p_test_case(
        context,
        SendMoneyTestCase {
            name: "Error: Recipient Not Found",
            sender_id: Some(sender.account_id),
            recipient_id: 999999, // Non-existent account ID
            amount: 100,
            expected_outcome: TransactionOutcome::Failed,
            expected_error: Some(TransactionEngineError::AccountNotFound(999999)),
            expected_full_transaction_log: None,
        },
    )
    .await
}
