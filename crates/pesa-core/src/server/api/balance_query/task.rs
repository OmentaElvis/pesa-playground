use base64::{Engine, engine::general_purpose};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey};

use crate::{
    accounts::{mmf_accounts::MmfAccount, utility_accounts::UtilityAccount},
    business::Business,
    business_operators::BusinessOperator,
    projects::Project,
    server::{
        ApiError, MpesaError,
        api::{
            auth::INVALID_CREDENTIALS,
            balance_query::{
                BalanceQueryCallbackResponse, BalanceQueryRequest, BalanceQueryRequestResponse,
                BalanceQueryResultCodes,
            },
            stkpush::generate_checkout_request_id,
        },
        async_handler::PpgAsyncRequest,
    },
};

pub struct BalanceQuery {
    pub conversation_id: String,
    pub originator_conversation_id: String,
    pub result_url: String,
    pub business: Business,
    pub utility_account: UtilityAccount,
    pub mmf_account: MmfAccount,
    pub project: Project,
}

impl PpgAsyncRequest for BalanceQuery {
    type RequestData = BalanceQueryRequest;
    type SyncResponseData = BalanceQueryRequestResponse;
    type CallbackPayload = BalanceQueryCallbackResponse;
    type Error = BalanceQueryResultCodes;

    fn api_name() -> &'static str {
        "balance_query"
    }

    async fn init(
        state: &crate::server::ApiState,
        req: Self::RequestData,
        conversation_id: &str,
        _api_key: crate::api_keys::ApiKey,
    ) -> Result<(Self::SyncResponseData, Self), crate::server::ApiError>
    where
        Self: Sized,
    {
        let originator_conversation_id = generate_checkout_request_id();

        let business = Business::get_by_short_code(&state.context.db, &req.party_a)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InvalidShortcode,
                "Shortcode not found",
            ))?;
        let utility_account = UtilityAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InternalError,
                "Failed to load utility account",
            ))?;

        let mmf_account = MmfAccount::find_by_business_id(&state.context.db, business.id)
            .await
            .map_err(|error| {
                ApiError::new(
                    crate::server::MpesaError::InternalError,
                    format!("An internal error occured: {}", error),
                )
            })?
            .ok_or(ApiError::new(
                crate::server::MpesaError::InternalError,
                "Failed to load mmf account",
            ))?;

        // get the operator by username.
        let operator = BusinessOperator::find_by_business(
            &state.context.db,
            req.initiator.clone(),
            business.id,
        )
        .await
        .map_err(|error| {
            ApiError::new(
                MpesaError::InvalidCredentials,
                format!("An internal error occured: {}", error),
            )
        })?
        .ok_or(ApiError::new(
            crate::server::MpesaError::InvalidCredentials,
            "Initiator username not found",
        ))?;
        let settings: crate::settings::models::AppSettings = state.context.settings.get().await;
        let private_key = settings
            .encryption_keys
            .ok_or(ApiError::new(
                MpesaError::InternalError,
                "Settings app public and private keys have not be initialized".to_string(),
            ))?
            .private_key;

        // validate the password
        let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key).map_err(|err| {
            ApiError::new(
                MpesaError::InternalError,
                format!("Failed to load private key {}", err),
            )
        })?;

        let credential_decode = general_purpose::STANDARD
            .decode(req.security_credential)
            .map_err(|err| {
                ApiError::new(
                    MpesaError::InternalError,
                    format!("Failed to decode security credential base64: {}", err),
                )
            })?;

        let decrypted = private_key
            .decrypt(Pkcs1v15Encrypt, &credential_decode)
            .map_err(|err| {
                ApiError::new(
                    MpesaError::InvalidCredentials,
                    format!("Failed to decrypt SecurityCredential: {}", err),
                )
            })?;

        if !decrypted.eq(operator.password.as_bytes()) {
            return Err(ApiError::new(
                MpesaError::InvalidCredentials,
                "Invalid SecurityCredential",
            ));
        }
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
        let result = BalanceQueryResultCodes::Success;

        Ok((
            BalanceQueryRequestResponse {
                conversation_id: conversation_id.to_string(),
                originator_conversation_id: originator_conversation_id.clone(),
                response_code: result.code().to_string(),
                response_description: result.to_string(),
            },
            Self {
                conversation_id: conversation_id.to_string(),
                originator_conversation_id: originator_conversation_id.clone(),
                result_url: req.result_url,
                business,
                utility_account,
                mmf_account,
                project,
            },
        ))
    }

    async fn execute(
        &mut self,
        _state: &crate::server::ApiState,
    ) -> Result<Self::CallbackPayload, Self::Error> {
        Ok(self.generate_response(&BalanceQueryResultCodes::Success)) // will be converted into correct payload
    }

    fn get_callback_url(&self) -> Option<&str> {
        Some(&self.result_url)
    }

    fn get_originator_id(&self) -> &str {
        &self.originator_conversation_id
    }
}
