use strum::{Display, EnumString};

use serde::{Deserialize, Serialize};

pub mod register;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display, EnumString)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum ResponseType {
    Completed,
    Cancelled,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum C2bTransactionType {
    #[serde(rename = "Pay Bill")]
    PayBill,
    #[serde(rename = "Buy Goods")]
    Till,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ValidationRequest {
    #[serde(rename = "TransactionType")]
    pub transaction_type: C2bTransactionType,
    #[serde(rename = "TransID")]
    pub transaction_id: String,
    #[serde(rename = "TransTime")]
    pub transaction_time: String,
    #[serde(rename = "TransAmount")]
    pub transaction_amount: String,
    #[serde(rename = "BusinessShortCode")]
    pub business_shortcode: String,
    #[serde(rename = "BillRefNumber")]
    pub bill_ref_number: String,
    #[serde(rename = "InvoiceNumber")]
    pub invoice_number: String,
    #[serde(rename = "OrgAccountBalance")]
    pub org_account_balance: String,
    #[serde(rename = "ThirdPartyTransID")]
    pub third_party_transaction_id: String,
    #[serde(rename = "MSISDN")]
    pub msisdn: String,
    #[serde(rename = "FirstName")]
    pub first_name: String,
    #[serde(rename = "MiddleName")]
    pub middle_name: String,
    #[serde(rename = "LastName")]
    pub last_name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ResultCode {
    C2B00011,
    C2B00012,
    C2B00013,
    C2B00014,
    C2B00015,
    C2B00016,
    #[serde(rename = "0")]
    Ok,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResultResponse {
    #[serde(rename = "ResultCode")]
    pub result_code: ResultCode,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidationResponse {
    #[serde(rename = "ResultCode")]
    pub result_code: ResultCode,
    #[serde(rename = "ResultDesc")]
    pub result_desc: String,
    #[serde(rename = "ThirdPartyTransID")]
    pub third_party_trans_id: Option<String>,
}
