//! Image loading with support for multiple formats
//!
//! This module provides unified image loading across common formats.
//! For advanced formats (AVIF, JXL, HEIC), it supports both native Rust decoders
//! (via feature flags) and automatic conversion via ffmpeg as a fallback.

use crate::error::{SpatialError, SpatialResult};
use image::DynamicImage;
use std::path::Path;
use std::process::Command;

/// Load an image from disk, supporting multiple formats
///
/// Automatically detects format from file extension and uses appropriate decoder.
/// Supports natively: JPEG, PNG, GIF, BMP, TIFF, WebP
///
/// For AVIF, JXL, and HEIC formats:
/// - Tries native Rust decoder first (if feature enabled)
/// - Falls back to ffmpeg conversion if native decoder unavailable or fails
/// - Returns helpful error if neither option works
///
/// # Arguments
///
/// * `path` - Path to image file
///
/// # Returns
///
/// A `DynamicImage` in RGB format ready for processing
///
/// # Errors
///
/// Returns `SpatialError::ImageError` if:
/// - File cannot be read
/// - Format is not supported
/// - Image data is corrupted
/// - Decoder fails
/// - Conversion fails (for advanced formats)
///
/// # Examples
///
/// ```no_run
/// use spatial_maker::image_loader;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let img = image_loader::load_image(Path::new("photo.jpg")).await?;
/// println!("Loaded: {}x{}", img.width(), img.height());
/// # Ok(())
/// # }
/// ```
pub async fn load_image(path: impl AsRef<Path>) -> SpatialResult<DynamicImage> {
    let path = path.as_ref();

    // Validate file exists
    if !path.exists() {
        return Err(SpatialError::ImageError(format!(
            "Image file not found: {:?}",
            path
        )));
    }

    // Get file extension and normalize to lowercase
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| SpatialError::ImageError(format!("File has no extension: {:?}", path)))?;

    tracing::debug!("Loading image from {:?} (format: {})", path, extension);

    match extension.as_str() {
        // AVIF: Try native decoder first (if feature enabled), then ffmpeg
        "avif" => load_avif(path).await,

        // JXL: Try native decoder first (if feature enabled), then ffmpeg
        "jxl" => load_jxl(path).await,

        // HEIC: Try native decoder first (if feature enabled), then ffmpeg
        "heic" | "heif" => load_heic(path).await,

        // Standard formats supported by image crate
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" => {
            load_standard(path)
        }

        _ => Err(SpatialError::ImageError(format!(
            "Unsupported image format: .{}. Supported: JPEG, PNG, GIF, BMP, TIFF, WebP, AVIF, JXL, HEIC",
            extension
        ))),
    }
}

/// Load standard image formats (JPEG, PNG, GIF, BMP, TIFF, WebP, etc.)
fn load_standard(path: impl AsRef<Path>) -> SpatialResult<DynamicImage> {
    let path = path.as_ref();

    let img = image::open(path)
        .map_err(|e| SpatialError::ImageError(format!("Failed to load image {:?}: {}", path, e)))?;

    tracing::info!(
        "Loaded standard format image: {}x{} ({:?})",
        img.width(),
        img.height(),
        img.color()
    );

    Ok(img)
}

/// Load AVIF image (native or ffmpeg fallback)
async fn load_avif(path: &Path) -> SpatialResult<DynamicImage> {
    // Try native decoder if feature is enabled
    #[cfg(feature = "avif")]
    {
        tracing::debug!("Attempting native AVIF decoder");
        match load_avif_native(path) {
            Ok(img) => {
                tracing::info!(
                    "Loaded AVIF image using native decoder: {}x{}",
                    img.width(),
                    img.height()
                );
                return Ok(img);
            }
            Err(e) => {
                tracing::warn!("Native AVIF decoder failed: {}, falling back to ffmpeg", e);
            }
        }
    }

    // Fallback to ffmpeg conversion
    #[cfg(not(feature = "avif"))]
    tracing::debug!("Native AVIF decoder not enabled, using ffmpeg");

    load_with_conversion(path, "avif").await
}

