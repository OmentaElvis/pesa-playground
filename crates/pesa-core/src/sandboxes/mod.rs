use serde::Serialize;

pub mod ui;

pub struct RunningSandbox {
    pub port: u16,
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub handle: tokio::task::JoinHandle<anyhow::Result<()>>,
}

#[derive(Serialize)]
pub struct Status {
    pub project_id: u32,
    port: u16,
    error: Option<String>,
    status: String,
}
