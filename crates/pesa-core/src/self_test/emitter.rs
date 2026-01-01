use serde::Serialize;
use std::sync::Arc;

use crate::AppEventManager;

use super::context::TestMode;

// Re-export this so it's in one place
pub use super::runner::{
    SELF_TEST_FINISH, SELF_TEST_PLAN, SELF_TEST_START, SELF_TEST_STEP_UPDATE,
};
pub use super::context::SELF_TEST_PROGRESS_LOG;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Panicked,
    TimedOut,
}

#[derive(Serialize)]
pub struct TestStartPayload {
    pub mode: TestMode,
    pub working_dir: String,
}

#[derive(Serialize)]
pub struct TestStepInfo<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Serialize)]
pub struct TestPlanPayload<'a> {
    pub steps: Vec<TestStepInfo<'a>>,
}

#[derive(Serialize)]
pub struct TestStepUpdatePayload<'a> {
    pub index: usize,
    pub name: &'a str,
    pub status: TestStatus,
    pub message: String,
}

#[derive(Serialize)]
pub struct TestFinishPayload {
    pub status: TestStatus,
}

#[derive(Serialize)]
pub struct TestProgressLogPayload<'a> {
    pub name: &'a str,
    pub index: usize,
    pub message: &'a str,
    pub runner: bool,
}

#[derive(Clone)]
pub struct MainUiEmitter {
    emitter: Arc<dyn AppEventManager + Send + Sync>,
}

impl MainUiEmitter {
    pub fn new(emitter: Arc<dyn AppEventManager + Send + Sync>) -> Self {
        Self { emitter }
    }

    fn emit(&self, event: &str, payload: impl Serialize) -> anyhow::Result<()> {
        self.emitter
            .emit_all(event, serde_json::to_value(payload)?)
    }

    pub fn emit_start(&self, payload: &TestStartPayload) -> anyhow::Result<()> {
        self.emit(SELF_TEST_START, payload)
    }

    pub fn emit_plan(&self, payload: &TestPlanPayload) -> anyhow::Result<()> {
        self.emit(SELF_TEST_PLAN, payload)
    }

    pub fn emit_step_update(&self, payload: &TestStepUpdatePayload) -> anyhow::Result<()> {
        self.emit(SELF_TEST_STEP_UPDATE, payload)
    }

    pub fn emit_finish(&self, payload: &TestFinishPayload) -> anyhow::Result<()> {
        self.emit(SELF_TEST_FINISH, payload)
    }

    pub fn log_runner(&self, message: &str) {
        let _ = self.emit(
            SELF_TEST_PROGRESS_LOG,
            &TestProgressLogPayload {
                name: "main",
                index: 0,
                message,
                runner: true,
            },
        );
    }

    pub fn log_step(&self, index: usize, name: &str, message: &str) {
        let _ = self.emit(
            SELF_TEST_PROGRESS_LOG,
            &TestProgressLogPayload {
                name,
                index,
                message,
                runner: false,
            },
        );
    }
}
