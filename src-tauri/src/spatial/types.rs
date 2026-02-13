use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpatialConfig {
    pub encoder_size: String,
    pub max_disparity: u32,
    pub skip_downscale: bool,
    pub duration: Option<f64>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgressPayload {
    pub encoder_size: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub progress: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadCompletePayload {
    pub encoder_size: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadErrorPayload {
    pub encoder_size: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct SpatialTask {
    pub id: String,
    pub file_path: String,
    pub config: SpatialConfig,
}

#[derive(Clone, Serialize)]
pub struct SpatialStartedPayload {
    pub id: String,
}

#[derive(Clone, Serialize)]
pub struct SpatialProgressPayload {
    pub id: String,
    pub progress: f64,
    pub stage: String,
}

#[derive(Clone, Serialize)]
pub struct SpatialCompletedPayload {
    pub id: String,
    pub output_path: String,
}

#[derive(Clone, Serialize)]
pub struct SpatialErrorPayload {
    pub id: String,
    pub error: String,
}

#[derive(Clone, Serialize)]
pub struct SpatialLogPayload {
    pub id: String,
    pub line: String,
}
