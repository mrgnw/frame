# Image Format Support Guide

## Supported Input Formats

### ✅ Directly Supported (No Conversion Needed)

These formats are natively supported by spatial-maker and can be used as input without any preprocessing:

| Format | Extensions | Notes |
|--------|-----------|-------|
| **JPEG** | `.jpg`, `.jpeg` | Most compatible, recommended for quality/size balance |
| **PNG** | `.png` | Lossless, supports transparency (converted to RGB) |
| **GIF** | `.gif` | Animated GIFs use first frame |
| **BMP** | `.bmp` | Uncompressed, larger files |
| **TIFF** | `.tiff`, `.tif` | High-quality, larger files |
| **WebP** | `.webp` | Modern format, good compression |

### 📋 Formats Requiring Conversion

These formats are supported as **outputs** but require conversion before use as input:

| Format | Extension | Why Conversion Needed | Conversion Tools |
|--------|-----------|----------------------|------------------|
| **AVIF** | `.avif` | Advanced codec, limited decoder availability | ffmpeg, ImageMagick |
| **JPEG XL** | `.jxl` | Next-gen format, limited codec availability | ffmpeg, ImageMagick |
| **HEIC/HEIF** | `.heic`, `.heif` | Apple's proprietary format | ffmpeg, ImageMagick, heic-decode |

---

## Quick Start: Converting Advanced Formats

### Convert HEIC (from iPhone) to JPEG

```bash
# Using ffmpeg (recommended)
ffmpeg -i photo.heic -c:v libjpeg -q:v 2 photo.jpg

# Using ImageMagick
convert photo.heic photo.jpg

# Using macOS heic-decode tool
heic-decode -i photo.heic -o photo.jpg
```

### Convert AVIF to JPEG

```bash
# Using ffmpeg
ffmpeg -i photo.avif -c:v libjpeg -q:v 2 photo.jpg

# Using ImageMagick
convert photo.avif photo.jpg
```

### Convert JPEG XL to JPEG

```bash
# Using ffmpeg
ffmpeg -i photo.jxl -c:v libjpeg -q:v 2 photo.jpg

# Using ImageMagick
convert photo.jxl photo.jpg
```

### Batch Convert Multiple HEIC Files

```bash
# Convert all HEIC files in directory to JPEG
for file in *.heic; do
  ffmpeg -i "$file" -c:v libjpeg -q:v 2 "${file%.heic}.jpg"
done
```

---

## Recommended Input Formats

### For Best Quality
```
PNG (lossless)
TIFF (high-quality professional)
JPEG at quality 90+ (excellent balance)
```

### For Best Compatibility
```
JPEG (widely supported)
PNG (fallback option)
```

### For File Size
```
WebP (modern compression)
JPEG (traditional compression)
```

### For iPhone/Apple Photos
```
Convert HEIC → JPEG first
Then use JPEG as input
```

---

## Why Some Formats Need Conversion

### HEIC (High Efficiency Image Container)
- Apple's proprietary container format
- Uses HEVC video codec for compression
- Great compression but limited ecosystem support
- No standard Rust decoder available
- **Solution:** Convert to JPEG or PNG before processing

### AVIF (AV1 Image Format)
- Next-generation image format based on AV1 video codec
- Excellent compression (20-30% better than JPEG)
- Limited decoder availability in Rust ecosystem
- **Solution:** Convert to JPEG or PNG before processing

### JPEG XL (JXL)
- Modern image format designed to replace JPEG
- Superior compression and quality
- Better than AVIF in some scenarios
- Limited decoder availability
- **Solution:** Convert to JPEG or PNG before processing

---

## Output Format Support

spatial-maker supports these output formats for stereo images and spatial photos:

| Format | Type | Use Case |
|--------|------|----------|
| **JPEG** | Raster | Desktop viewing, web sharing, social media |
| **PNG** | Raster | Archival, lossless quality, transparency |
| **HEIC** | Container | Apple devices, spatial photos, MV-HEVC |

