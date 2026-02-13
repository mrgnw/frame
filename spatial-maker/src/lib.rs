//! # spatial-maker
//!
//! A Rust library for generating spatial photos and videos using ONNX-based depth estimation.
//!
//! ## Features
//!
//! - **Depth Estimation**: Load and run ONNX Depth Anything V2 models via the `ort` crate
//! - **Stereo Generation**: Convert depth maps to stereo pairs using depth-image-based rendering (DIBR)
//! - **Model Management**: Automatic model discovery and download from HuggingFace
//! - **Photo Pipeline**: End-to-end spatial photo generation (single image)
//! - **Video Pipeline**: Frame-by-frame video processing with progress callbacks
//! - **CoreML Support**: Leverages Apple Neural Engine on macOS with CoreML execution provider
//!
//! ## Example
//!
//! ```no_run
//! use spatial_maker::{process_photo, SpatialConfig};
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = SpatialConfig::default();
//! process_photo(
//!     Path::new("input.jpg"),
//!     Path::new("output_sbs.jpg"),
//!     config,
//! ).await?;
//! # Ok(())
//! # }
//! ```

pub mod depth;
pub mod error;
pub mod model;
pub mod output;
pub mod stereo;

pub use depth::{estimate_depth, DepthConfig};
pub use error::{SpatialError, SpatialResult};
pub use model::{find_model, get_checkpoint_dir, model_exists};
pub use output::{save_stereo_image, ImageEncoding, MVHEVCConfig, OutputFormat, OutputOptions};
pub use stereo::generate_stereo_pair;

use std::path::Path;

/// Configuration for spatial photo/video processing
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpatialConfig {
    /// Depth estimation model size: "s" (small), "b" (base), or "l" (large)
    pub encoder_size: String,

    /// Maximum disparity for stereo generation (pixels)
    pub max_disparity: u32,

    /// Target input size for depth model (shorter side, in pixels)
    pub target_depth_size: u32,

    /// Whether to use CoreML execution provider on macOS (if available)
    pub use_coreml: bool,
}

/// Legacy type alias for backward compatibility
pub type StereoOutputFormat = OutputFormat;

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            encoder_size: "s".to_string(),
            max_disparity: 30,
            target_depth_size: 518,
            use_coreml: true,
        }
    }
}

/// Process a single photo: load → estimate depth → generate stereo → save
///
/// # Arguments
///
/// * `input_path` - Path to input image (JPEG, PNG, etc.)
/// * `output_path` - Path to write output stereo image
/// * `config` - Spatial processing configuration (depth estimation)
/// * `output_options` - Output format and encoding options
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Example
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use spatial_maker::{process_photo, SpatialConfig, OutputOptions};
/// use std::path::Path;
///
/// let config = SpatialConfig {
///     encoder_size: "s".to_string(),
///     max_disparity: 30,
///     ..Default::default()
/// };
/// let output_options = OutputOptions::default();
/// process_photo(
///     Path::new("photo.jpg"),
///     Path::new("spatial_photo.jpg"),
///     config,
///     output_options,
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn process_photo(
    input_path: &Path,
    output_path: &Path,
    config: SpatialConfig,
    output_options: OutputOptions,
) -> SpatialResult<()> {
    tracing::info!("📸 Processing photo: {:?}", input_path);

    // Load input image
    tracing::debug!("Loading image from {:?}", input_path);
    let input_image = image::open(input_path)
        .map_err(|e| SpatialError::ImageError(format!("Failed to load image: {}", e)))?;

    // Estimate depth
    tracing::debug!("Estimating depth with encoder: {}", config.encoder_size);
    let depth_map = estimate_depth(
        &input_image,
        &DepthConfig {
            encoder_size: config.encoder_size.clone(),
            target_size: config.target_depth_size,
            use_coreml: config.use_coreml,
        },
    )
    .await?;

    // Generate stereo pair
    tracing::debug!(
        "Generating stereo pair with max_disparity: {}",
        config.max_disparity
    );
    let (left, right) = generate_stereo_pair(&input_image, &depth_map, config.max_disparity)?;

    // Save stereo output
    tracing::info!("Saving stereo image to {:?}", output_path);
    save_stereo_image(&left, &right, output_path, output_options)?;

    tracing::info!("✅ Photo processing complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = SpatialConfig::default();
        assert_eq!(config.encoder_size, "s");
        assert_eq!(config.max_disparity, 30);
        assert_eq!(config.target_depth_size, 518);
        assert!(config.use_coreml);
    }
}
