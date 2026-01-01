use crate::{
    AppContext,
    sandboxes::{RunningSandbox, Status},
    server::start_project_server,
};
use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::oneshot};

async fn try_bind_preferred(host: &str, port: u16) -> Option<TcpListener> {
    TcpListener::bind((host, port)).await.ok()
}

async fn bind_fallback(host: &str) -> anyhow::Result<TcpListener> {
    Ok(TcpListener::bind(format!("{}:0", host)).await?)
}

async fn bind_sandbox_port(
    project_id: u32,
    host: &str,
) -> anyhow::Result<(TcpListener, u16, bool)> {
    let preferred = 8000 + (project_id % 1000) as u16;

    if let Some(listener) = try_bind_preferred(host, preferred).await {
        return Ok((listener, preferred, true));
    }

    let listener = bind_fallback(host).await?;
    let port = listener.local_addr()?.port();

    Ok((listener, port, false))
}

pub async fn start_sandbox(
    ctx: &AppContext,
    project_id: u32,
    host: Option<String>,
) -> Result<String> {
    if let Some(s) = ctx.running.get(&project_id)
        && !s.handle.is_finished()
    {
        let addr = format!("{}:{}", s.host, s.port);
        return Ok(format!("http://{}", addr));
    }

    let host = host.unwrap_or("127.0.0.1".to_string());
    let (listener, port, derived) = bind_sandbox_port(project_id, &host)
        .await
        .context("Failed to bind to port")?;

    if !derived {
        tracing::warn!(
            project_id,
            port,
            "preferred sandbox port unavailable, using fallback"
        );
    }

    let addr = format!("{}:{}", host, port);

    ctx.event_manager.emit_all(
        "sandbox_status",
        json!({
            "project_id": project_id,
            "port": port,
            "host": host,
            "status": "starting",
        }),
    )?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let ctx_clone = ctx.clone();
    let host_clone = host.clone();
    let handle = tokio::spawn(async move {
        let server_result = start_project_server(
            project_id,
            listener,
            ctx_clone.clone(),
            shutdown_rx,
            host_clone.clone(),
            port,
        )
        .await;

        if let Err(e) = &server_result {
            let _ = ctx_clone.event_manager.emit_all(
                "sandbox_status",
                json!({
                    "project_id": project_id,
                    "port": port,
                    "host": host_clone,
                    "status": "error",
                    "error": e.to_string(),
                }),
            );
        }
        server_result
    });

    ctx.running.insert(
        project_id,
        RunningSandbox {
            shutdown: shutdown_tx,
            handle,
            port,
            host,
        },
    );

    Ok(format!("http://{}", addr))
}

pub async fn stop_sandbox(ctx: &AppContext, project_id: u32) -> Result<()> {
    let rs = if let Some((_, rs)) = ctx.running.remove(&project_id) {
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
            "host": rs.host,
            "status": "off",
        }),
    )?;

    Ok(())
}

pub async fn sandbox_status(ctx: &AppContext, project_id: u32) -> Result<Value> {
    if let Some(rs) = ctx.running.get(&project_id) {
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

    let (_, rs) = ctx.running.remove(&project_id).unwrap();

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
    let mut instances: Vec<Status> = Vec::new();
    for rs in ctx.running.iter() {
        let mut status = Status {
            project_id: *rs.key(),
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
