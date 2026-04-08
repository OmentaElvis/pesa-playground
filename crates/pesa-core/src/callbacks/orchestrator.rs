use super::{CallbackLog, CreateCallbackParams, DispatchOutcome};
use crate::callbacks::dispatch::{CallbackDispatchService, DispatchConfig};
use sea_orm::ConnectionTrait;
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

pub struct CallbackOrchestrator;

impl CallbackOrchestrator {
    pub async fn handle_callback<C: ConnectionTrait, T: Serialize>(
        db: &C,
        params: CreateCallbackParams,
        payload: T,
    ) {
        // Create and save a "Pending" callback record.
        let saved_log = match CallbackLog::create(db, params).await {
            Ok(log) => log,
            Err(e) => {
                tracing::error!("Failed to insert pending callback into database: {:?}", e);
                return;
            }
        };

        // Dispatch the callback.
        let dispatch_service = CallbackDispatchService::new(DispatchConfig {
            timeout: Duration::from_secs(30),
            max_retries: 2,
        });

        let dispatch_result = dispatch_service
            .dispatch(&saved_log.callback_url, &payload)
            .await;

        // Update the database record with the outcome.
        let outcome = match dispatch_result {
            Ok(res) => DispatchOutcome::Delivered {
                status_code: res.final_status_code,
                headers: json!(
                    res.final_headers
                        .iter()
                        .map(|(name, value)| {
                            (
                                name.to_string(),
                                value.to_str().unwrap_or_default().to_string(),
                            )
                        })
                        .collect::<HashMap<String, String>>()
                ),
                body: res.final_body,
            },
            Err(e) => DispatchOutcome::Failed {
                error_message: e.to_string(),
            },
        };

        if let Err(e) = saved_log.update_dispatch_status(db, outcome).await {
            tracing::error!("Failed to update callback status in database: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::Business;
    use crate::business::CreateBusiness;
    use crate::callbacks::{CallbackStatus, CallbackType};
    use crate::projects::{CreateProject, Project};
    use crate::tests::TestDb;
    use serde_json::json;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_test_project(db: &sea_orm::DatabaseConnection) -> u32 {
        let business_input = CreateBusiness {
            name: "Test Business".to_string(),
            short_code: "123456".to_string(),
            initial_working_balance: 10000.0,
            initial_utility_balance: 5000.0,
        };
        let business = Business::create(db, business_input).await.unwrap();

        let project_input = CreateProject {
            business_id: business.id,
            name: "Test Project".to_string(),
            callback_url: Some("https://example.com/callback".to_string()),
            simulation_mode: crate::projects::SimulationMode::Realistic,
            stk_delay: 1000,
            prefix: Some("TEST".to_string()),
        };
        let project = Project::create(db, project_input).await.unwrap();
        project.id
    }

    #[tokio::test]
    async fn test_handle_callback_success() {
        let test_db = TestDb::in_memory().await.unwrap();
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let project_id = setup_test_project(&test_db.conn).await;

        let params = CreateCallbackParams {
            project_id,
            callback_type: CallbackType::StkPush,
            url: format!("{}/callback", mock_server.uri().trim_end_matches('/')),
            conversation_id: "test_conv".to_string(),
            originator_id: "originator".to_string(),
            payload: json!({"test": "data"}),
            transaction_id: Some("txn123".to_string()),
        };

        CallbackOrchestrator::handle_callback(&test_db.conn, params, json!({"result": "success"}))
            .await;

        // Verify callback was created and status updated
        let callbacks = super::CallbackLog::find_by_project(&test_db.conn, project_id)
            .await
            .unwrap();
        assert_eq!(callbacks.len(), 1);
        let callback = &callbacks[0];
        assert_eq!(callback.status, CallbackStatus::Delivered);
    }

    #[tokio::test]
    async fn test_handle_callback_failure() {
        let test_db = TestDb::in_memory().await.unwrap();
        let mock_server = MockServer::start().await;

        // No mock mounted - will cause failure
        let project_id = setup_test_project(&test_db.conn).await;

        let params = CreateCallbackParams {
            project_id,
            callback_type: CallbackType::StkPush,
            url: format!("{}/callback", mock_server.uri().trim_end_matches('/')),
            conversation_id: "test_conv".to_string(),
            originator_id: "originator".to_string(),
            payload: json!({"test": "data"}),
            transaction_id: Some("txn123".to_string()),
        };

        CallbackOrchestrator::handle_callback(&test_db.conn, params, json!({"result": "fail"}))
            .await;

        // Verify callback was created and status is Failed
        let callbacks = super::CallbackLog::find_by_project(&test_db.conn, project_id)
            .await
            .unwrap();
        assert_eq!(callbacks.len(), 1);
        let callback = &callbacks[0];
        assert_eq!(callback.status, CallbackStatus::Failed);
        assert!(callback.error.is_some());
    }
}
