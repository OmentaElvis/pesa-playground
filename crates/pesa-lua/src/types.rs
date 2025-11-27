use pesa_macros::wrap_core_types;

wrap_core_types! {
    CreateProject from pesa_core::projects,
    UpdateProject from pesa_core::projects,
    CreateBusiness from pesa_core::business,
    UpdateBusiness from pesa_core::business,
    CreatePaybillAccount from pesa_core::accounts::paybill_accounts,
    UpdatePaybillAccount from pesa_core::accounts::paybill_accounts,
    CreateTillAccount from pesa_core::accounts::till_accounts,
    UpdateTillAccount from pesa_core::accounts::till_accounts,
    TransactionFilter from pesa_core::transactions::ui,
    LipaArgs from pesa_core::transactions::ui,
    TransactionType from pesa_core::transactions,
    UpdateApiLogRequest from pesa_core::api_logs,
    ApiLogFilter from pesa_core::api_logs::ui,
    TransactionCostData from pesa_core::transaction_costs::ui,
    UserResponse from pesa_core::callbacks::stk
}
