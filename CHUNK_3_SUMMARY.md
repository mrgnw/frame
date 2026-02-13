# Chunk 3: Photo Output & Example — Quick Summary

**Status**: ✅ **COMPLETE** | **Tests**: 34/34 passing | **Build**: ✅ Clean

---

## What Was Built

### 1. Output Module (`src/output.rs`)
A production-ready stereo image output system supporting:

| Feature | Support |
|---------|---------|
| **Layouts** | Side-by-side, top-and-bottom, separate files |
| **Formats** | JPEG (quality 1-100), PNG (lossless) |
| **MV-HEVC** | Optional encoding via `spatial` CLI |
| **Error Handling** | Dimension validation, quality clamping |
| **Tests** | 16 dedicated unit tests |

### 2. Enhanced CLI Example (`examples/photo.rs`)
Full-featured CLI demonstrating all output options:
```bash
# Side-by-side JPEG (default)
cargo run --example photo -- --input in.jpg --output out.jpg

# Top-and-bottom PNG
cargo run --example photo -- --input in.jpg --output out.png \
  --format top-bottom --image-encoding png

# Separate left/right files
cargo run --example photo -- --input in.jpg --output stereo.jpg \
  --format separate

# With MV-HEVC encoding
cargo run --example photo -- --input in.jpg --output out.heic \
  --mvhevc --mvhevc-quality 95
```

### 3. Updated Core API (`src/lib.rs`)
- `process_photo()` now accepts `OutputOptions` for flexible output control
- Calls `save_stereo_image()` internally for format/encoding handling
- Maintains async signature for depth estimation

---

## Key Types

```rust
pub enum OutputFormat {
    SideBySide,      // left | right
    TopAndBottom,    // left above right
    Separate,        // _L and _R files
}

pub enum ImageEncoding {
    Jpeg { quality: u8 },  // 1-100
    Png,
}

pub struct OutputOptions {
    pub layout: OutputFormat,
    pub image_format: ImageEncoding,
    pub mvhevc: Option<MVHEVCConfig>,
}

pub struct MVHEVCConfig {
    pub spatial_cli_path: Option<PathBuf>,
    pub enabled: bool,
    pub quality: u8,
    pub keep_intermediate: bool,
}
```

---

## Public API

```rust
pub fn save_stereo_image(
    left: &DynamicImage,
    right: &DynamicImage,
    output_path: impl AsRef<Path>,
    options: OutputOptions,
) -> SpatialResult<()>
```

**Usage Example:**
```rust
let options = OutputOptions {
    layout: OutputFormat::SideBySide,
    image_format: ImageEncoding::Jpeg { quality: 95 },
    mvhevc: None,
};

save_stereo_image(&left, &right, "output.jpg", options)?;
```

---

## Validation Rules

- **Side-by-side**: Left and right must have same **height**
- **Top-and-bottom**: Left and right must have same **width**
- **Separate**: No restrictions (each image saved independently)
- **JPEG Quality**: Automatically clamped to 1-100

---

## Error Handling

Clear error messages for common issues:
- "Height mismatch: left=X, right=Y" → Use separate layout or resize
- "Width mismatch: left=X, right=Y" → Use separate layout or resize
- "Failed to run `spatial` CLI: ..." → `spatial` tool not installed
- "Failed to create output directory: ..." → Permission issue

---

## Test Coverage

**Total**: 34 tests, all passing ✅

**Output Module Tests (16)**:
- Format creation: SBS JPEG/PNG, TB JPEG/PNG, separate
- Dimension validation: Height/width mismatch detection
- Full integration tests via `save_stereo_image()`
- Configuration defaults

**Other Tests (18)**:
- Depth estimation (6)
- Stereo generation (4)
- Model management (4)
- Error handling (2)
- Core config (2)

**Run tests**:
```bash
cargo test --lib
```

---

## Build Status

```
✅ cargo check
✅ cargo build --example photo
✅ cargo test --lib (34/34 passing)
✅ No warnings
```

---

## Integration Notes for Frame

When integrating into Frame:

1. **Add dependency** to `src-tauri/Cargo.toml`:
   ```toml
   spatial-maker = { path = "../spatial-maker" }
   ```

2. **Update worker calls** from:
   ```rust
   // Old: subprocess
   std::process::Command::new("python")
       .arg("spatial_pipeline.py")
       .output()?
   ```
   
   To:
   ```rust
   // New: direct crate
   let options = OutputOptions::default();
   process_photo(input, output, config, options).await?;
   ```

3. **Replace progress callbacks** from stdout JSON to Rust channels/tokio broadcast

4. **Replace SIGKILL** with cancellation tokens for graceful shutdown

---

## MV-HEVC Notes

- Requires `spatial` CLI tool to be installed separately
- Creates `.heic` files (motion video format)
- Intermediate SBS image can be kept or automatically removed
- Quality parameter (0-100) passed to `spatial encode`

**Example with MV-HEVC**:
```rust
let options = OutputOptions {
    layout: OutputFormat::SideBySide,
    image_format: ImageEncoding::Jpeg { quality: 95 },
    mvhevc: Some(MVHEVCConfig {
        spatial_cli_path: Some(PathBuf::from("/usr/local/bin/spatial")),
        enabled: true,
        quality: 95,
        keep_intermediate: false,
    }),
};

save_stereo_image(&left, &right, "output.jpg", options)?;
// Creates output.heic, removes output.jpg
```

---

## Files Changed

| File | Changes |
|------|---------|
| `src/output.rs` | ✅ NEW (670 lines) |
| `src/lib.rs` | Updated: `process_photo()` signature, exports |
| `examples/photo.rs` | Rewritten with all format/encoding options |
| Tests | +16 new output module tests |

---

## What's Next?

**Chunk 4 (Recommended)**: Video Pipeline
- Frame extraction, batching
- Progress callbacks + cancellation tokens
- FFmpeg integration for H.264/H.265

**Optional**: Chunk 2 revisited for stereo refinement

---

## Backward Compatibility

✅ `OutputOptions::default()` uses sensible defaults
✅ Existing code can be migrated incrementally
✅ API is extensible for future formats

---

## Status

🎯 **Chunk 3 is production-ready and ready for Frame integration.**

Next milestone: Frame integration + Chunk 4 (Video Pipeline)