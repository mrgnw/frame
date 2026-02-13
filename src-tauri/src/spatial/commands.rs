use std::collections::HashMap;
use std::path::PathBuf;

use futures_util::StreamExt;
use tauri::{command, AppHandle, Emitter};

use crate::spatial::error::SpatialError;
use crate::spatial::manager::{SpatialManager, SpatialMessage};
use crate::spatial::types::{
    ModelDownloadCompletePayload, ModelDownloadErrorPayload, ModelDownloadProgressPayload,
    SpatialConfig, SpatialTask,
};

fn get_checkpoint_dir() -> PathBuf {
    dirs_next().join("checkpoints")
}

fn dirs_next() -> PathBuf {
    if let Ok(val) = std::env::var("SPATIAL_MAKER_CHECKPOINTS") {
        return PathBuf::from(val);
    }
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    home.join(".spatial-maker")
}

fn encoder_to_checkpoint(encoder_size: &str) -> Option<(&'static str, &'static str)> {
    match encoder_size {
        "s" => Some((
            "depth_anything_v2_vits.pth",
            "https://huggingface.co/depth-anything/Depth-Anything-V2-Small/resolve/main/depth_anything_v2_vits.pth",
        )),
        "m" => Some((
            "depth_anything_v2_vitb.pth",
            "https://huggingface.co/depth-anything/Depth-Anything-V2-Base/resolve/main/depth_anything_v2_vitb.pth",
        )),
        "l" => Some((
            "depth_anything_v2_vitl.pth",
            "https://huggingface.co/depth-anything/Depth-Anything-V2-Large/resolve/main/depth_anything_v2_vitl.pth",
        )),
        _ => None,
    }
}

#[command]
pub async fn check_spatial_models() -> Result<HashMap<String, bool>, SpatialError> {
    let checkpoint_dir = get_checkpoint_dir();
    let mut result = HashMap::new();
    for size in &["s", "m", "l"] {
        if let Some((filename, _)) = encoder_to_checkpoint(size) {
            let exists = checkpoint_dir.join(filename).exists();
            result.insert(size.to_string(), exists);
        }
    }
    Ok(result)
}

#[command]
pub async fn download_spatial_model(
    app: AppHandle,
    encoder_size: String,
) -> Result<(), SpatialError> {
    let (filename, url) = encoder_to_checkpoint(&encoder_size).ok_or_else(|| {
        SpatialError::InvalidInput(format!("Invalid encoder size: {}", encoder_size))
    })?;

    let checkpoint_dir = get_checkpoint_dir();
    std::fs::create_dir_all(&checkpoint_dir).map_err(|e| {
        SpatialError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create checkpoint dir: {}", e),
        ))
    })?;

    let dest = checkpoint_dir.join(filename);
    let temp_dest = checkpoint_dir.join(format!("{}.downloading", filename));

    let response = reqwest::get(url).await.map_err(|e| {
        let _ = app.emit(
            "spatial-model-download-error",
            ModelDownloadErrorPayload {
                encoder_size: encoder_size.clone(),
                error: e.to_string(),
            },
        );
        SpatialError::Shell(format!("Download failed: {}", e))
    })?;

    let total_bytes = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&temp_dest).map_err(SpatialError::Io)?;

    use std::io::Write;
    let mut last_emit_pct: f64 = -1.0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            let _ = std::fs::remove_file(&temp_dest);
            SpatialError::Shell(format!("Download stream error: {}", e))
        })?;
        file.write_all(&chunk).map_err(|e| {
            let _ = std::fs::remove_file(&temp_dest);
            SpatialError::Io(e)
        })?;
        downloaded += chunk.len() as u64;

        let pct = if total_bytes > 0 {
            (downloaded as f64 / total_bytes as f64 * 100.0).round()
        } else {
            0.0
        };

        if pct != last_emit_pct {
            last_emit_pct = pct;
            let _ = app.emit(
                "spatial-model-download-progress",
                ModelDownloadProgressPayload {
                    encoder_size: encoder_size.clone(),
                    bytes_downloaded: downloaded,
                    total_bytes,
                    progress: pct,
                },
            );
        }
    }

    drop(file);
    std::fs::rename(&temp_dest, &dest).map_err(|e| {
        let _ = std::fs::remove_file(&temp_dest);
        SpatialError::Io(e)
    })?;

    let _ = app.emit(
        "spatial-model-download-complete",
        ModelDownloadCompletePayload {
            encoder_size: encoder_size.clone(),
        },
    );

    Ok(())
}

#[command]
pub async fn queue_spatial(
    manager: tauri::State<'_, SpatialManager>,
    id: String,
    file_path: String,
    config: SpatialConfig,
) -> Result<(), SpatialError> {
    if file_path.is_empty() {
        return Err(SpatialError::InvalidInput("File path is empty".to_string()));
    }

    if !std::path::Path::new(&file_path).exists() {
        return Err(SpatialError::InvalidInput(format!(
            "File not found: {}",
            file_path
        )));
    }

    let valid_sizes = ["s", "m", "l"];
    if !valid_sizes.contains(&config.encoder_size.as_str()) {
        return Err(SpatialError::InvalidInput(format!(
            "Invalid encoder size '{}'. Must be s, m, or l",
            config.encoder_size
        )));
    }

    let task = SpatialTask {
        id,
        file_path,
        config,
    };

    manager
        .sender
        .send(SpatialMessage::Enqueue(task))
        .await
        .map_err(|e| SpatialError::Channel(e.to_string()))?;
    Ok(())
}

#[command]
pub async fn cancel_spatial(
    manager: tauri::State<'_, SpatialManager>,
    id: String,
) -> Result<(), SpatialError> {
    manager.cancel_task(&id)
}
