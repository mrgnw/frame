# spatial-maker

A high-performance Rust library for generating spatial photos and videos using ONNX-based depth estimation.

Replaces the Python `spatial-maker` package with a native Rust implementation, eliminating subprocess overhead and enabling CoreML acceleration on macOS.

## Features

- **ONNX-based Depth Estimation**: Uses pre-trained Depth Anything V2 models
- **Fast Stereo Generation**: Depth-Image-Based Rendering (DIBR) in pure Rust
- **Apple Neural Engine**: CoreML execution provider for M-series Macs
- **Modern Format Support**: Native AVIF, JXL, HEIC decoding (opt-in) + ffmpeg fallback
- **Photo & Video Support**: Single image and frame-by-frame video pipelines
- **Progress Callbacks**: Real-time feedback during processing

## Quick Start

```rust
use spatial_maker::{process_photo, SpatialConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SpatialConfig::default();
    process_photo(
        Path::new("input.jpg"),
        Path::new("spatial.jpg"),
        config,
    ).await?;
    Ok(())
}
```

### CLI Example

```bash
cargo run --example photo -- --input photo.jpg --output spatial.jpg --encoder s

# Works with modern formats too!
cargo run --example photo -- --input photo.heic --output spatial.jpg
```

## Supported Input Formats

**Native** (no conversion): JPEG, PNG, GIF, BMP, TIFF, WebP

**Opt-in Native Decoders** (enable via features):
- AVIF: `--features avif`
- JPEG XL: `--features jxl` (pure Rust, no system deps)
- HEIC: `--features heic` (requires system `libheif`)
- All: `--features native-formats`

**Auto-converted** (requires `ffmpeg`): AVIF, JXL, HEIC (when native features disabled)

See [docs/AUTOMATIC_CONVERSION.md](docs/AUTOMATIC_CONVERSION.md) for details.

## Installation

```toml
[dependencies]
spatial-maker = { path = "../spatial-maker" }

# With native format support
spatial-maker = { path = "../spatial-maker", features = ["jxl", "avif"] }
```

## Configuration

```rust
let config = SpatialConfig {
    encoder_size: "s".to_string(),  // "s" (small), "b" (base), "l" (large)
    max_disparity: 30,              // 3D effect strength
    target_depth_size: 518,         // Depth model resolution
    use_coreml: true,               // Apple Neural Engine (macOS)
};
```

### Model Sizes

| Model | Size | Speed | Quality |
|-------|------|-------|---------|
| Small (s) | 99MB | Fast | Good |
| Base (b) | 380MB | Medium | Excellent |
| Large (l) | 1.3GB | Slow | Best |

Models auto-download to `~/.spatial-maker/checkpoints/`

## Documentation

- [Usage Guide](docs/USAGE.md) - Detailed API and examples
- [Image Formats](docs/IMAGE_FORMATS.md) - Supported formats and conversion
- [Output Formats](docs/OUTPUT_FORMATS.md) - Stereo and MV-HEVC output
- [Automatic Conversion](docs/AUTOMATIC_CONVERSION.md) - Format conversion details
- [Release Notes](docs/releases/v0.1.0.md) - Version history

## Performance

On M1 MacBook Pro (small model):
- **Depth estimation**: ~300ms (CoreML) / ~800ms (CPU)
- **Stereo generation**: ~50ms
- **Total**: ~400ms end-to-end

## Testing

```bash
cargo test                          # Default features
cargo test --features jxl,avif      # With native decoders
```

## Dependencies

- `ort` - ONNX Runtime inference
- `image` - Image loading/saving
- `ndarray` - Tensor operations
- `tokio` - Async runtime
- `jxl-oxide` - JPEG XL decoder (optional)
- `libheif-rs` - HEIC decoder (optional)

## License

MIT

## References

- [Depth Anything V2](https://github.com/aimagelab/depth-anything-v2)
- [ONNX Community Models](https://huggingface.co/onnx-community)