/// Load JXL image (native or ffmpeg fallback)
async fn load_jxl(path: &Path) -> SpatialResult<DynamicImage> {
    // Try native decoder if feature is enabled
    #[cfg(feature = "jxl")]
    {
        tracing::debug!("Attempting native JXL decoder (jxl-oxide)");
        match load_jxl_native(path) {
            Ok(img) => {
                tracing::info!(
                    "Loaded JXL image using native decoder: {}x{}",
                    img.width(),
                    img.height()
                );
                return Ok(img);
            }
            Err(e) => {
                tracing::warn!("Native JXL decoder failed: {}, falling back to ffmpeg", e);
            }
        }
    }

    // Fallback to ffmpeg conversion
    #[cfg(not(feature = "jxl"))]
    tracing::debug!("Native JXL decoder not enabled, using ffmpeg");

    load_with_conversion(path, "jxl").await
}

/// Load HEIC image (native or ffmpeg fallback)
async fn load_heic(path: &Path) -> SpatialResult<DynamicImage> {
    // Try native decoder if feature is enabled
    #[cfg(feature = "heic")]
    {
        tracing::debug!("Attempting native HEIC decoder (libheif)");
        match load_heic_native(path) {
            Ok(img) => {
                tracing::info!(
                    "Loaded HEIC image using native decoder: {}x{}",
                    img.width(),
                    img.height()
                );
                return Ok(img);
            }
            Err(e) => {
                tracing::warn!("Native HEIC decoder failed: {}, falling back to ffmpeg", e);
            }
        }
    }

    // Fallback to ffmpeg conversion
    #[cfg(not(feature = "heic"))]
    tracing::debug!("Native HEIC decoder not enabled, using ffmpeg");

    load_with_conversion(path, "heic").await
}

/// Load AVIF using native decoder (requires 'avif' feature)
#[cfg(feature = "avif")]
fn load_avif_native(path: &Path) -> SpatialResult<DynamicImage> {
    // AVIF support is built into the image crate when the feature is enabled
    let img = image::open(path)
        .map_err(|e| SpatialError::ImageError(format!("Native AVIF decode failed: {}", e)))?;

    Ok(img)
}

/// Load JXL using native decoder (requires 'jxl' feature)
#[cfg(feature = "jxl")]
fn load_jxl_native(path: &Path) -> SpatialResult<DynamicImage> {
    use jxl_oxide::JxlImage;

    let data = std::fs::read(path)
        .map_err(|e| SpatialError::IoError(format!("Failed to read JXL file: {}", e)))?;

    let jxl_image = JxlImage::builder()
        .read(&data[..])
        .map_err(|e| SpatialError::ImageError(format!("JXL decode failed: {:?}", e)))?;

    let width = jxl_image.width();
    let height = jxl_image.height();

    // Render the first frame
    let render = jxl_image
        .render_frame(0)
        .map_err(|e| SpatialError::ImageError(format!("JXL render failed: {:?}", e)))?;

    // Get planar image data (Vec<FrameBuffer>)
    let planar = render.image_planar();

    if planar.is_empty() {
        return Err(SpatialError::ImageError(
            "JXL image has no color channels".to_string(),
        ));
    }

    // Build interleaved RGB data
    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;

            // Get RGB values (or replicate grayscale)
            // jxl-oxide returns f32 values in [0, 1] range - convert to u8 [0, 255]
            let r = (planar[0].buf()[idx] * 255.0).clamp(0.0, 255.0) as u8;
            let g = if planar.len() > 1 {
                (planar[1].buf()[idx] * 255.0).clamp(0.0, 255.0) as u8
            } else {
                r
            };
            let b = if planar.len() > 2 {
                (planar[2].buf()[idx] * 255.0).clamp(0.0, 255.0) as u8
            } else {
                r
            };

            rgb_data.push(r);
            rgb_data.push(g);
            rgb_data.push(b);
        }
    }

    // Create RGB image buffer
    let img_buffer = image::RgbImage::from_raw(width, height, rgb_data).ok_or_else(|| {
        SpatialError::ImageError("Failed to create image buffer from JXL data".to_string())
    })?;

    Ok(DynamicImage::ImageRgb8(img_buffer))
}

