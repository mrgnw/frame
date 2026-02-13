# Proposal: Native Format Decoding with Optional Features

## Executive Summary

Replace ffmpeg-based conversion with **optional native Rust decoders** for AVIF, JXL, and HEIC formats. This provides better performance, eliminates external dependencies, and gives users flexible deployment options.

## Current State

**Implementation:** ffmpeg-based conversion (automatic, transparent)
- ✅ Works for all three formats (AVIF, JXL, HEIC)
- ✅ Single external dependency (ffmpeg)
- ✅ Simple to use and understand
- ❌ Requires external tool installation
- ❌ Process spawning overhead (~0.1-0.5s per conversion)
- ❌ Temporary file creation and cleanup

## Proposed Solution: Feature Flags for Native Decoding

### Architecture

```toml
[features]
default = ["jpeg", "png", "gif", "bmp", "tiff", "webp"]

# Native format decoders (optional)
avif = ["image/avif"]           # AVIF via dav1d (system lib)
jxl = ["jxl-oxide"]              # JPEG XL (pure Rust!)
heic = ["libheif-rs"]            # HEIC via libheif (system lib)

# Convenience bundles
native-formats = ["avif", "jxl", "heic"]
all-formats = ["native-formats"]

# Core image formats (included in default)
jpeg = ["image/jpeg"]
png = ["image/png"]
gif = ["image/gif"]
bmp = ["image/bmp"]
tiff = ["image/tiff"]
webp = ["image/webp"]
```

### Decoding Priority

For each format, the loader will try in order:

1. **Native decoder** (if feature enabled)
   - Direct memory-to-memory decoding
   - No temporary files
   - Fastest option

2. **ffmpeg fallback** (if native fails or not enabled)
   - Current implementation
   - Works if ffmpeg is installed
   - Automatic conversion to JPEG

3. **Error with guidance** (if both unavailable)
   - Clear error message
   - Installation instructions for both options

### Implementation Strategy

```rust
// src/image_loader.rs

pub async fn load_image(path: impl AsRef<Path>) -> SpatialResult<DynamicImage> {
    let path = path.as_ref();
    let extension = get_extension(path)?;
    
    match extension.as_str() {
        // Native formats (always available)
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => {
            load_standard(path)
        }
        
        // AVIF: Try native first, then ffmpeg
        "avif" => {
            #[cfg(feature = "avif")]
            {
                load_avif_native(path).or_else(|_| load_with_ffmpeg(path, "avif").await)
            }
            #[cfg(not(feature = "avif"))]
            {
                load_with_ffmpeg(path, "avif").await
            }
        }
        
        // JXL: Try native first, then ffmpeg
        "jxl" => {
            #[cfg(feature = "jxl")]
            {
                load_jxl_native(path).or_else(|_| load_with_ffmpeg(path, "jxl").await)
            }
            #[cfg(not(feature = "jxl"))]
            {
                load_with_ffmpeg(path, "jxl").await
            }
        }
        
        // HEIC: Try native first, then ffmpeg
        "heic" | "heif" => {
            #[cfg(feature = "heic")]
            {
                load_heic_native(path).or_else(|_| load_with_ffmpeg(path, "heic").await)
            }
            #[cfg(not(feature = "heic"))]
            {
                load_with_ffmpeg(path, "heic").await
            }
        }
        
        _ => Err(SpatialError::ImageError(format!("Unsupported format: {}", extension)))
    }
}

#[cfg(feature = "avif")]
fn load_avif_native(path: &Path) -> SpatialResult<DynamicImage> {
    // Use image crate with avif feature
    image::open(path).map_err(Into::into)
}

#[cfg(feature = "jxl")]
fn load_jxl_native(path: &Path) -> SpatialResult<DynamicImage> {
    use jxl_oxide::JxlImage;
    
    let data = std::fs::read(path)?;
    let image = JxlImage::from_reader(&data[..])?;
    
    // Convert to DynamicImage
    // ... conversion logic ...
    
    Ok(dynamic_image)
}

#[cfg(feature = "heic")]
fn load_heic_native(path: &Path) -> SpatialResult<DynamicImage> {
    use libheif_rs::{LibHeif, HeifContext, ColorSpace, RgbChroma};
    
    let lib_heif = LibHeif::new();
    let ctx = HeifContext::read_from_file(path)?;
    let handle = ctx.primary_image_handle()?;
    let image = lib_heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)?;
    
    // Convert to DynamicImage
    // ... conversion logic ...
    
    Ok(dynamic_image)
}
```

## Detailed Format Analysis

### 1. AVIF (AV1 Image Format)

**Option A: image crate with avif feature**
```toml
image = { version = "0.24", features = ["avif"] }
# Uses dav1d decoder (requires system libdav1d)
```

