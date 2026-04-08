pub mod api_client;
pub mod callback;
pub mod context;
pub mod emitter;
pub mod events;
pub mod macros;
pub mod runner;
pub mod tests;
pub mod ui;

#[cfg(test)]
mod test {
    use std::{sync::Arc, time::Duration};

    use tempfile::TempDir;

    pub use super::runner::{
        SELF_TEST_FINISH, SELF_TEST_PLAN, SELF_TEST_START, SELF_TEST_STEP_UPDATE,
    };
    use crate::{
        AppEventManager,
        self_test::{context::SELF_TEST_PROGRESS_LOG, runner::TestRunner},
    };

    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const GREEN: &str = "\x1b[32m";
    const DIM: &str = "\x1b[2m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const CYAN: &str = "\x1b[36m";
    // const MAGENTA: &str = "\x1b[35m";

    struct MyTestEmitter {}

    impl AppEventManager for MyTestEmitter {
        fn emit_all(&self, event: &str, payload: serde_json::Value) -> anyhow::Result<()> {
            match event {
                SELF_TEST_START => {
                    let mode: &str = payload.get("mode").unwrap().as_str().unwrap();
                    let working_dir: &str = payload.get("working_dir").unwrap().as_str().unwrap();

                    println!("{GREEN}{BOLD}self-test{RESET}{RESET} {mode} at {working_dir}");
                }
                SELF_TEST_PLAN => {
                    println!("{GREEN}{BOLD}self-test{RESET}{RESET} Running the following:");
                    let steps = payload.get("steps").unwrap().as_array().unwrap();

                    for step in steps {
                        let name = step.get("name").unwrap().as_str().unwrap();
                        let description = step.get("description").unwrap().as_str().unwrap();

                        println!(" - {CYAN}{BOLD}{name}{RESET}{RESET} {description}");
                    }
                }
                SELF_TEST_FINISH => {
                    let state: &str = payload.get("status").unwrap().as_str().unwrap();
                    let color = match state {
                        "failed" | "panicked" => RED,
                        "timedout" => YELLOW,
                        _ => GREEN,
                    };

                    println!("{color}{BOLD}self-test{RESET}{RESET} finished with {state}");
                }
                SELF_TEST_STEP_UPDATE => {
                    let status: &str = payload.get("status").unwrap().as_str().unwrap();
                    let message: &str = payload.get("message").unwrap().as_str().unwrap();
                    let name: &str = payload.get("name").unwrap().as_str().unwrap();

                    println!(" {CYAN}{BOLD}{name}{RESET}{RESET} [{status}] {message}");
                }
                SELF_TEST_PROGRESS_LOG => {
                    let runner: bool = payload.get("runner").unwrap().as_bool().unwrap();
                    let message: &str = payload.get("message").unwrap().as_str().unwrap();
                    let name: &str = payload.get("name").unwrap().as_str().unwrap();

                    if runner {
                        println!("{GREEN}{BOLD}self-test{RESET}{RESET} {message}");
                    } else {
                        println!("  {DIM}{BOLD}{name}{RESET} {message}");
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }

    #[tokio::test]
    pub async fn run() {
        let ui_emitter = Arc::new(MyTestEmitter {});
        let app_root = TempDir::new().unwrap();

        let runner = TestRunner::new(Duration::from_secs(60), Duration::from_secs(30));
        runner
            .run_suite(
                crate::self_test::context::TestMode::Fresh,
                ui_emitter,
                app_root.path().to_owned(),
            )
            .await
            .unwrap();
    }
}