You can specify output format with:
```bash
./target/release/examples/photo \
  --input photo.jpg \
  --output photo-lr.jpg \
  --image-format jpeg    # or 'png'
```

---

## Installation: Conversion Tools

### Install ffmpeg

**macOS (Homebrew)**
```bash
brew install ffmpeg
```

**Ubuntu/Debian**
```bash
sudo apt-get install ffmpeg
```

**Windows (Chocolatey)**
```bash
choco install ffmpeg
```

### Install ImageMagick

**macOS (Homebrew)**
```bash
brew install imagemagick
```

**Ubuntu/Debian**
```bash
sudo apt-get install imagemagick
```

**Windows (Chocolatey)**
```bash
choco install imagemagick
```

### Install heic-decode (macOS only)

```bash
brew install heic-decode
```

---

## Workflow: iPhone Photo to Spatial

If you want to process a photo taken on iPhone:

### Step 1: Get the HEIC File
```bash
# Copy from iPhone, Google Photos, iCloud, etc.
# File will be in .heic format
```

### Step 2: Convert to JPEG
```bash
ffmpeg -i IMG_1234.heic -c:v libjpeg -q:v 2 IMG_1234.jpg
```

### Step 3: Process with spatial-maker
```bash
./target/release/examples/photo \
  --input IMG_1234.jpg \
  --output IMG_1234-lr.jpg \
  --format side-by-side \
  --encoder s \
  --mvhevc
```

### Step 4: View Results
- **Desktop:** Open `IMG_1234-lr.jpg` with cross-eyed viewing
- **iPhone/iPad:** Open `IMG_1234-lr.heic` in Photos app for 3D viewing

---

## Format Comparison

### File Size Example (6000×4000 photo)

| Input Format | File Size | Notes |
|-------------|-----------|-------|
| JPEG (q95) | 5.6 MB | Baseline |
| HEIC | 3.8 MB | 32% smaller |
| PNG (8-bit) | 15-20 MB | Lossless, much larger |
| WebP (q90) | 4.2 MB | 25% smaller than JPEG |
| AVIF (q60) | 2.5 MB | 55% smaller (requires conversion) |
| TIFF | 72 MB | Uncompressed |

### Processing Time (Same 6000×4000 photo)

| Input Format | Load Time | Reason |
|-------------|-----------|--------|
| JPEG | <100ms | Fast decompression |
| PNG | 100-150ms | Larger file, no compression |
| HEIC | N/A | Requires conversion first |
| WebP | 50-100ms | Efficient decompression |

---

## Troubleshooting

### "Unsupported image format: .heic"

**Problem:** You're trying to process a HEIC file directly
**Solution:** Convert to JPEG first
```bash
ffmpeg -i photo.heic photo.jpg
```

### "Unsupported image format: .avif"

**Problem:** You're trying to process an AVIF file directly
**Solution:** Convert to JPEG or PNG first
```bash
ffmpeg -i photo.avif photo.jpg
```

### "Failed to load image: unsupported image type"

**Problem:** Format not recognized or corrupted file
**Solutions:**
1. Check file extension matches actual format
2. Verify file is not corrupted
3. Try converting to JPEG as intermediate format

### "File has no extension"

**Problem:** Image file has no extension
**Solution:** Add correct extension
```bash
mv image image.jpg      # if it's a JPEG
mv image image.png      # if it's a PNG
```

---

## Best Practices

### For Maximum Compatibility
```
Input:  JPEG (quality 90+) or PNG
Output: spatial-maker generates JPEG/HEIC automatically
```

### For Maximum Quality
```
Input:  PNG (lossless) or TIFF (professional)
Output: Same format maintained through processing
```

### For Apple Ecosystem
```
Input:  Convert HEIC → JPEG first
Output: Use --mvhevc flag to generate HEIC spatial photos
```

