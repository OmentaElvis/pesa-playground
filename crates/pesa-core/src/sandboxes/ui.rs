use crate::{
    AppContext,
    sandboxes::{RunningSandbox, Status},
    server::start_project_server,
};
use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use tokio::sync::oneshot;

pub async fn start_sandbox(ctx: &AppContext, project_id: u32) -> Result<String> {
    let port: u16 = (8000 + (project_id % 1000))
        .try_into()
        .context("Failed to create port".to_string())?;

    let addr = format!("127.0.0.1:{}", port);

    let mut running = ctx.running.lock().await;
    if running.contains_key(&project_id) {
        return Ok(format!("http://{}", addr));
    }

    ctx.event_manager.emit_all(
        "sandbox_status",
        json!({
            "project_id": project_id,
            "port": port,
            "status": "starting",
        }),
    )?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let handle = tokio::spawn(start_project_server(
        project_id,
        port,
        ctx.clone(),
        shutdown_rx,
    ));

    running.insert(
        project_id,
        RunningSandbox {
            shutdown: shutdown_tx,
            handle,
            port,
        },
    );

    Ok(format!("http://{}", addr))
}

pub async fn stop_sandbox(ctx: &AppContext, project_id: u32) -> Result<()> {
    let mut running = ctx.running.lock().await;
    let rs = if let Some(rs) = running.remove(&project_id) {
        rs
    } else {
        return Ok(());
    };

    if !rs.handle.is_finished() {
        rs.shutdown
            .send(())
            .map_err(|_| anyhow!("Failed to send shutdown signal"))?;
    }

    ctx.event_manager.emit_all(
        "sandbox_status",
        json!({
            "project_id": project_id,
            "port": rs.port,
            "status": "off",
        }),
    )?;

    Ok(())
}

pub async fn sandbox_status(ctx: &AppContext, project_id: u32) -> Result<Value> {
    let mut running = ctx.running.lock().await;
    if let Some(rs) = running.get(&project_id) {
        if !rs.handle.is_finished() {
            return Ok(json! ({
                "status": "on",
                "port": rs.port
            }));
        }
    } else {
        return Ok(json!({
            "status": "off",
            "port": 0
        }));
    }

    let rs = running.remove(&project_id).unwrap();

    match rs.handle.await {
        Err(err) => Ok(json! ({
            "status": "error",
            "port": rs.port,
            "error": err.to_string()
        })),
        Ok(res) => match res {
            Ok(()) => Ok(json!({
                "status": "off",
                "port": 0
            })),
            Err(err) => Ok(json! ({
                "status": "error",
                "port": rs.port,
                "error": err.to_string()
            })),
        },
    }
}

pub async fn list_running_sandboxes(ctx: &AppContext) -> Result<Vec<Status>> {
    let mut running = ctx.running.lock().await;

    let mut instances: Vec<Status> = Vec::new();
    for (project_id, rs) in running.iter_mut() {
        let mut status = Status {
            project_id: *project_id,
            port: rs.port,
            error: None,
            status: "on".to_string(),
        };

        if rs.handle.is_finished() {
            status.port = 0;
            status.status = "off".to_string();
        };

        instances.push(status);
    }

    Ok(instances)
}