**Pros:**
- ✅ Integrates directly with `image` crate (no conversion needed)
- ✅ Maintained by image-rs team
- ✅ Same API as other formats

**Cons:**
- ❌ Requires system `libdav1d` library
- ❌ Build complexity (pkg-config on Linux, vcpkg on Windows)

**Installation Requirements:**
```bash
# macOS
brew install dav1d

# Ubuntu/Debian
sudo apt-get install libdav1d-dev

# Windows
vcpkg install dav1d
```

**Performance:** Fast native decoding, no conversion overhead

---

### 2. JPEG XL (JXL)

**Option A: jxl-oxide (Pure Rust!)**
```toml
jxl-oxide = "0.12"
```

**Pros:**
- ✅ **Pure Rust** - No system dependencies!
- ✅ No external libraries required
- ✅ Cross-platform without hassle
- ✅ Safe and fast

**Cons:**
- ❌ Newer crate (but actively maintained)
- ❌ Manual conversion to `DynamicImage` needed

**Installation Requirements:**
- None! Pure Rust, builds everywhere

**Performance:** Excellent, pure Rust implementation

---

### 3. HEIC/HEIF

**Option A: libheif-rs**
```toml
libheif-rs = { version = "2.6", features = ["image"] }
```

**Pros:**
- ✅ Mature and stable
- ✅ Handles all HEIC variants (including Apple spatial photos)
- ✅ Supports metadata extraction (Exif, XMP)
- ✅ Optional `image` crate integration

**Cons:**
- ❌ Requires system `libheif` library (>= 1.17.0)
- ❌ Build complexity similar to AVIF

**Installation Requirements:**
```bash
# macOS
brew install libheif

# Ubuntu/Debian
sudo apt-get install libheif-dev

# Windows
vcpkg install libheif
```

**Performance:** Fast native decoding with hardware acceleration on some platforms

---

## Comparison Matrix

| Format | Native Decoder | System Deps | Build Complexity | Performance | Recommended |
|--------|---------------|-------------|------------------|-------------|-------------|
| AVIF | image/avif + dav1d | libdav1d | Medium | Excellent | ✅ Yes |
| JXL | jxl-oxide | **None** | **Low** | Excellent | ✅✅ **Highly Recommended** |
| HEIC | libheif-rs | libheif | Medium | Excellent | ✅ Yes |

---

## User Experience

### Scenario 1: Simple User (Default)
```bash
# Just install ffmpeg (current behavior)
brew install ffmpeg

cargo run --example photo -- --input photo.heic --output spatial.jpg
# ✅ Works via ffmpeg conversion (automatic)
```

### Scenario 2: Performance User (Native Decoders)
```bash
# Install system libraries
brew install libdav1d libheif

# Build with native features
cargo build --features native-formats

cargo run --example photo -- --input photo.heic --output spatial.jpg
# ✅ Works via native decoding (faster, no temp files)
```

### Scenario 3: Pure Rust User (JXL Only)
```bash
# No system dependencies needed!
cargo build --features jxl

cargo run --example photo -- --input photo.jxl --output spatial.jpg
# ✅ Works via pure Rust decoder (best of both worlds)
```

### Scenario 4: Server Deployment
```dockerfile
FROM rust:latest

# Option A: ffmpeg only (simple)
RUN apt-get update && apt-get install -y ffmpeg

# Option B: Native decoders (faster)
RUN apt-get update && apt-get install -y \
    libdav1d-dev \
    libheif-dev

# Build with appropriate features
RUN cargo build --release --features native-formats
```

---

## Migration Path

### Phase 1: Add Optional Features (Non-Breaking)
1. Add feature flags to `Cargo.toml`
2. Implement native decoders with feature gates
3. Keep ffmpeg as fallback (current default behavior)
4. Update documentation

**Impact:** Zero breaking changes, purely additive

### Phase 2: Optimize Defaults (Optional, Future)
1. Consider making `jxl` a default feature (pure Rust, no deps)
2. Keep AVIF/HEIC as opt-in (require system libs)
3. Deprecate ffmpeg requirement in documentation

**Impact:** Better out-of-box experience for JXL

---

## Recommended Implementation Plan

### Immediate (Phase 1)

**Priority: JXL Native Decoder**
- ✅ Pure Rust (no system dependencies)
- ✅ Easy to add
- ✅ No breaking changes
- ✅ Best user experience

```toml
[dependencies]
jxl-oxide = { version = "0.12", optional = true }

[features]
jxl = ["jxl-oxide"]
```

**Rationale:** JXL is the easiest win - pure Rust, no system deps, great performance.

### Short-term (Phase 1 continued)

**Priority: AVIF and HEIC Native Decoders**

