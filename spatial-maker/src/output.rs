//! Output module for saving stereo images in various formats and generating MV-HEVC
//!
//! Supports:
//! - Side-by-side (SBS) stereo images (JPEG, PNG)
//! - Top-and-bottom stereo images (JPEG, PNG)
//! - Separate left/right image files
//! - Optional MV-HEVC encoding via the `spatial` CLI tool
//!
//! # Examples
//!
//! ```no_run
//! use spatial_maker::{save_stereo_image, OutputFormat, OutputOptions};
//! use image::DynamicImage;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let left = DynamicImage::new_rgb8(640, 480);
//! let right = DynamicImage::new_rgb8(640, 480);
//!
//! // Save as side-by-side JPEG
//! save_stereo_image(
//!     &left,
//!     &right,
//!     Path::new("output.jpg"),
//!     OutputOptions::default(),
//! )?;
//! # Ok(())
//! # }
//! ```

use crate::error::{SpatialError, SpatialResult};
use image::DynamicImage;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Output format for stereo images
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    /// Side-by-side stereo (left | right)
    SideBySide,
    /// Top-and-bottom stereo (left above right)
    TopAndBottom,
    /// Individual left and right files with _L and _R suffixes
    Separate,
}

impl OutputFormat {
    /// Get the canonical format name
    pub fn name(&self) -> &'static str {
        match self {
            OutputFormat::SideBySide => "side-by-side",
            OutputFormat::TopAndBottom => "top-and-bottom",
            OutputFormat::Separate => "separate",
        }
    }
}

/// Image encoding format
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageEncoding {
    /// JPEG format (lossy)
    Jpeg { quality: u8 },
    /// PNG format (lossless)
    Png,
}

impl ImageEncoding {
    /// Get file extension for this encoding
    pub fn extension(&self) -> &'static str {
        match self {
            ImageEncoding::Jpeg { .. } => "jpg",
            ImageEncoding::Png => "png",
        }
    }

    /// Detect encoding from file extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "png" => ImageEncoding::Png,
            _ => ImageEncoding::Jpeg { quality: 95 },
        }
    }
}

/// Configuration for MV-HEVC encoding via `spatial` CLI
#[derive(Clone, Debug)]
pub struct MVHEVCConfig {
    /// Path to the `spatial` CLI tool (defaults to "spatial" in PATH)
    pub spatial_cli_path: Option<PathBuf>,

    /// Enable MV-HEVC encoding
    pub enabled: bool,

    /// Quality/bitrate parameter (1-100, where 100 is highest quality)
    pub quality: u8,

    /// Whether to keep the intermediate stereo image after HEVC encoding
    pub keep_intermediate: bool,
}

impl Default for MVHEVCConfig {
    fn default() -> Self {
        Self {
            spatial_cli_path: None,
            enabled: false,
            quality: 95,
            keep_intermediate: false,
        }
    }
}

/// Options for saving stereo images
#[derive(Clone, Debug)]
pub struct OutputOptions {
    /// Stereo layout format
    pub layout: OutputFormat,

    /// Image encoding format
    pub image_format: ImageEncoding,

    /// Optional MV-HEVC encoding configuration
    pub mvhevc: Option<MVHEVCConfig>,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            layout: OutputFormat::SideBySide,
            image_format: ImageEncoding::Jpeg { quality: 95 },
            mvhevc: None,
        }
    }
}

