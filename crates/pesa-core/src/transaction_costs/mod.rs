pub mod db;
pub mod ui;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
};

use crate::transactions::TransactionType;

use self::db::{ActiveModel as TransactionCostActiveModel, Entity};

pub async fn get_fee<C>(
    db: &C,
    transaction_type: &TransactionType,
    amount: i64,
) -> Result<i64, DbErr>
where
    C: ConnectionTrait,
{
    let txn_type = match transaction_type {
        TransactionType::Paybill | TransactionType::BuyGoods | TransactionType::SendMoney => {
            TransactionType::SendMoney.to_string()
        }
        TransactionType::Withdraw => TransactionType::Withdraw.to_string(),
        TransactionType::Deposit => TransactionType::Deposit.to_string(),
        TransactionType::Airtime => TransactionType::Airtime.to_string(),
        TransactionType::Reversal => TransactionType::Reversal.to_string(),
        TransactionType::Unknown(s) => s.to_string(),
    };

    let amount_in_kes = amount / 100;
    let rule = Entity::find()
        .filter(db::Column::TransactionType.eq(txn_type))
        .filter(db::Column::MinAmount.lte(amount_in_kes))
        .filter(db::Column::MaxAmount.gte(amount_in_kes))
        .one(db)
        .await?;

    if let Some(rule) = rule {
        let mut fee = 0;
        if let Some(fixed) = rule.fee_fixed {
            fee += fixed * 100; // Convert fixed fee to cents
        }
        if let Some(percentage) = rule.fee_percentage {
            fee += (amount as f64 * percentage / 100.0).round() as i64;
        }
        return Ok(fee);
    }

    Ok(0)
}

pub async fn init_default_costs<C>(db: &C) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    if Entity::find().count(db).await? == 0 {
        let default_costs = vec![
            // Withdraw From M-PESA Agent
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1),
                max_amount: sea_orm::ActiveValue::Set(49),
                fee_fixed: sea_orm::ActiveValue::Set(Some(0)), // Free
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(50),
                max_amount: sea_orm::ActiveValue::Set(100),
                fee_fixed: sea_orm::ActiveValue::Set(Some(11)), // 11.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(101),
                max_amount: sea_orm::ActiveValue::Set(500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(29)), // 29.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(501),
                max_amount: sea_orm::ActiveValue::Set(1000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(29)), // 29.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1001),
                max_amount: sea_orm::ActiveValue::Set(1500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(29)), // 29.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1501),
                max_amount: sea_orm::ActiveValue::Set(2500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(29)), // 29.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(2501),
                max_amount: sea_orm::ActiveValue::Set(3500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(52)), // 52.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(3501),
                max_amount: sea_orm::ActiveValue::Set(5000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(69)), // 69.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(5001),
                max_amount: sea_orm::ActiveValue::Set(7500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(87)), // 87.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(7501),
                max_amount: sea_orm::ActiveValue::Set(10000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(115)), // 115.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(10001),
                max_amount: sea_orm::ActiveValue::Set(15000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(167)), // 167.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(15001),
                max_amount: sea_orm::ActiveValue::Set(20000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(185)), // 185.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(20001),
                max_amount: sea_orm::ActiveValue::Set(35000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(197)), // 197.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(35001),
                max_amount: sea_orm::ActiveValue::Set(50000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(278)), // 278.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Withdraw.to_string()),
                min_amount: sea_orm::ActiveValue::Set(50001),
                max_amount: sea_orm::ActiveValue::Set(150000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(309)), // 309.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            // Send to other M-PESA Users, Pochi La Biashara and Business Till To customer
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1),
                max_amount: sea_orm::ActiveValue::Set(100),
                fee_fixed: sea_orm::ActiveValue::Set(Some(0)), // Free
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(101),
                max_amount: sea_orm::ActiveValue::Set(500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(7)), // 7.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(501),
                max_amount: sea_orm::ActiveValue::Set(1000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(13)), // 13.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1001),
                max_amount: sea_orm::ActiveValue::Set(1500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(23)), // 23.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1501),
                max_amount: sea_orm::ActiveValue::Set(2500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(33)), // 33.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(2501),
                max_amount: sea_orm::ActiveValue::Set(3500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(53)), // 53.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(3501),
                max_amount: sea_orm::ActiveValue::Set(5000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(57)), // 57.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(5001),
                max_amount: sea_orm::ActiveValue::Set(7500),
                fee_fixed: sea_orm::ActiveValue::Set(Some(78)), // 78.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(7501),
                max_amount: sea_orm::ActiveValue::Set(10000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(90)), // 90.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(10001),
                max_amount: sea_orm::ActiveValue::Set(15000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(100)), // 100.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(15001),
                max_amount: sea_orm::ActiveValue::Set(20000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(105)), // 105.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(20001),
                max_amount: sea_orm::ActiveValue::Set(35000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(108)), // 108.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(35001),
                max_amount: sea_orm::ActiveValue::Set(50000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(108)), // 108.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::SendMoney.to_string()),
                min_amount: sea_orm::ActiveValue::Set(50001),
                max_amount: sea_orm::ActiveValue::Set(150000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(108)), // 108.00 KES
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
            // Deposit (usually free)
            TransactionCostActiveModel {
                transaction_type: sea_orm::ActiveValue::Set(TransactionType::Deposit.to_string()),
                min_amount: sea_orm::ActiveValue::Set(1),
                max_amount: sea_orm::ActiveValue::Set(150000),
                fee_fixed: sea_orm::ActiveValue::Set(Some(0)), // KES 0.00
                fee_percentage: sea_orm::ActiveValue::Set(None),
                ..Default::default()
            },
        ];

        for cost in default_costs {
            TransactionCostActiveModel::insert(cost, db).await?;
        }
    }
    Ok(())
}
