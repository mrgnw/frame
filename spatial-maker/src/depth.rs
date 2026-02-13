//! Depth estimation using ONNX models
//!
//! This module handles:
//! - Loading ONNX models from disk via the `ort` crate
//! - Preprocessing images (resize, normalize, convert to NCHW tensor)
//! - Running inference
//! - Extracting and validating depth map output

use crate::error::{SpatialError, SpatialResult};
use crate::model;
use image::DynamicImage;
use ort::session::Session;

/// Configuration for depth estimation
#[derive(Clone, Debug)]
pub struct DepthConfig {
    /// Model size: "s" (small), "b" (base), or "l" (large)
    pub encoder_size: String,

    /// Target input size for the shorter side (in pixels)
    /// The model will resize to this size, maintaining aspect ratio
    pub target_size: u32,

    /// Whether to use CoreML execution provider (macOS only)
    pub use_coreml: bool,
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            encoder_size: "s".to_string(),
            target_size: 518,
            use_coreml: true,
        }
    }
}

/// ImageNet normalization constants
/// These are the standard mean and std values used for preprocessing
const IMAGENET_MEAN: &[f32] = &[0.485, 0.456, 0.406];
const IMAGENET_STD: &[f32] = &[0.229, 0.224, 0.225];

/// Load and cache an ONNX model session
///
/// This loads the model from the checkpoint directory using the `ort` crate.
/// The model must be present (use `model::ensure_model_exists` to download).
async fn load_model_session(encoder_size: &str) -> SpatialResult<Session> {
    let model_path = model::find_model(encoder_size)?;

    tracing::info!("Loading ONNX model from: {:?}", model_path);

    let session = Session::builder()
        .map_err(|e| SpatialError::OrtError(format!("Failed to create session builder: {:?}", e)))?
        .commit_from_file(&model_path)
        .map_err(|e| SpatialError::ModelError(format!("Failed to load model: {:?}", e)))?;

    tracing::info!("Model loaded successfully");
    Ok(session)
}

/// Preprocess an image for depth estimation
///
/// This converts a DynamicImage to an NCHW float32 tensor (batch=1):
/// 1. Resize to target size (shorter side), maintaining aspect ratio
/// 2. Convert to RGB
/// 3. Normalize using ImageNet mean/std
/// 4. Convert to NCHW format for ONNX model
///
/// # Returns
///
/// A tuple of (input_tensor: Vec<f32>, actual_height: u32, actual_width: u32)
fn preprocess_image(image: &DynamicImage, target_size: u32) -> (Vec<f32>, u32, u32) {
    // Compute resize dimensions maintaining aspect ratio
    let (orig_width, orig_height) = (image.width(), image.height());
    let (new_width, new_height) = if orig_width < orig_height {
        let h = (orig_height as f32 * target_size as f32 / orig_width as f32) as u32;
        (target_size, h)
    } else {
        let w = (orig_width as f32 * target_size as f32 / orig_height as f32) as u32;
        (w, target_size)
    };

    tracing::debug!(
        "Preprocessing: Original {}x{}, resizing to {}x{} (target_size={})",
        orig_width,
        orig_height,
        new_width,
        new_height,
        target_size
    );

    // Resize image
    let resized = image.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert to RGB
    let rgb_image = resized.to_rgb8();

    // Convert to f32 and normalize
    let mut tensor_data = Vec::with_capacity((3 * new_height * new_width) as usize);

    for pixel in rgb_image.pixels() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        // Normalize using ImageNet mean/std
        let r_norm = (r - IMAGENET_MEAN[0]) / IMAGENET_STD[0];
        let g_norm = (g - IMAGENET_MEAN[1]) / IMAGENET_STD[1];
        let b_norm = (b - IMAGENET_MEAN[2]) / IMAGENET_STD[2];

        tensor_data.push(r_norm);
        tensor_data.push(g_norm);
        tensor_data.push(b_norm);
    }

    // Reshape to NCHW: (1, 3, height, width)
    let mut nchw = vec![0.0_f32; (3 * new_height * new_width) as usize];

    for y in 0..new_height {
        for x in 0..new_width {
            let pixel_idx = (y * new_width + x) as usize;
            let rgb_idx = pixel_idx * 3;

            // R channel
            nchw[0 * (new_height * new_width) as usize + pixel_idx] = tensor_data[rgb_idx];
            // G channel
            nchw[1 * (new_height * new_width) as usize + pixel_idx] = tensor_data[rgb_idx + 1];
            // B channel
            nchw[2 * (new_height * new_width) as usize + pixel_idx] = tensor_data[rgb_idx + 2];
        }
    }

    (nchw, new_height, new_width)
}