/// Load HEIC using native decoder (requires 'heic' feature)
#[cfg(feature = "heic")]
fn load_heic_native(path: &Path) -> SpatialResult<DynamicImage> {
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let lib_heif = LibHeif::new();

    let ctx = HeifContext::read_from_file(
        path.to_str()
            .ok_or_else(|| SpatialError::IoError("Invalid path encoding".to_string()))?,
    )
    .map_err(|e| SpatialError::ImageError(format!("Failed to load HEIC file: {:?}", e)))?;

    let handle = ctx.primary_image_handle().map_err(|e| {
        SpatialError::ImageError(format!("Failed to get HEIC image handle: {:?}", e))
    })?;

    let width = handle.width();
    let height = handle.height();

    // Decode to RGB
    let image = lib_heif
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .map_err(|e| SpatialError::ImageError(format!("HEIC decode failed: {:?}", e)))?;

    // Get the interleaved plane data
    let planes = image.planes();
    let interleaved = planes.interleaved.ok_or_else(|| {
        SpatialError::ImageError("No interleaved plane in HEIC image".to_string())
    })?;

    // Convert to RGB image buffer
    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        let row_start = (y * interleaved.stride as u32) as usize;
        let row_end = row_start + (width * 3) as usize;
        rgb_data.extend_from_slice(&interleaved.data[row_start..row_end]);
    }

    let img_buffer = image::RgbImage::from_raw(width, height, rgb_data).ok_or_else(|| {
        SpatialError::ImageError("Failed to create image buffer from HEIC data".to_string())
    })?;

    Ok(DynamicImage::ImageRgb8(img_buffer))
}

/// Load an image by converting it first using ffmpeg
async fn load_with_conversion(path: impl AsRef<Path>, format: &str) -> SpatialResult<DynamicImage> {
    let path = path.as_ref();

    tracing::info!(
        "Converting {} image to JPEG using ffmpeg...",
        format.to_uppercase()
    );

    // Check if ffmpeg is available
    if !is_ffmpeg_available() {
        return Err(SpatialError::ImageError(format!(
            "{} format requires either:\n\
             1. Native decoder (enable feature flag: --features {}), OR\n\
             2. ffmpeg for automatic conversion\n\
             \n\
             ffmpeg is not installed or not in PATH.\n\
             \n\
             To install ffmpeg:\n\
               macOS:   brew install ffmpeg\n\
               Ubuntu:  sudo apt-get install ffmpeg\n\
               Windows: choco install ffmpeg\n\
             \n\
             Or enable native decoder:\n\
               cargo build --features {}\n\
             \n\
             Or manually convert your file to JPEG:\n\
               ffmpeg -i {:?} -c:v libjpeg -q:v 2 output.jpg",
            format.to_uppercase(),
            format,
            format,
            path
        )));
    }

    // Create a temporary file for the converted image
    let temp_dir = std::env::temp_dir();
    let temp_filename = format!(
        "spatial_maker_convert_{}_{}.jpg",
        format,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let temp_path = temp_dir.join(temp_filename);

    // Convert the image using ffmpeg
    convert_image_with_ffmpeg(path, &temp_path, format)?;

    // Load the converted image
    let img = image::open(&temp_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        SpatialError::ImageError(format!("Failed to load converted image: {}", e))
    })?;

    tracing::info!(
        "Successfully converted and loaded {} image: {}x{}",
        format.to_uppercase(),
        img.width(),
        img.height()
    );

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_path);

    Ok(img)
}

