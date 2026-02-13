# Quick Start Guide — spatial-maker v0.1.0

**TL;DR:** Generate spatial photos from any image format in seconds.

---

## Installation

```bash
# Add to Cargo.toml
[dependencies]
spatial-maker = "0.1.0"

# Or with native decoders (recommended)
spatial-maker = { version = "0.1.0", features = ["avif", "jxl"] }
```

---

## Basic Usage (Library)

```rust
use spatial_maker::{process_photo, SpatialConfig, OutputOptions};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    process_photo(
        Path::new("input.jpg"),      // Any format: jpg, png, heic, avif, jxl
        Path::new("output.jpg"),      // Output format
        SpatialConfig::default(),     // Use default settings
        OutputOptions::default(),     // Side-by-side stereo
    ).await?;
    Ok(())
}
```

---

## CLI Usage

```bash
# Clone and run
git clone <repo-url>
cd spatial-maker

# Process an image (any format)
cargo run --example photo -- \
  --input photo.heic \
  --output spatial.jpg

# With native decoders
cargo run --features avif,jxl --example photo -- \
  --input photo.jxl \
  --output spatial.jpg
```

---

## Supported Formats

### Input (All Work Out of the Box)
- ✅ JPEG / JPG
- ✅ PNG
- ✅ AVIF (native or ffmpeg)
- ✅ JPEG XL / JXL (native or ffmpeg)
- ✅ HEIC / HEIF (native or ffmpeg)
- ✅ WebP, GIF, BMP, TIFF

### Output
- ✅ JPEG
- ✅ PNG
- ✅ MV-HEVC (Apple spatial photo format)

---

## Feature Flags

| Flag | What It Does | System Deps |
|------|--------------|-------------|
| `jxl` | Native JXL decoder | None (pure Rust) |
| `avif` | Native AVIF decoder | May need libdav1d |
| `heic` | Native HEIC decoder | Requires libheif |
| `native-formats` | All of the above | All of the above |

**Recommended:** `--features avif,jxl` (best balance of features vs dependencies)

---

## System Requirements

### Mandatory
- Rust 1.70+ (2021 edition)
- ONNX Runtime (via `ort` crate)

### Optional (For Modern Formats)
- **ffmpeg** (fallback for AVIF/JXL/HEIC without native features)
  ```bash
  brew install ffmpeg              # macOS
  sudo apt-get install ffmpeg      # Ubuntu
  choco install ffmpeg             # Windows
  ```

- **libheif** (for `--features heic`)
  ```bash
  brew install libheif             # macOS
  sudo apt-get install libheif-dev # Ubuntu
  ```

- **libdav1d** (may be needed for `--features avif`)
  ```bash
  brew install dav1d               # macOS
  sudo apt-get install libdav1d-dev # Ubuntu
  ```

---

## Configuration Examples

### Default Settings
```rust
let config = SpatialConfig::default();
// encoder_size: "s" (small model, fast)
// max_disparity: 30 (moderate 3D effect)
// format: SideBySide
```

### Custom Settings
```rust
let config = SpatialConfig {
    encoder_size: "b".to_string(),  // base model (better quality, slower)
    max_disparity: 50,              // stronger 3D effect
    ..Default::default()
};

let output_options = OutputOptions {
    format: OutputFormat::TopAndBottom, // or Separate, SideBySide
    encoding: ImageEncoding::Png,       // or Jpeg, MvHevc
    ..Default::default()
};

process_photo(input, output, config, output_options).await?;
```

---

## Common Tasks

### Convert iPhone Photo to Spatial
```bash
cargo run --example photo -- \
  --input IMG_1234.heic \
  --output spatial_1234.jpg
```

### Batch Process
```bash
for img in *.heic; do
  cargo run --example photo -- \
    --input "$img" \
    --output "${img%.*}_spatial.jpg"
done
```

### Use Native Decoders
```rust
// Enable features in Cargo.toml
[dependencies]
spatial-maker = { version = "0.1.0", features = ["jxl", "avif"] }

// Code stays the same - native decoders used automatically!
process_photo(
    Path::new("photo.jxl"),
    Path::new("output.jpg"),
    config,
    output_options,
).await?;
```

---

## Build & Test

```bash
# Build with defaults
cargo build --release

# Build with native decoders
cargo build --release --features avif,jxl

# Run all tests
cargo test

# Run tests with features
cargo test --features avif,jxl

# Check build time
time cargo build --release --features avif,jxl
```

---

## Troubleshooting

### "HEIC format requires ffmpeg"
**Solution:** Install ffmpeg or enable native decoder
```bash
brew install ffmpeg
# OR
cargo build --features heic  # (requires libheif)
```

### "libheif not found" during build
**Solution:** Install system library or disable feature
```bash
brew install libheif
# OR
cargo build  # without --features heic
```

### Build takes too long
**Solution:** Use incremental builds or fewer features
```bash
cargo build --features jxl  # JXL only (faster than all)
```

### Tests fail
**Solution:** Verify dependencies and rebuild
```bash
cargo clean
cargo test
```

---

## Performance Tips

1. **Use native decoders** for formats you process frequently
   - Eliminates conversion overhead
   - Better image quality
   - Faster processing

2. **Choose appropriate model size**
   - `"s"` (small) — Fast, good quality
   - `"b"` (base) — Balanced
   - `"l"` (large) — Best quality, slowest

3. **Adjust max_disparity**
   - Lower (20-30) — Subtle 3D, faster
   - Higher (40-60) — Strong 3D, slower

4. **Enable CoreML** (macOS only)
   - Automatic via `ort` crate with `coreml` feature
   - Uses Apple Neural Engine
   - Significant speedup on Mac

---

## Next Steps

- 📖 Read [USAGE.md](./USAGE.md) for detailed API documentation
- 🎨 See [EXAMPLE_OUTPUTS.md](./EXAMPLE_OUTPUTS.md) for output samples
- 🔧 Check [AUTOMATIC_CONVERSION.md](./AUTOMATIC_CONVERSION.md) for ffmpeg details
- 📋 Review [IMAGE_FORMAT_SUPPORT.md](./IMAGE_FORMAT_SUPPORT.md) for format matrix
- 🚀 Read [RELEASE_NOTES_v0.1.0.md](./RELEASE_NOTES_v0.1.0.md) for what's new

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────┐
│ spatial-maker v0.1.0 — Quick Reference                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Install:                                                    │
│   cargo add spatial-maker --features avif,jxl              │
│                                                             │
│ Basic Code:                                                 │
│   process_photo(input, output, config, opts).await?;       │
│                                                             │
│ CLI:                                                        │
│   cargo run --example photo -- -i in.heic -o out.jpg       │
│                                                             │
│ Features:                                                   │
│   jxl          → Native JXL (no deps)                       │
│   avif         → Native AVIF (may need libdav1d)            │
│   heic         → Native HEIC (needs libheif)                │
│   native-formats → All native decoders                      │
│                                                             │
│ Formats:                                                    │
│   Input:  jpg, png, heic, avif, jxl, webp, gif, bmp, tiff  │
│   Output: jpg, png, heic (mv-hevc spatial)                 │
│                                                             │
│ System Tools (Optional):                                    │
│   ffmpeg       → Fallback conversion                        │
│   libheif      → For --features heic                        │
│   libdav1d     → May be needed for --features avif          │
│                                                             │
│ Tests: 46/46 passing ✅                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

**Version:** 0.1.0  
**License:** MIT  
**Status:** Production Ready ✅

**Get Started:**
```bash
cargo add spatial-maker --features avif,jxl
```

Happy spatial photo making! 📸✨