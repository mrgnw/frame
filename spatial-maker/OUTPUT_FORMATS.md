# Output Formats Guide

This document explains the different output formats available in `spatial-maker` and the naming conventions used.

## Overview

`spatial-maker` can generate stereo images in multiple formats:

| Format | File Type | Use Case | Naming |
|--------|-----------|----------|--------|
| **Left-Right (LR)** | JPG/PNG | Desktop viewing, VR headsets | `-lr.jpg` |
| **Top-Bottom (TB)** | JPG/PNG | Some VR formats, compatibility | `-tb-stereo.jpg` |
| **Separate** | JPG/PNG pair | Processing, individual access | `-L.jpg`, `-R.jpg` |
| **Spatial (MV-HEVC)** | HEIC | Apple devices, true 3D viewing | `-lr.heic` |

## File Naming Convention

### Stereo Pair Layouts

When generating stereo layouts (desktop-viewable formats), use these naming patterns:

#### Left-Right Stereo
- **Pattern:** `{name}-lr.{ext}`
- **Example:** `example-humanos-lr.jpg`
- **Description:** Side-by-side stereo pair (left image | right image)
- **Aspect ratio:** 2:1 (double width)
- **Viewing:** Cross-eyed or using a stereo viewer

#### Top-Bottom Stereo
- **Pattern:** `{name}-tb-stereo.{ext}`
- **Example:** `example-humanos-tb-stereo.jpg`
- **Description:** Top-bottom stereo pair (top: left image, bottom: right image)
- **Aspect ratio:** 1:2 (double height)
- **Viewing:** Some VR formats, anaglyph viewers

#### Separate Stereo
- **Pattern:** `{name}_L.{ext}` and `{name}_R.{ext}`
- **Example:** `example-humanos_L.jpg`, `example-humanos_R.jpg`
- **Description:** Two individual images (left eye and right eye)
- **Use case:** Individual processing, format conversion

### Spatial (MV-HEVC) Format

- **Pattern:** `{name}-lr.heic`
- **Example:** `example-humanos-lr.heic`
- **Description:** MV-HEVC encoded spatial photo
- **Viewing:** Apple Photos, Apple Vision Pro, any MV-HEVC compatible viewer
- **Advantages:**
  - Displays as single 3D image (not stereo pair)
  - Contains stereo metadata (camera distance, disparity adjustment)
  - Optimized file size
  - Apple device native support

## Generating Each Format

### Left-Right Stereo (Desktop Viewing)

```bash
cargo run --example photo -- \
  --input photo.jpg \
  --output photo-lr.jpg \
  --format side-by-side \
  --encoder s
```

**Output:** `photo-lr.jpg` (16MB for a 6000×4000 image)

View by:
- Viewing with cross-eyed or parallel stereo technique
- Using a stereo viewer device
- Red-cyan anaglyph glasses (after format conversion)

### Top-Bottom Stereo

```bash
cargo run --example photo -- \
  --input photo.jpg \
  --output photo-tb-stereo.jpg \
  --format top-bottom \
  --encoder s
```

**Output:** `photo-tb-stereo.jpg` (16MB for a 6000×4000 image)

### Separate Files

```bash
cargo run --example photo -- \
  --input photo.jpg \
  --output photo_L.jpg \
  --format separate \
  --encoder s
```

**Outputs:** 
- `photo_L.jpg` (left eye)
- `photo_R.jpg` (right eye)

### Spatial Photo (MV-HEVC)

```bash
cargo run --example photo -- \
  --input photo.jpg \
  --output photo-lr.jpg \
  --format side-by-side \
  --encoder s \
  --mvhevc
```

**Outputs:**
- `photo-lr.jpg` (stereo pair intermediate, removed by default)
- `photo-lr.heic` (final spatial image)

**Keep intermediate stereo for reference:**

```bash
cargo run --example photo -- \
  --input photo.jpg \
  --output photo-lr.jpg \
  --format side-by-side \
  --encoder s \
  --mvhevc \
  --keep-intermediate
```

## File Size Comparison

For a typical 6000×4000 photo:

| Format | Size | Compression |
|--------|------|-------------|
| Original JPG | ~5.6MB | - |
| Left-Right Stereo JPG | ~16MB | 150 KB/pixel |
| Top-Bottom Stereo JPG | ~16MB | 150 KB/pixel |
| Separate L+R JPG | ~16MB | 150 KB/pixel |
| Spatial (MV-HEVC) | ~6.7MB | 68 KB/pixel |

*MV-HEVC provides ~2.4x compression over stereo pairs*

## Viewing Instructions

### Desktop (Stereo Pairs)