```toml
[dependencies]
image = { version = "0.24", default-features = false, features = ["jpeg", "png"] }
libheif-rs = { version = "2.6", optional = true }

[features]
default = ["jpeg", "png", "gif", "bmp", "tiff", "webp"]
avif = ["image/avif"]
heic = ["libheif-rs"]
jxl = ["jxl-oxide"]
native-formats = ["avif", "jxl", "heic"]
```

**Rationale:** Complete the native decoder support while keeping ffmpeg as fallback.

---

## Documentation Updates

### README.md

Add feature flags section:
```markdown
## Feature Flags

### Image Format Support

**Default:** JPEG, PNG, GIF, BMP, TIFF, WebP (always available)

**Optional Native Decoders:**
- `avif` - Native AVIF decoding via dav1d (requires libdav1d)
- `jxl` - Native JPEG XL decoding (pure Rust, no system deps!)
- `heic` - Native HEIC decoding via libheif (requires libheif)
- `native-formats` - Enable all native decoders

**Usage:**
```bash
# With JPEG XL support (pure Rust, recommended)
cargo build --features jxl

# With all native decoders
cargo build --features native-formats

# Default (ffmpeg fallback for AVIF/JXL/HEIC)
cargo build
```

**Note:** Without native features, AVIF/JXL/HEIC will use ffmpeg conversion if available.
```

### NATIVE_DECODING.md (New File)

Complete guide covering:
- Feature flag options
- System dependencies for each format
- Installation instructions
- Performance comparison (native vs ffmpeg)
- When to use which approach
- Troubleshooting

---

## Performance Expectations

### Decoding Speed (2160×1440 image)

| Format | ffmpeg | Native | Improvement |
|--------|--------|--------|-------------|
| AVIF | 0.5-1.5s | 0.1-0.3s | **3-5x faster** |
| JXL | 0.2-0.8s | 0.05-0.2s | **2-4x faster** |
| HEIC | 0.3-0.5s | 0.1-0.2s | **2-3x faster** |

### Memory Usage

| Approach | Peak Memory |
|----------|-------------|
| ffmpeg | Base + Image + Temp File |
| Native | Base + Image (no temp file) |
| **Savings** | **~2-10 MB per image** |

---

## Risk Analysis

### Low Risk ✅
- **JXL native decoder** (pure Rust, no system deps)
  - Easy to add
  - No build complexity
  - Works everywhere Rust works

### Medium Risk ⚠️
- **AVIF native decoder** (requires libdav1d)
  - Build complexity on some platforms
  - Mitigation: Keep ffmpeg fallback
  
- **HEIC native decoder** (requires libheif)
  - Build complexity on some platforms
  - Mitigation: Keep ffmpeg fallback

### Mitigation Strategy
- All native decoders are **optional**
- ffmpeg fallback always available
- Clear error messages if system libs missing
- Comprehensive documentation

---

## Success Criteria

✅ **Must Have:**
- [ ] Native decoders implemented for all three formats
- [ ] Feature flags working correctly
- [ ] ffmpeg fallback still functional
- [ ] No breaking changes to existing API
- [ ] All tests passing
- [ ] Documentation complete

✅ **Nice to Have:**
- [ ] Automatic feature detection in CI
- [ ] Performance benchmarks
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] Docker examples for both approaches

---

## Recommendation

### Implement in Order:

1. **JXL (Pure Rust)** - Highest priority
   - No system dependencies
   - Easy win for users
   - Best user experience

2. **HEIC (libheif-rs)** - High priority
   - Most common use case (iPhone photos)
   - Good native decoder available

3. **AVIF (dav1d)** - Medium priority
   - Growing format adoption
   - Integrates with image crate

### Keep ffmpeg as fallback for all formats
- Simple deployment option
- No breaking changes
- Works for users who can't install system libraries

---

## Questions for Discussion

1. Should JXL native decoder be **enabled by default** since it's pure Rust with no dependencies?

2. Should we document the "native-formats" feature prominently or keep it as an advanced option?

3. Do we need performance benchmarks before implementation, or document improvements afterward?

4. Should we add automatic feature detection in build.rs to warn about missing system libraries?

5. Docker/CI: Should we provide both "slim" (ffmpeg) and "native" (with system libs) builds?

---

## Conclusion

**Recommended Approach:**
- ✅ Add optional native decoders for all three formats
- ✅ Keep ffmpeg as fallback (maintains backward compatibility)
- ✅ Start with JXL (pure Rust, easiest win)
- ✅ Document both approaches clearly
- ✅ Let users choose based on their deployment needs

This provides maximum flexibility while maintaining the simple "just install ffmpeg" option for users who prefer it.

**Next Steps:**
1. Get approval on this proposal
2. Implement JXL native decoder first (pure Rust)
3. Add HEIC and AVIF native decoders
4. Update documentation
5. Add performance benchmarks
6. Test on all platforms