/// Save a stereo pair to disk in the specified format
///
/// # Arguments
///
/// * `left` - Left stereo image
/// * `right` - Right stereo image
/// * `output_path` - Path to save the output image
/// * `options` - Output configuration (format, compression, etc.)
///
/// # Returns
///
/// Returns Ok on success. If MV-HEVC encoding is enabled, an intermediate
/// side-by-side image may be created and removed if `keep_intermediate` is false.
///
/// # Errors
///
/// Returns an error if:
/// - Image dimensions are incompatible
/// - File I/O fails
/// - Image encoding fails
/// - The `spatial` CLI tool is not found (for MV-HEVC)
pub fn save_stereo_image(
    left: &DynamicImage,
    right: &DynamicImage,
    output_path: impl AsRef<Path>,
    options: OutputOptions,
) -> SpatialResult<()> {
    let output_path = output_path.as_ref();

    tracing::info!("💾 Saving stereo image to {:?}", output_path);

    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            SpatialError::ImageError(format!("Failed to create output directory: {}", e))
        })?;
    }

    // Generate stereo image based on layout
    match options.layout {
        OutputFormat::SideBySide => {
            save_side_by_side(left, right, output_path, options.image_format)?;
        }
        OutputFormat::TopAndBottom => {
            save_top_and_bottom(left, right, output_path, options.image_format)?;
        }
        OutputFormat::Separate => {
            save_separate(left, right, output_path, options.image_format)?;
        }
    }

    // Handle optional MV-HEVC encoding
    if let Some(mvhevc_config) = options.mvhevc {
        if mvhevc_config.enabled {
            encode_mvhevc(output_path, &mvhevc_config)?;
            if !mvhevc_config.keep_intermediate {
                if let Err(e) = std::fs::remove_file(output_path) {
                    tracing::warn!("Failed to remove intermediate stereo image: {}", e);
                }
            }
        }
    }

    tracing::info!("✅ Stereo image saved to {:?}", output_path);
    Ok(())
}

/// Create and save a side-by-side stereo image (left | right)
fn save_side_by_side(
    left: &DynamicImage,
    right: &DynamicImage,
    output_path: &Path,
    encoding: ImageEncoding,
) -> SpatialResult<()> {
    let left_width = left.width();
    let left_height = left.height();
    let right_width = right.width();
    let right_height = right.height();

    // Ensure both images have the same height
    if left_height != right_height {
        return Err(SpatialError::ImageError(format!(
            "Left and right images must have the same height: {} != {}",
            left_height, right_height
        )));
    }

    let combined_width = left_width + right_width;
    let combined_height = left_height;

    let mut combined = DynamicImage::new_rgb8(combined_width, combined_height);

    // Paste left image
    image::imageops::overlay(&mut combined, left, 0, 0);

    // Paste right image
    image::imageops::overlay(&mut combined, right, left_width as i64, 0);

    save_image(&combined, output_path, encoding)
}

/// Create and save a top-and-bottom stereo image (top: left, bottom: right)
fn save_top_and_bottom(
    left: &DynamicImage,
    right: &DynamicImage,
    output_path: &Path,
    encoding: ImageEncoding,
) -> SpatialResult<()> {
    let left_width = left.width();
    let left_height = left.height();
    let right_width = right.width();
    let right_height = right.height();

    // Ensure both images have the same width
    if left_width != right_width {
        return Err(SpatialError::ImageError(format!(
            "Left and right images must have the same width: {} != {}",
            left_width, right_width
        )));
    }

    let combined_width = left_width;
    let combined_height = left_height + right_height;

    let mut combined = DynamicImage::new_rgb8(combined_width, combined_height);

    // Paste left image at top
    image::imageops::overlay(&mut combined, left, 0, 0);

    // Paste right image at bottom
    image::imageops::overlay(&mut combined, right, 0, left_height as i64);

    save_image(&combined, output_path, encoding)
}

/// Save left and right images as separate files with _L and _R suffixes
fn save_separate(
    left: &DynamicImage,
    right: &DynamicImage,
    output_path: &Path,
    encoding: ImageEncoding,
) -> SpatialResult<()> {
    let stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| SpatialError::ImageError("Invalid output path".to_string()))?;

    let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    let ext = encoding.extension();

    let left_path = parent.join(format!("{}_L.{}", stem, ext));
    let right_path = parent.join(format!("{}_R.{}", stem, ext));

    save_image(left, &left_path, encoding)?;
    save_image(right, &right_path, encoding)?;

    tracing::info!("✅ Separate images saved:");
    tracing::info!("   Left:  {:?}", left_path);
    tracing::info!("   Right: {:?}", right_path);

    Ok(())
}