/// Check if ffmpeg is available on the system
fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Convert an image using ffmpeg
fn convert_image_with_ffmpeg(input: &Path, output: &Path, format: &str) -> SpatialResult<()> {
    let output_str = output
        .to_str()
        .ok_or_else(|| SpatialError::IoError("Invalid output path".to_string()))?;

    let input_str = input
        .to_str()
        .ok_or_else(|| SpatialError::IoError("Invalid input path".to_string()))?;

    tracing::debug!("Converting {:?} ({}) to {:?}", input, format, output);

    let output = Command::new("ffmpeg")
        .args(&["-i", input_str])
        .args(&["-c:v", "libjpeg"])
        .args(&["-q:v", "2"]) // JPEG quality (2 = highest)
        .args(&["-y"]) // Overwrite output file
        .arg(output_str)
        .output()
        .map_err(|e| SpatialError::IoError(format!("Failed to run ffmpeg: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpatialError::ImageError(format!(
            "ffmpeg conversion failed for {} format:\n{}",
            format.to_uppercase(),
            stderr
        )));
    }

    tracing::debug!("Successfully converted image using ffmpeg");

    Ok(())
}

/// Get friendly format name
pub fn format_name(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "JPEG",
        "png" => "PNG",
        "gif" => "GIF",
        "bmp" => "BMP",
        "tiff" | "tif" => "TIFF",
        "webp" => "WebP",
        "avif" => "AVIF",
        "jxl" => "JPEG XL",
        "heic" | "heif" => "HEIC/HEIF",
        _ => "Unknown",
    }
}

/// List natively supported input formats
pub fn supported_formats() -> &'static str {
    "JPEG, PNG, GIF, BMP, TIFF, WebP"
}

/// List formats that support native decoding (if features enabled)
pub fn native_decoder_formats() -> &'static str {
    // Return a static string based on enabled features
    #[cfg(all(feature = "avif", feature = "jxl", feature = "heic"))]
    return "AVIF, JXL, HEIC";

    #[cfg(all(feature = "avif", feature = "jxl", not(feature = "heic")))]
    return "AVIF, JXL";

    #[cfg(all(feature = "avif", not(feature = "jxl"), feature = "heic"))]
    return "AVIF, HEIC";

    #[cfg(all(not(feature = "avif"), feature = "jxl", feature = "heic"))]
    return "JXL, HEIC";

    #[cfg(all(feature = "avif", not(feature = "jxl"), not(feature = "heic")))]
    return "AVIF";

    #[cfg(all(not(feature = "avif"), feature = "jxl", not(feature = "heic")))]
    return "JXL";

    #[cfg(all(not(feature = "avif"), not(feature = "jxl"), feature = "heic"))]
    return "HEIC";

    #[cfg(not(any(feature = "avif", feature = "jxl", feature = "heic")))]
    return "None (use --features to enable)";
}

/// List formats that require conversion (ffmpeg fallback)
pub fn conversion_fallback_formats() -> &'static str {
    "AVIF, JXL, HEIC/HEIF (via ffmpeg if native decoder not enabled/available)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        let path = Path::new("photo.jpg");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap();
        assert_eq!(ext, "jpg");
    }

    #[test]
    fn test_heic_extension() {
        let path = Path::new("photo.heic");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap();
        assert_eq!(ext, "heic");
    }

    #[test]
    fn test_avif_extension() {
        let path = Path::new("photo.avif");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap();
        assert_eq!(ext, "avif");
    }

    #[test]
    fn test_jxl_extension() {
        let path = Path::new("photo.jxl");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap();
        assert_eq!(ext, "jxl");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(format_name("jpg"), "JPEG");
        assert_eq!(format_name("heic"), "HEIC/HEIF");
        assert_eq!(format_name("avif"), "AVIF");
        assert_eq!(format_name("jxl"), "JPEG XL");
    }

    #[test]
    fn test_supported_formats() {
        let formats = supported_formats();
        assert!(formats.contains("JPEG"));
        assert!(formats.contains("PNG"));
    }

    #[test]
    fn test_ffmpeg_availability_check() {
        // This test just verifies the function runs without panic
        let _available = is_ffmpeg_available();
    }

    #[test]
    fn test_native_decoder_formats() {
        // This test verifies the function returns a valid string
        let formats = native_decoder_formats();
        assert!(!formats.is_empty());
    }
}
