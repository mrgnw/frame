use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::spatial::error::SpatialError;
use crate::spatial::manager::SpatialMessage;
use crate::spatial::types::{
    SpatialCompletedPayload, SpatialLogPayload, SpatialProgressPayload, SpatialStartedPayload,
    SpatialTask,
};

pub async fn run_spatial_worker(
    app: AppHandle,
    tx: mpsc::Sender<SpatialMessage>,
    task: SpatialTask,
) -> Result<(), SpatialError> {
    let id = task.id.clone();

    let mut cmd = Command::new("uv");
    cmd.arg("tool")
        .arg("run")
        .arg("spatial-maker")
        .arg(&task.file_path)
        .arg("--json-progress")
        .arg("--encoder")
        .arg(&task.config.encoder_size)
        .arg("--max-disparity")
        .arg(task.config.max_disparity.to_string());

    if task.config.skip_downscale {
        cmd.arg("--skip-downscale");
    }

    if let Some(duration) = task.config.duration {
        cmd.arg("--duration").arg(duration.to_string());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| SpatialError::Shell(format!("Failed to spawn spatial-maker: {}", e)))?;

    let pid = child.id().unwrap_or(0);
    let _ = tx
        .send(SpatialMessage::TaskStarted(id.clone(), pid))
        .await;
    let _ = app.emit(
        "spatial-started",
        SpatialStartedPayload { id: id.clone() },
    );
    let _ = app.emit(
        "spatial-progress",
        SpatialProgressPayload {
            id: id.clone(),
            progress: 0.0,
            stage: "starting".to_string(),
        },
    );

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| SpatialError::Worker("Failed to capture stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| SpatialError::Worker("Failed to capture stderr".to_string()))?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let app_stdout = app.clone();
    let id_stdout = id.clone();

    let stdout_handle = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let line: String = line;
            let _ = app_stdout.emit(
                "spatial-log",
                SpatialLogPayload {
                    id: id_stdout.clone(),
                    line: line.clone(),
                },
            );

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                match json.get("event").and_then(|v| v.as_str()) {
                    Some("stage") => {
                        let stage = json
                            .get("stage")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let progress = match stage {
                            "downscale" => 5.0,
                            "depth_stereo" => 10.0,
                            "audio_mux" => 85.0,
                            "spatial_make" => 90.0,
                            _ => 0.0,
                        };
                        let _ = app_stdout.emit(
                            "spatial-progress",
                            SpatialProgressPayload {
                                id: id_stdout.clone(),
                                progress,
                                stage: stage.to_string(),
                            },
                        );
                    }
                    Some("progress") => {
                        if let Some(pct) = json.get("pct").and_then(|v| v.as_f64()) {
                            // depth_stereo is 10-85% of total
                            let mapped = 10.0 + (pct / 100.0) * 75.0;
                            let _ = app_stdout.emit(
                                "spatial-progress",
                                SpatialProgressPayload {
                                    id: id_stdout.clone(),
                                    progress: mapped,
                                    stage: "depth_stereo".to_string(),
                                },
                            );
                        }
                    }
                    Some("done") => {
                        let output = json
                            .get("output")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let _ = app_stdout.emit(
                            "spatial-completed",
                            SpatialCompletedPayload {
                                id: id_stdout.clone(),
                                output_path: output,
                            },
                        );
                    }
                    Some("error") => {
                        let msg = json
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        let _ = app_stdout.emit(
                            "spatial-log",
                            SpatialLogPayload {
                                id: id_stdout.clone(),
                                line: format!("[SPATIAL ERROR] {}", msg),
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
    });

    let app_stderr = app.clone();
    let id_stderr = id.clone();

    let stderr_handle = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let line: String = line;
            let _ = app_stderr.emit(
                "spatial-log",
                SpatialLogPayload {
                    id: id_stderr.clone(),
                    line: line.clone(),
                },
            );
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| SpatialError::Worker(format!("Failed to wait for process: {}", e)))?;

    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(SpatialError::Worker(format!(
            "spatial-maker exited with code {}",
            code
        )))
    }
}