/// Save an image with the specified encoding
fn save_image(image: &DynamicImage, path: &Path, encoding: ImageEncoding) -> SpatialResult<()> {
    match encoding {
        ImageEncoding::Jpeg { quality } => {
            let rgb_image = image.to_rgb8();
            let file = std::fs::File::create(path).map_err(|e| {
                SpatialError::ImageError(format!("Failed to create output file: {}", e))
            })?;

            let mut jpeg_encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
            jpeg_encoder
                .encode(
                    rgb_image.as_ref(),
                    rgb_image.width(),
                    rgb_image.height(),
                    image::ColorType::Rgb8,
                )
                .map_err(|e| SpatialError::ImageError(format!("Failed to encode JPEG: {}", e)))?;
        }
        ImageEncoding::Png => {
            image
                .save(path)
                .map_err(|e| SpatialError::ImageError(format!("Failed to save PNG: {}", e)))?;
        }
    }

    Ok(())
}

/// Encode stereo image to MV-HEVC using the `spatial` CLI
fn encode_mvhevc(stereo_path: &Path, config: &MVHEVCConfig) -> SpatialResult<()> {
    tracing::info!("🎬 Encoding MV-HEVC with `spatial` CLI");

    let spatial_path = config
        .spatial_cli_path
        .as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| Path::new("spatial"));

    // Determine output path (replace extension with .heic)
    let hevc_path = stereo_path.with_extension("heic");

    // Build the command
    let mut cmd = Command::new(spatial_path);
    cmd.arg("encode")
        .arg("--input")
        .arg(stereo_path)
        .arg("--output")
        .arg(&hevc_path)
        .arg("--quality")
        .arg(config.quality.to_string());

    tracing::debug!("Running: {:?}", cmd);

    // Execute the command
    let output = cmd.output().map_err(|e| {
        SpatialError::ImageError(format!(
            "Failed to run `spatial` CLI: {}. Ensure the `spatial` tool is installed and in PATH.",
            e
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpatialError::ImageError(format!(
            "MV-HEVC encoding failed: {}",
            stderr
        )));
    }

    tracing::info!("✅ MV-HEVC encoded to {:?}", hevc_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    fn create_test_image(width: u32, height: u32, color: (u8, u8, u8)) -> DynamicImage {
        let img = ImageBuffer::from_fn(width, height, |_, _| {
            image::Rgb([color.0, color.1, color.2])
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_output_format_name() {
        assert_eq!(OutputFormat::SideBySide.name(), "side-by-side");
        assert_eq!(OutputFormat::TopAndBottom.name(), "top-and-bottom");
        assert_eq!(OutputFormat::Separate.name(), "separate");
    }

    #[test]
    fn test_image_encoding_detection() {
        assert_eq!(
            ImageEncoding::from_path("test.jpg"),
            ImageEncoding::Jpeg { quality: 95 }
        );
        assert_eq!(ImageEncoding::from_path("test.png"), ImageEncoding::Png);
        assert_eq!(
            ImageEncoding::from_path("test.unknown"),
            ImageEncoding::Jpeg { quality: 95 }
        );
    }

    #[test]
    fn test_image_encoding_extension() {
        assert_eq!(ImageEncoding::Jpeg { quality: 95 }.extension(), "jpg");
        assert_eq!(ImageEncoding::Png.extension(), "png");
    }

    #[test]
    fn test_save_side_by_side_jpeg() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_sbs.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_side_by_side(
            &left,
            &right,
            &output_path,
            ImageEncoding::Jpeg { quality: 95 },
        );
        assert!(result.is_ok());
        assert!(output_path.exists());

        let loaded = image::open(&output_path).unwrap();
        assert_eq!(loaded.width(), 200);
        assert_eq!(loaded.height(), 100);
    }

    #[test]
    fn test_save_side_by_side_png() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_sbs.png");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_side_by_side(&left, &right, &output_path, ImageEncoding::Png);
        assert!(result.is_ok());
        assert!(output_path.exists());

        let loaded = image::open(&output_path).unwrap();
        assert_eq!(loaded.width(), 200);
        assert_eq!(loaded.height(), 100);
    }

    #[test]
    fn test_save_side_by_side_height_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_sbs.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 50, (0, 255, 0));

        let result = save_side_by_side(
            &left,
            &right,
            &output_path,
            ImageEncoding::Jpeg { quality: 95 },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_save_top_and_bottom_jpeg() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_tb.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_top_and_bottom(
            &left,
            &right,
            &output_path,
            ImageEncoding::Jpeg { quality: 95 },
        );
        assert!(result.is_ok());
        assert!(output_path.exists());

        let loaded = image::open(&output_path).unwrap();
        assert_eq!(loaded.width(), 100);
        assert_eq!(loaded.height(), 200);
    }

    #[test]
    fn test_save_top_and_bottom_png() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_tb.png");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_top_and_bottom(&left, &right, &output_path, ImageEncoding::Png);
        assert!(result.is_ok());
        assert!(output_path.exists());

        let loaded = image::open(&output_path).unwrap();
        assert_eq!(loaded.width(), 100);
        assert_eq!(loaded.height(), 200);
    }

    #[test]
    fn test_save_top_and_bottom_width_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test_tb.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(50, 100, (0, 255, 0));

        let result = save_top_and_bottom(
            &left,
            &right,
            &output_path,
            ImageEncoding::Jpeg { quality: 95 },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_save_separate_jpeg() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_separate(
            &left,
            &right,
            &output_path,
            ImageEncoding::Jpeg { quality: 95 },
        );
        assert!(result.is_ok());

        let left_path = temp_dir.path().join("test_L.jpg");
        let right_path = temp_dir.path().join("test_R.jpg");
        assert!(left_path.exists());
        assert!(right_path.exists());
    }

    #[test]
    fn test_save_separate_png() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("test.png");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let result = save_separate(&left, &right, &output_path, ImageEncoding::Png);
        assert!(result.is_ok());

        let left_path = temp_dir.path().join("test_L.png");
        let right_path = temp_dir.path().join("test_R.png");
        assert!(left_path.exists());
        assert!(right_path.exists());
    }

    #[test]
    fn test_save_stereo_image_sbs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("output.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let options = OutputOptions {
            layout: OutputFormat::SideBySide,
            image_format: ImageEncoding::Jpeg { quality: 95 },
            mvhevc: None,
        };

        let result = save_stereo_image(&left, &right, &output_path, options);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_save_stereo_image_tb() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("output.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let options = OutputOptions {
            layout: OutputFormat::TopAndBottom,
            image_format: ImageEncoding::Jpeg { quality: 90 },
            mvhevc: None,
        };

        let result = save_stereo_image(&left, &right, &output_path, options);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_save_stereo_image_separate() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("output.jpg");

        let left = create_test_image(100, 100, (255, 0, 0));
        let right = create_test_image(100, 100, (0, 255, 0));

        let options = OutputOptions {
            layout: OutputFormat::Separate,
            image_format: ImageEncoding::Jpeg { quality: 95 },
            mvhevc: None,
        };

        let result = save_stereo_image(&left, &right, &output_path, options);
        assert!(result.is_ok());

        let left_path = temp_dir.path().join("output_L.jpg");
        let right_path = temp_dir.path().join("output_R.jpg");
        assert!(left_path.exists());
        assert!(right_path.exists());
    }

    #[test]
    fn test_mvhevc_config_default() {
        let config = MVHEVCConfig::default();
        assert_eq!(config.spatial_cli_path, None);
        assert!(!config.enabled);
        assert_eq!(config.quality, 95);
        assert!(!config.keep_intermediate);
    }

    #[test]
    fn test_output_options_default() {
        let options = OutputOptions::default();
        assert_eq!(options.layout, OutputFormat::SideBySide);
        assert_eq!(options.image_format, ImageEncoding::Jpeg { quality: 95 });
        assert!(options.mvhevc.is_none());
    }
}
