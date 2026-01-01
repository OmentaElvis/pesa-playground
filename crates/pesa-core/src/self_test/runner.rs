use anyhow::anyhow;
use futures::FutureExt;
use std::{
    panic::{AssertUnwindSafe, PanicHookInfo},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::timeout;
use tracing::error;

use super::{
    context::{TestContext, TestMode},
    emitter::{
        MainUiEmitter, TestFinishPayload, TestPlanPayload, TestStartPayload, TestStatus,
        TestStepInfo, TestStepUpdatePayload,
    },
};
use crate::{AppEventManager, self_test::callback::CallbackManager};

#[derive(Debug, Clone)]
pub struct PanicReport {
    pub message: String,
    pub location: Option<String>,
}

pub const SELF_TEST_START: &str = "self_test_start";
pub const SELF_TEST_PLAN: &str = "self_test_plan";
pub const SELF_TEST_STEP_UPDATE: &str = "self_test_step_update";
pub const SELF_TEST_FINISH: &str = "self_test_finish";

/// Represents a single, sequential step in the self-test suite.
///
/// Each test step is run in order, and the suite stops on the first failure.
/// Steps can share state with each other through the `TestContext`.
///
/// # Example
///
/// Here is a complete example of a test step that simulates creating a user,
/// waiting for a confirmation callback, and storing the new user's ID in the
/// shared context for the next test to use.
///
/// ```rust
/// use std::time::Duration;
/// use anyhow::{Context, bail};
/// use serde::{Serialize, Deserialize};
/// use crate::self_test::{
///     callback::CallbackManager,
///     context::TestContext,
///     runner::TestStep,
/// };
///
/// // The data expected from the application in the callback
/// #[derive(Deserialize)]
/// struct ConfirmationBody {
///     user_id: String,
///     status: String,
/// }
///
/// pub struct CreateUserTest;
///
/// // The core business logic function being tested (hypothetical)
/// async fn create_user_in_app(
///     confirmation_url: &str,
///     username: &str,
/// ) -> anyhow::Result<()> {
///     // In a real test, this would call the actual application logic
///     // which would then make an HTTP request to `confirmation_url`.
///     Ok(())
/// }
///
/// impl TestStep for CreateUserTest {
///     async fn run(
///         &self,
///         context: &mut TestContext,
///         callback_manager: &mut CallbackManager,
///     ) -> anyhow::Result<()> {
///         context.log("Starting: Create User Test").await;
///
///         // 1. Register a URL to receive a callback from the application.
///         let confirmation_callback = callback_manager
///             .register_callback::<ConfirmationBody>("/user_confirm")?;
///
///         context.log(&format!(
///             "Registered callback URL: {}",
///             confirmation_callback.url()
///         )).await;
///
///         // 2. Call the application logic that should trigger the callback.
///         create_user_in_app(confirmation_callback.url(), "test_user").await?;
///         context.log("Called app logic, now waiting for callback...").await;
///
///         // 3. Await the callback. This future has a built-in timeout.
///         // If the callback is not received in time, it will return an error.
///         let call = confirmation_callback.await?;
///         context.log("Callback received from application.").await;
///
///         // 4. Assert the contents of the callback body.f
///         if call.body.status != "confirmed" {
///             bail!("User status was not 'confirmed', got: {}", call.body.status);
///         }
///
///         // 5. Optionally, send a response back to the application.
///         call.respond(axum::http::StatusCode::OK, &serde_json::json!({"status": "ok"}), None).await?;
///         context.log("Sent response back to the application.").await;
///
///         // 6. Store data in the shared context for a subsequent test to use.
///         context.set("new_user_id", &call.body.user_id)?;
///         context.log(&format!("Stored new user ID: {}", call.body.user_id)).await;
///
///         Ok(())
///     }
/// }
/// ```
pub trait TestStep: Send + Sync + 'static {
    /// Executes the test step. It receives the shared `TestContext` and `CallbackManager`.
    fn run(
        &self,
        context: &mut TestContext,
        callback_manager: &mut CallbackManager,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

/// The main orchestrator for running the self-test suite.
pub struct TestRunner {
    test_timeout: Duration,
    callback_timeout: Duration,
}

impl TestRunner {
    pub fn new(test_timeout: Duration, callback_timeout: Duration) -> Self {
        Self {
            test_timeout,
            callback_timeout,
        }
    }

    pub async fn run_suite(
        &self,
        mode: TestMode,
        main_ui_emitter: Arc<dyn AppEventManager + Send + Sync>,
        app_root: PathBuf,
    ) -> anyhow::Result<()> {
        let emitter = MainUiEmitter::new(main_ui_emitter.clone());
        emitter.log_runner("Starting test suite execution");

        let tests = super::tests::all_tests();
        let steps: Vec<TestStepInfo> = tests
            .iter()
            .map(|test| TestStepInfo {
                name: test.name(),
                description: test.description(),
            })
            .collect();

        emitter.log_runner(&format!("Found {} tests", steps.len()));

        emitter.emit_plan(&TestPlanPayload { steps })?;

        emitter.log_runner("Initializing test context");
        let mut context = TestContext::new(
            mode.clone(),
            emitter.clone(),
            self.callback_timeout,
            app_root,
        )
        .await?;

        emitter.log_runner("Initializing test callback manager");

        let mut callback_manager = CallbackManager::new(self.callback_timeout).await?;

        emitter.emit_start(&TestStartPayload {
            mode,
            working_dir: context.app_context.app_root.to_string_lossy().to_string(),
        })?;

        let panic_report_mx = Arc::new(Mutex::new(None));

        for (index, step) in tests.iter().enumerate() {
            let step_name = step.name();
            emitter.log_runner(&format!("Running step: {}", step_name));
            context.current_test = Some((step_name.to_string(), index));

            emitter.emit_step_update(&TestStepUpdatePayload {
                index,
                name: step_name,
                status: TestStatus::Running,
                message: "".to_string(),
            })?;

            // Set a temporary panic hook just for this step
            let previous_hook = std::panic::take_hook();
            let report_for_hook = panic_report_mx.clone();
            std::panic::set_hook(Box::new(move |panic_info: &PanicHookInfo| {
                let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Panic occurred with an unknown payload type.".to_string()
                };
                let location = panic_info.location().map(|loc| loc.to_string());
                *report_for_hook.lock().unwrap() = Some(PanicReport { message, location });
            }));

            // Clear any previous panic report
            *panic_report_mx.lock().unwrap() = None;

            // Catch panics from the test step so they can be reported gracefully.
            let future =
                AssertUnwindSafe(step.run(&mut context, &mut callback_manager)).catch_unwind();
            let result = timeout(self.test_timeout, future).await;

            // Restore the original panic hook
            std::panic::set_hook(previous_hook);

            let (status, message) = match result {
                // Task timed out
                Err(_) => {
                    let msg = format!(
                        "Step '{}' timed out after {:?}.",
                        step_name, self.test_timeout
                    );
                    emitter.log_runner(&msg);
                    (TestStatus::TimedOut, msg)
                }
                // Task finished (or panicked)
                Ok(step_result) => match step_result {
                    // Task panicked
                    Err(panic_payload) => {
                        let panic_report = panic_report_mx.lock().unwrap().take();

                        let error_detail = if let Some(report) = panic_report {
                            // We have a detailed report from the hook
                            format!(
                                "{}\nLocation: {}",
                                report.message,
                                report.location.unwrap_or_else(|| "Unknown".to_string())
                            )
                        } else {
                            // Fallback to just the payload if the hook failed for some reason
                            if let Some(s) = panic_payload.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "Panic occurred with an unknown payload type.".to_string()
                            }
                        };

                        let log_msg = format!("Step '{}' panicked: {}", step_name, error_detail);
                        context.log(&log_msg).await;
                        error!("{}", log_msg);

                        (TestStatus::Panicked, error_detail)
                    }
                    // Task returned a Result
                    Ok(run_result) => match run_result {
                        // Test was successful
                        Ok(_) => {
                            context
                                .log(&format!("Step '{}' completed successfully.", step_name))
                                .await;
                            (TestStatus::Passed, "".to_string())
                        }
                        // Test returned an error
                        Err(e) => (TestStatus::Failed, format!("{:?}", e)),
                    },
                },
            };

            emitter.emit_step_update(&TestStepUpdatePayload {
                index,
                name: step_name,
                status: status.clone(),
                message,
            })?;

            // Check if the callback server has panicked
            callback_manager.check_server_panic().await?;

            // If a step fails or times out, we might want to stop the whole suite
            if !matches!(status, TestStatus::Passed) {
                emitter.emit_finish(&TestFinishPayload {
                    status: TestStatus::Failed,
                })?;

                return Err(anyhow!(
                    "Self-test suite stopped due to failure in step '{}'",
                    step_name
                ));
            }
            context.current_test = None;
        }
        context.current_test = None;
        context.log("Self-test suite completed successfully.").await;
        emitter.emit_finish(&TestFinishPayload {
            status: TestStatus::Passed,
        })?;
        Ok(())
    }
}