**Left-Right (-lr.jpg):**
```
Cross-eyed technique:
1. Hold image at arm's length
2. Relax eyes as if looking through the image
3. Two views will appear to merge in the center
4. Adjust distance for focus
```

**Top-Bottom (-tb-stereo.jpg):**
- Requires stereo viewer or special glasses
- View vertically stacked images

### Apple Devices (Spatial .heic)

1. Open in Photos app
2. Supports spatial viewing on:
   - Apple Vision Pro (native 3D)
   - iPhone 15 Pro+ (3D playback)
   - iPad with stereo display
3. Share via AirDrop between compatible devices

### Generic VR/3D Viewers

1. Left-Right format: Most VR headsets support this layout
2. Check documentation for preferred format (some prefer top-bottom)
3. Separate files: Useful for custom VR applications

## Quality Settings

All formats support quality adjustment:

```bash
# Standard JPEG quality (1-100, default 95)
--quality 95

# MV-HEVC quality (0-100, default 95)
--mvhevc-quality 95
```

### Quality vs File Size

| Quality | Stereo Pair | Spatial (MV-HEVC) |
|---------|-------------|-------------------|
| 80 | ~14MB | ~5.5MB |
| 90 | ~15MB | ~6.2MB |
| 95 | ~16MB | ~6.7MB |
| 100 | ~18MB | ~7.5MB |

## Model Sizes and Performance

The `--encoder` flag controls depth estimation model:

| Size | Speed | Quality | Best For |
|------|-------|---------|----------|
| `s` (small) | ~8 sec | Good | Real-time, quick previews |
| `b` (base) | ~15 sec | Excellent | Balanced quality/speed |
| `l` (large) | ~45 sec | Best | Offline, highest quality |

*Times are approximate for 6000×4000 image on Apple Silicon*

## Technical Details

### Stereo Pair Dimensions

After processing, stereo pairs are:
- **Resized to:** 6000×4000 (matches original for consistency)
- **Depth map:** 518×770 (model output)
- **Disparity:** 0-30 pixels (default, configurable)

### MV-HEVC Metadata

When encoded to spatial format, includes:
- **Camera distance:** 65mm (default stereo baseline)
- **Disparity adjustment:** 0.0 (neutral)
- **Field of view:** 80° (estimated from aspect ratio)
- **Color profile:** sRGB IEC61966-2.1

To verify, use `spatial info`:

```bash
spatial info --input photo-lr.heic
```

## Recommendations

### For Social Media
- Use `-lr.jpg` or `-tb-stereo.jpg`
- Quality: 90-95
- Format: JPEG for compatibility

### For VR/Headsets
- Use `-lr.jpg` (most compatible)
- Some headsets prefer `-tb-stereo.jpg`
- Check headset documentation

### For Apple Ecosystem
- Use `--mvhevc` flag
- Results in `.heic` spatial photo
- Native support on all recent Apple devices

### For Archival
- Keep original + `-lr.heic` (spatial format)
- Provides both stereo pair and 3D viewing
- Smallest optimized size

### For Processing
- Use `--format separate`
- Results in `_L.jpg` and `_R.jpg`
- Useful for custom post-processing

## Troubleshooting

### "Spatial image doesn't look 3D"
- Increase `--max-disparity` (default 30, try 40-50)
- Use larger encoder model (`-b` or `-l`)
- Ensure source image has clear depth cues

### "File size too large"
- Reduce `--quality` (e.g., 80 instead of 95)
- Use MV-HEVC format (adds `--mvhevc` flag)
- Consider top-bottom if distribution matters

### "Can't view on device"
- Desktop: Use `-lr.jpg` with cross-eyed viewing
- iOS: Use `--mvhevc` for `.heic` spatial photo
- VR: Check headset supported format

## File Organization

Recommended directory structure:

```
photos/
├── originals/
│   └── example-humanos.jpg
├── stereo/
│   ├── example-humanos-lr.jpg      # Left-right pair
│   ├── example-humanos-tb-stereo.jpg  # Top-bottom pair
│   └── example-humanos_L.jpg       # Separate left
│   └── example-humanos_R.jpg       # Separate right
└── spatial/
    └── example-humanos-lr.heic     # Spatial photo
```

## Summary

| Need | Command | Output | Naming |
|------|---------|--------|--------|
| Desktop viewing | `--format side-by-side` | JPG | `-lr.jpg` |
| VR headset | `--format side-by-side` | JPG | `-lr.jpg` |
| Apple device | `--mvhevc` | HEIC | `-lr.heic` |
| Processing | `--format separate` | JPG pair | `_L.jpg`, `_R.jpg` |
| Vertical viewing | `--format top-bottom` | JPG | `-tb-stereo.jpg` |