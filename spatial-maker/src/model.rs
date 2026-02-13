//! Model discovery and checkpoint management
//!
//! This module handles:
//! - Locating the checkpoint directory (respects SPATIAL_MAKER_CHECKPOINTS env var)
//! - Discovering existing ONNX models
//! - Downloading models from HuggingFace if not present

use crate::error::{SpatialError, SpatialResult};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Get the checkpoint directory path
///
/// Respects the `SPATIAL_MAKER_CHECKPOINTS` environment variable if set,
/// otherwise defaults to `~/.spatial-maker/checkpoints/`
pub fn get_checkpoint_dir() -> SpatialResult<PathBuf> {
    if let Ok(custom_dir) = std::env::var("SPATIAL_MAKER_CHECKPOINTS") {
        Ok(PathBuf::from(custom_dir))
    } else {
        let home = dirs::home_dir().ok_or_else(|| {
            SpatialError::ConfigError("Could not determine home directory".to_string())
        })?;
        Ok(home.join(".spatial-maker").join("checkpoints"))
    }
}

/// Model metadata: name, size, and download URL
#[derive(Clone, Debug)]
pub struct ModelMetadata {
    pub name: String,
    pub filename: String,
    pub url: String,
    pub size_mb: u32,
}

impl ModelMetadata {
    fn for_encoder(encoder_size: &str) -> SpatialResult<Self> {
        match encoder_size {
            "s" | "small" => Ok(ModelMetadata {
                name: "depth-anything-v2-small".to_string(),
                filename: "depth_anything_v2_small.onnx".to_string(),
                url: "https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx".to_string(),
                size_mb: 99,
            }),
            "b" | "base" => Ok(ModelMetadata {
                name: "depth-anything-v2-base".to_string(),
                filename: "depth_anything_v2_base.onnx".to_string(),
                url: "https://huggingface.co/onnx-community/depth-anything-v2-base/resolve/main/onnx/model.onnx".to_string(),
                size_mb: 380,
            }),
            "l" | "large" => Ok(ModelMetadata {
                name: "depth-anything-v2-large".to_string(),
                filename: "depth_anything_v2_large.onnx".to_string(),
                url: "https://huggingface.co/onnx-community/depth-anything-v2-large/resolve/main/onnx/model.onnx".to_string(),
                size_mb: 1300,
            }),
            other => Err(SpatialError::ConfigError(
                format!("Unknown encoder size: '{}'. Use 's', 'b', or 'l'", other)
            )),
        }
    }
}

/// Find the model file for a given encoder size
///
/// Returns the path if it exists, otherwise returns an error.
/// Does not download; use `ensure_model_exists` for automatic downloads.
pub fn find_model(encoder_size: &str) -> SpatialResult<PathBuf> {
    let checkpoint_dir = get_checkpoint_dir()?;
    let metadata = ModelMetadata::for_encoder(encoder_size)?;
    let model_path = checkpoint_dir.join(&metadata.filename);

    if model_path.exists() {
        tracing::info!("Found model: {:?}", model_path);
        Ok(model_path)
    } else {
        Err(SpatialError::ModelError(format!(
            "Model not found: {:?}. Run download_model first.",
            model_path
        )))
    }
}

/// Check if a model exists for the given encoder size
pub fn model_exists(encoder_size: &str) -> bool {
    find_model(encoder_size).is_ok()
}

/// Download a model from HuggingFace if it doesn't exist
///
/// # Arguments
///
/// * `encoder_size` - Model size: "s", "b", or "l"
/// * `progress_fn` - Optional callback for progress updates: `(current_bytes, total_bytes)`
pub async fn ensure_model_exists<F>(
    encoder_size: &str,
    progress_fn: Option<F>,
) -> SpatialResult<PathBuf>
where
    F: FnMut(u64, u64),
{
    let checkpoint_dir = get_checkpoint_dir()?;
    let metadata = ModelMetadata::for_encoder(encoder_size)?;
    let model_path = checkpoint_dir.join(&metadata.filename);

    // Create checkpoint directory if it doesn't exist
    tokio::fs::create_dir_all(&checkpoint_dir)
        .await
        .map_err(|e| {
            SpatialError::IoError(format!("Failed to create checkpoint directory: {}", e))
        })?;

    // Return early if model already exists
    if model_path.exists() {
        tracing::info!("Model already exists: {:?}", model_path);
        return Ok(model_path);
    }

    tracing::info!("Downloading model: {}", metadata.name);
    download_model(&metadata, &model_path, progress_fn).await?;
    Ok(model_path)
}

/// Download a model from the given URL with progress tracking
async fn download_model<F>(
    metadata: &ModelMetadata,
    destination: &Path,
    mut progress_fn: Option<F>,
) -> SpatialResult<()>
where
    F: FnMut(u64, u64),
{
    tracing::info!("Downloading from: {}", metadata.url);

    let response = reqwest::get(&metadata.url)
        .await
        .map_err(|e| SpatialError::Other(format!("Failed to download model: {}", e)))?;

    let total_bytes = response
        .content_length()
        .unwrap_or(metadata.size_mb as u64 * 1_000_000);

    let mut file = tokio::fs::File::create(destination)
        .await
        .map_err(|e| SpatialError::IoError(format!("Failed to create file: {}", e)))?;

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| SpatialError::Other(format!("Download interrupted: {}", e)))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| SpatialError::IoError(format!("Failed to write to file: {}", e)))?;

        downloaded += chunk.len() as u64;

        if let Some(ref mut f) = progress_fn {
            f(downloaded, total_bytes);
        }

        tracing::debug!(
            "Downloaded {:.1}% ({}/{}MB)",
            (downloaded as f64 / total_bytes as f64 * 100.0),
            downloaded / 1_000_000,
            total_bytes / 1_000_000
        );
    }

    tracing::info!("Model downloaded successfully: {:?}", destination);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_metadata_small() {
        let meta = ModelMetadata::for_encoder("s").unwrap();
        assert_eq!(meta.name, "depth-anything-v2-small");
        assert!(meta.url.contains("depth-anything-v2-small"));
    }

    #[test]
    fn test_model_metadata_base() {
        let meta = ModelMetadata::for_encoder("b").unwrap();
        assert_eq!(meta.name, "depth-anything-v2-base");
    }

    #[test]
    fn test_model_metadata_large() {
        let meta = ModelMetadata::for_encoder("l").unwrap();
        assert_eq!(meta.name, "depth-anything-v2-large");
    }

    #[test]
    fn test_invalid_encoder() {
        let result = ModelMetadata::for_encoder("x");
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_dir_with_env() {
        std::env::set_var("SPATIAL_MAKER_CHECKPOINTS", "/tmp/test");
        let dir = get_checkpoint_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/test"));
        std::env::remove_var("SPATIAL_MAKER_CHECKPOINTS");
    }
}
