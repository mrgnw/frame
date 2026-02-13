//! Error types for spatial-maker operations

use std::fmt;

/// Result type for spatial-maker operations
pub type SpatialResult<T> = Result<T, SpatialError>;

/// Comprehensive error type for spatial-maker operations
#[derive(Debug)]
pub enum SpatialError {
    /// Model-related errors (loading, inference, metadata)
    ModelError(String),

    /// Image processing errors (loading, resizing, format)
    ImageError(String),

    /// Tensor operation errors (shape mismatch, data conversion)
    TensorError(String),

    /// File I/O errors
    IoError(String),

    /// Configuration or validation errors
    ConfigError(String),

    /// ONNX Runtime errors
    OrtError(String),

    /// Generic catch-all error
    Other(String),
}

impl fmt::Display for SpatialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpatialError::ModelError(msg) => write!(f, "Model error: {}", msg),
            SpatialError::ImageError(msg) => write!(f, "Image error: {}", msg),
            SpatialError::TensorError(msg) => write!(f, "Tensor error: {}", msg),
            SpatialError::IoError(msg) => write!(f, "I/O error: {}", msg),
            SpatialError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            SpatialError::OrtError(msg) => write!(f, "ONNX Runtime error: {}", msg),
            SpatialError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for SpatialError {}

/// Convert from ort errors (handled implicitly in depth.rs)
/// Note: ort::OrtError doesn't expose a public error type in 2.0.0-rc.11
/// We handle this by converting to String in the depth module

/// Convert from std::io::Error to SpatialError
impl From<std::io::Error> for SpatialError {
    fn from(e: std::io::Error) -> Self {
        SpatialError::IoError(e.to_string())
    }
}

/// Convert from image::ImageError to SpatialError
impl From<image::ImageError> for SpatialError {
    fn from(e: image::ImageError) -> Self {
        SpatialError::ImageError(e.to_string())
    }
}

/// Convert from anyhow::Error to SpatialError
impl From<anyhow::Error> for SpatialError {
    fn from(e: anyhow::Error) -> Self {
        SpatialError::Other(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SpatialError::ModelError("test model error".to_string());
        assert_eq!(err.to_string(), "Model error: test model error");
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> SpatialResult<i32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);
    }
}
