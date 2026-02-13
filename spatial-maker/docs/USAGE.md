# spatial-maker Quick Start Guide

A no-frills guide to using the `spatial-maker` Rust crate.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spatial-maker = { path = "../spatial-maker" }
```

Or from a workspace:

```toml
spatial-maker = { path = "spatial-maker" }
```

## Minimal Example

```rust
use spatial_maker::{process_photo, SpatialConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SpatialConfig::default();
    process_photo(
        Path::new("input.jpg"),
        Path::new("output.jpg"),
        config,
    ).await?;
    Ok(())
}
```

## Configure Logging

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("spatial_maker=info")
        .init();

    // ... your code here
    Ok(())
}
```

Or set environment variable:
```bash
RUST_LOG=spatial_maker=debug cargo run
```

## Configuration Options

```rust
use spatial_maker::{SpatialConfig, process_photo};
use std::path::Path;

let config = SpatialConfig {
    encoder_size: "b".to_string(),      // "s", "b", or "l"
    max_disparity: 40,                  // Larger = more 3D, more artifacts
    target_depth_size: 518,             // Input resolution for model
    use_coreml: true,                   // Use Apple Neural Engine on macOS
};

process_photo(
    Path::new("input.jpg"),
    Path::new("output.jpg"),
    config,
).await?;
```

## Model Size Tradeoffs

| Size | Speed | Quality | When to Use |
|------|-------|---------|-------------|
| s (small) | ⚡⚡⚡ Fast | Good | Real-time, memory-constrained |
| b (base) | ⚡⚡ Medium | Excellent | Recommended default |
| l (large) | ⚡ Slow | Best | Offline, highest quality |

First run downloads model (~100MB-1.3GB depending on size).

## Environment Variables

Override checkpoint directory (default: `~/.spatial-maker/checkpoints/`):

```bash
export SPATIAL_MAKER_CHECKPOINTS=/custom/path
cargo run --example photo -- input.jpg --output output.jpg
```

## Error Handling

```rust
use spatial_maker::{process_photo, SpatialError, SpatialConfig};
use std::path::Path;

match process_photo(
    Path::new("input.jpg"),
    Path::new("output.jpg"),
    SpatialConfig::default(),
).await {
    Ok(_) => println!("✅ Success!"),
    Err(SpatialError::ModelError(msg)) => eprintln!("❌ Model: {}", msg),
    Err(SpatialError::ImageError(msg)) => eprintln!("❌ Image: {}", msg),
    Err(SpatialError::ConfigError(msg)) => eprintln!("❌ Config: {}", msg),
    Err(e) => eprintln!("❌ Error: {}", e),
}
```

## Using Individual Modules

### Just Get Depth Map

```rust
use spatial_maker::{estimate_depth, DepthConfig};
use image::open;

let image = open("input.jpg")?;
let depth = estimate_depth(&image, &DepthConfig::default()).await?;
println!("Depth map shape: {:?}", depth.dim());
```

### Just Generate Stereo

```rust
use spatial_maker::generate_stereo_pair;
use image::open;
use ndarray::Array2;

let image = open("input.jpg")?;
let depth = Array2::from_elem((image.height() as usize, image.width() as usize), 0.5);
let (left, right) = generate_stereo_pair(&image, &depth, 30)?;
left.save("left.jpg")?;
right.save("right.jpg")?;
```

### Manual Model Management

```rust
use spatial_maker::model;

// Check if model exists
if model::model_exists("s") {
    println!("✅ Small model already downloaded");
} else {
    println!("⏳ Downloading small model...");
    let path = model::ensure_model_exists("s", None).await?;
    println!("✅ Downloaded to: {:?}", path);
}
```

With progress callback:

```rust
use spatial_maker::model;

let path = model::ensure_model_exists("b", Some(|current, total| {
    let pct = (current as f64 / total as f64 * 100.0).round();
    println!("Downloaded {}%", pct);
})).await?;
```

## CLI Tool

```bash
# Build
cargo build --example photo --release

# Basic usage
./target/release/examples/photo input.jpg --output spatial.jpg

# All options
./target/release/examples/photo \
    input.jpg \
    --output spatial.jpg \
    --encoder b \
    --max-disparity 40 \
    --target-size 518 \
    --verbose

# Disable CoreML (force CPU on macOS)
./target/release/examples/photo input.jpg --output spatial.jpg --no-coreml
```

## Performance Tips

### For Real-Time Inference
- Use `encoder_size: "s"` (small model)
- Reduce `target_depth_size` to 384 or 256
- Enable CoreML on macOS: `use_coreml: true`

### For Best Quality
- Use `encoder_size: "l"` (large model)
- Keep `target_depth_size: 518` or higher
- Increase `max_disparity` to 40-60

### For Memory-Constrained Systems
- Use `encoder_size: "s"`
- Reduce `target_depth_size` to 256
- Process images in batches with separate process calls

## Testing

Run all tests:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

Run specific test:
```bash
cargo test test_normalize_depth_range
```

## Integration with Frame

In Frame's `src-tauri/Cargo.toml`:

```toml
[dependencies]
spatial-maker = { path = "../spatial-maker" }
```

In a Tauri command:

```rust
use spatial_maker::{process_photo, SpatialConfig};
use std::path::Path;

#[tauri::command]
async fn spatial_process(input: String, output: String) -> Result<(), String> {
    let config = SpatialConfig::default();
    process_photo(
        Path::new(&input),
        Path::new(&output),
        config,
    )
    .await
    .map_err(|e| e.to_string())
}
```

## Troubleshooting

### "Model not found"
First run will download. Check `~/.spatial-maker/checkpoints/`:
```bash
ls -lh ~/.spatial-maker/checkpoints/
```

Or set custom path:
```bash
export SPATIAL_MAKER_CHECKPOINTS=/path/to/models
```

### "Out of memory"
Use smaller model or resolution:
```rust
let config = SpatialConfig {
    encoder_size: "s".to_string(),
    target_depth_size: 256,  // Smaller
    ..Default::default()
};
```

### "Image format not supported"
The `image` crate supports: JPEG, PNG, GIF, BMP, TIFF, ICO, PNM, WebP, Farbfeld.

### "CoreML errors on macOS"
Falls back to CPU automatically. To force CPU:
```rust
let config = SpatialConfig {
    use_coreml: false,
    ..Default::default()
};
```

### "Download fails"
Check internet connection and disk space (~1.3GB for large model):
```bash
# Test download
curl -I https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx

# Check disk space
df -h ~/.spatial-maker/checkpoints/
```

## API Reference

See `README.md` for complete API docs or:
```bash
cargo doc --open
```

## Examples

See `examples/photo.rs` for a complete CLI tool.

## Contributing

Found a bug or want to improve performance? Check `README.md` for contribution guidelines.

## License

MIT