/// Run inference on a preprocessed tensor
///
/// Returns a tuple of (depth_data, actual_height, actual_width)
/// The actual dimensions are extracted from the model output shape
fn run_inference(
    session: &mut Session,
    input_tensor: Vec<f32>,
    height: u32,
    width: u32,
) -> SpatialResult<(Vec<f32>, u32, u32)> {
    tracing::debug!("Running inference on {}x{} image", width, height);

    // Create input shape: (batch=1, channels=3, height, width)
    let shape = vec![1, 3, height as i64, width as i64];

    // Convert to ort Value
    let input_value = ort::value::Value::from_array((shape, input_tensor)).map_err(|e| {
        SpatialError::TensorError(format!("Failed to create input tensor: {:?}", e))
    })?;

    // Run inference
    // The model expects input name "pixel_values" (discovered in Chunk 0 spike)
    let inputs = vec![("pixel_values", &input_value)];

    let outputs = session
        .run(inputs)
        .map_err(|e| SpatialError::OrtError(format!("Inference failed: {:?}", e)))?;

    tracing::debug!("Inference completed, extracting outputs");

    // Extract the depth output
    // The model outputs "predicted_depth" with shape (batch, height, width)
    if let Some((_name, output)) = outputs.iter().next() {
        let depth_tensor = output.try_extract_tensor::<f32>().map_err(|e| {
            SpatialError::TensorError(format!("Failed to extract output tensor: {:?}", e))
        })?;

        let (shape, data) = depth_tensor;
        tracing::debug!(
            "Output tensor shape: {:?}, total elements: {}",
            shape,
            data.len()
        );

        // Extract actual dimensions from model output shape [batch, height, width]
        let (actual_height, actual_width) = if shape.len() == 3 {
            (shape[1] as u32, shape[2] as u32)
        } else if shape.len() == 2 {
            (shape[0] as u32, shape[1] as u32)
        } else {
            return Err(SpatialError::TensorError(format!(
                "Unexpected output shape: {:?}",
                shape
            )));
        };

        tracing::debug!(
            "Actual output dimensions: {}x{}",
            actual_height,
            actual_width
        );

        Ok((data.to_vec(), actual_height, actual_width))
    } else {
        Err(SpatialError::TensorError(
            "No outputs from model".to_string(),
        ))
    }
}

/// Normalize depth map to 0-1 range
///
/// Maps the output from the model (typically log-scale depth, 0.0-10.0)
/// to a normalized 0-1 range for easier processing downstream
fn normalize_depth(depth_raw: &[f32]) -> Vec<f32> {
    if depth_raw.is_empty() {
        return vec![];
    }

    let min_depth = depth_raw.iter().copied().fold(f32::INFINITY, f32::min);
    let max_depth = depth_raw.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let range = max_depth - min_depth;

    if range < 1e-6 {
        // All values are the same, return uniform array
        vec![0.5; depth_raw.len()]
    } else {
        depth_raw.iter().map(|&d| (d - min_depth) / range).collect()
    }
}

/// Estimate depth from an image
///
/// This is the main entry point for depth estimation:
/// 1. Ensure the model is downloaded
/// 2. Load the ONNX model
/// 3. Preprocess the image
/// 4. Run inference
/// 5. Normalize and return the depth map
///
/// # Arguments
///
/// * `image` - Input image (any format supported by `image` crate)
/// * `config` - Depth estimation configuration
///
/// # Returns
///
/// A 2D array representing the normalized depth map (0-1 range)
/// with shape (height, width)
pub async fn estimate_depth(
    image: &DynamicImage,
    config: &DepthConfig,
) -> SpatialResult<ndarray::Array2<f32>> {
    tracing::info!("Estimating depth with encoder: {}", config.encoder_size);

    // Ensure model is available (download if needed)
    let _model_path =
        model::ensure_model_exists::<fn(u64, u64)>(&config.encoder_size, None).await?;

    // Load model
    let mut session = load_model_session(&config.encoder_size).await?;

    // Preprocess image
    tracing::debug!("Preprocessing image");
    let (input_tensor, prep_height, prep_width) = preprocess_image(image, config.target_size);

    // Run inference
    let (depth_raw, actual_height, actual_width) =
        run_inference(&mut session, input_tensor, prep_height, prep_width)?;

    tracing::debug!(
        "Model output actual dimensions: {}x{} (expected {}x{})",
        actual_height,
        actual_width,
        prep_height,
        prep_width
    );

    // Normalize depth
    let depth_normalized = normalize_depth(&depth_raw);

    // Convert to ndarray (height, width)
    // Use actual output dimensions from the model, not our preprocessing dimensions
    let h = actual_height as usize;
    let w = actual_width as usize;
    let expected_elements = h * w;
    let actual_elements = depth_normalized.len();

    tracing::debug!(
        "Reshaping depth: using actual model output dimensions {}x{} = {} elements, got {}",
        h,
        w,
        expected_elements,
        actual_elements
    );

    let depth_2d = ndarray::Array1::from_vec(depth_normalized)
        .into_shape((h, w))
        .map_err(|e| {
            SpatialError::TensorError(format!(
                "Failed to reshape depth to {}x{} ({} elements): {}",
                h, w, expected_elements, e
            ))
        })?;

    tracing::info!(
        "Depth estimation complete: {}x{}",
        actual_height,
        actual_width
    );

    Ok(depth_2d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imagenet_normalization_constants() {
        // Verify constants are correct
        assert_eq!(IMAGENET_MEAN.len(), 3);
        assert_eq!(IMAGENET_STD.len(), 3);
        assert!(IMAGENET_MEAN[0] > 0.4 && IMAGENET_MEAN[0] < 0.5);
    }

    #[test]
    fn test_normalize_depth_range() {
        let raw = vec![0.0, 5.0, 10.0];
        let normalized = normalize_depth(&raw);

        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.0).abs() < 1e-6);
        assert!((normalized[1] - 0.5).abs() < 1e-6);
        assert!((normalized[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_depth_uniform() {
        let raw = vec![5.0, 5.0, 5.0];
        let normalized = normalize_depth(&raw);

        assert_eq!(normalized.len(), 3);
        assert!((normalized[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_depth_empty() {
        let raw: Vec<f32> = vec![];
        let normalized = normalize_depth(&raw);
        assert_eq!(normalized.len(), 0);
    }

    #[test]
    fn test_depth_config_defaults() {
        let config = DepthConfig::default();
        assert_eq!(config.encoder_size, "s");
        assert_eq!(config.target_size, 518);
        assert!(config.use_coreml);
    }
}
