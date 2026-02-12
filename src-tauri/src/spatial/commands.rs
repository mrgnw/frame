use tauri::command;

use crate::spatial::error::SpatialError;
use crate::spatial::manager::{SpatialManager, SpatialMessage};
use crate::spatial::types::{SpatialConfig, SpatialTask};

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
