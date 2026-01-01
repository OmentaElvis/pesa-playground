use tokio::time::Duration;

use crate::AppContext;

use super::{context::TestMode, emitter::MainUiEmitter, runner::TestRunner};

/// Runs the full self-test suite.
///
/// It sets up the test environment, executes all steps, and reports progress via events.
pub async fn run_self_tests(app_ctx: &AppContext, mode: TestMode) -> Result<(), String> {
    // Clone the main application's event manager to report test progress back to the UI
    let ui_emitter = app_ctx.event_manager.clone();
    let app_root = app_ctx.app_root.clone();

    // Spawn a background task to run the tests to avoid blocking the caller
    tokio::spawn(async move {
        let runner = TestRunner::new(Duration::from_secs(60), Duration::from_secs(30));
        let emitter = MainUiEmitter::new(ui_emitter.clone());

        match runner.run_suite(mode, ui_emitter, app_root).await {
            Ok(_) => {
                tracing::info!("Self-test completed successfully.");
            }
            Err(e) => {
                tracing::error!("Self-test failed: {:?}", e);
                emitter.log_runner(&format!("Self test failed: {:?}", e));
            }
        }
    });

    Ok(())
}