### For Web Sharing
```
Input:  JPEG (good compression)
Output: JPEG stereo pair or WebP (if target supports it)
```

### For Archival
```
Input:  PNG or TIFF (lossless)
Output: Keep original + HEIC spatial photo together
```

---

## Advanced: Batch Processing with Format Conversion

### Process All iPhone Photos in a Directory

```bash
#!/bin/bash

# Convert HEIC to JPEG, then process
for heic_file in *.heic; do
  jpg_file="${heic_file%.heic}.jpg"
  
  # Convert HEIC to JPEG
  echo "Converting $heic_file..."
  ffmpeg -i "$heic_file" -c:v libjpeg -q:v 2 "$jpg_file"
  
  # Process to spatial
  echo "Processing $jpg_file..."
  ./target/release/examples/photo \
    --input "$jpg_file" \
    --output "${jpg_file%.jpg}-spatial.jpg" \
    --format side-by-side \
    --encoder s
done

echo "Done! Check *-spatial.jpg files"
```

### Process Mixed Formats

```bash
#!/bin/bash

for input_file in *; do
  ext="${input_file##*.}"
  
  case "$ext" in
    heic|heif)
      # Convert HEIC to JPEG
      temp_file="${input_file%.*}.jpg"
      ffmpeg -i "$input_file" -c:v libjpeg -q:v 2 "$temp_file"
      input_file="$temp_file"
      ;;
    avif)
      # Convert AVIF to JPEG
      temp_file="${input_file%.*}.jpg"
      ffmpeg -i "$input_file" -c:v libjpeg -q:v 2 "$temp_file"
      input_file="$temp_file"
      ;;
  esac
  
  # Process file if it's now in supported format
  if [[ "$input_file" =~ \.(jpg|jpeg|png|gif|bmp|tiff|webp)$ ]]; then
    ./target/release/examples/photo \
      --input "$input_file" \
      --output "${input_file%.*}-spatial.jpg" \
      --format side-by-side \
      --encoder s
  fi
done
```

---

## Technical Details

### Why Direct HEIC Support is Difficult

1. **No Standard Rust Library**
   - HEIC is Apple's proprietary container
   - Requires HEVC video codec
   - Most Rust HEIC libraries are incomplete or unmaintained

2. **FFmpeg Alternative**
   - FFmpeg has excellent HEIC support
   - Works cross-platform
   - Simple one-line conversion
   - Recommended approach

3. **Performance**
   - Converting HEIC → JPEG adds ~1-2 seconds
   - Minimal compared to depth estimation (~1.1 seconds)
   - Negligible impact on overall pipeline

### Color Space Handling

spatial-maker converts all input images to RGB8 format:
- JPEG: Direct RGB conversion
- PNG: Alpha channel dropped if present
- Others: Automatic conversion via image crate

### Quality Preservation

Conversion recommendations to preserve quality:
```bash
# HEIC → JPEG
ffmpeg -i photo.heic -c:v libjpeg -q:v 2 photo.jpg

# AVIF → JPEG
ffmpeg -i photo.avif -c:v libjpeg -q:v 2 photo.jpg

# JXL → JPEG
ffmpeg -i photo.jxl -c:v libjpeg -q:v 2 photo.jpg
```

The `-q:v 2` flag sets JPEG quality to ~95 (highest quality).

---

## Summary

| Scenario | Input Format | Action |
|----------|--------------|--------|
| Desktop JPEG | JPEG | Use directly ✅ |
| Desktop PNG | PNG | Use directly ✅ |
| iPhone HEIC | HEIC | Convert to JPEG → Use ➡️ |
| AVIF file | AVIF | Convert to JPEG → Use ➡️ |
| JXL file | JXL | Convert to JPEG → Use ➡️ |

**Key Point:** All advanced formats (HEIC, AVIF, JXL) require one-line conversion to JPEG/PNG before use with spatial-maker. This is minimal overhead compared to the depth estimation process.