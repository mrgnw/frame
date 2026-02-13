# Automatic Format Conversion

## Overview

`spatial-maker` now supports automatic conversion of advanced image formats (AVIF, JPEG XL, HEIC) to JPEG before processing. This means you can pass these formats directly to the CLI or library without manually converting them first.

## Supported Formats

### Native Support (No Conversion Needed)
- **JPEG** (.jpg, .jpeg)
- **PNG** (.png)
- **GIF** (.gif)
- **BMP** (.bmp)
- **TIFF** (.tiff, .tif)
- **WebP** (.webp)

### Automatic Conversion (Requires ffmpeg)
- **AVIF** (.avif) → Converted to JPEG
- **JPEG XL** (.jxl) → Converted to JPEG
- **HEIC/HEIF** (.heic, .heif) → Converted to JPEG

## Requirements

Automatic conversion requires **ffmpeg** to be installed and available in your system PATH.

### Installation

#### macOS
```bash
brew install ffmpeg
```

#### Ubuntu/Debian
```bash
sudo apt-get install ffmpeg
```

#### Windows (using Chocolatey)
```bash
choco install ffmpeg
```

#### Windows (using scoop)
```bash
scoop install ffmpeg
```

#### Verify Installation
```bash
ffmpeg -version
```

## Usage

### Command Line

Simply pass your AVIF/JXL/HEIC image directly to the CLI:

```bash
# Convert HEIC image to spatial photo
cargo run --example photo -- --input photo.heic --output spatial.jpg

# Convert AVIF image
cargo run --example photo -- --input photo.avif --output spatial.jpg

# Convert JPEG XL image
cargo run --example photo -- --input photo.jxl --output spatial.jpg
```

The conversion happens automatically and transparently. The temporary converted JPEG is cleaned up after processing.

### Library Usage

```rust
use spatial_maker::{process_photo, SpatialConfig, OutputOptions, OutputFormat, ImageEncoding};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let spatial_config = SpatialConfig {
        encoder_size: "s".to_string(),
        max_disparity: 30,
        target_depth_size: 518,
        use_coreml: true,
    };

    let output_options = OutputOptions {
        layout: OutputFormat::SideBySide,
        image_format: ImageEncoding::Jpeg { quality: 95 },
        mvhevc: None,
    };

    // Just pass your HEIC/AVIF/JXL file directly!
    process_photo(
        Path::new("photo.heic"),
        Path::new("output.jpg"),
        spatial_config,
        output_options,
    ).await?;

    Ok(())
}
```

## How It Works

1. **Detection**: File extension is checked to identify the format
2. **ffmpeg Check**: System is checked for ffmpeg availability
3. **Conversion**: Image is converted to high-quality JPEG in a temporary file
4. **Processing**: The converted JPEG is processed normally
5. **Cleanup**: Temporary file is automatically deleted

## Conversion Parameters

The automatic conversion uses these ffmpeg parameters for optimal quality:

- **Codec**: `libjpeg` (standard JPEG encoder)
- **Quality**: `2` (highest quality, scale 1-31 where lower is better)
- **Output Format**: JPEG (.jpg)

This results in high-quality output suitable for depth estimation and stereo generation.

## Error Handling

If ffmpeg is not installed, you'll see a helpful error message:

```
AVIF format requires ffmpeg for automatic conversion.
ffmpeg is not installed or not in PATH.

Please install ffmpeg:
  macOS:   brew install ffmpeg
  Ubuntu:  sudo apt-get install ffmpeg
  Windows: choco install ffmpeg

Or manually convert your file to JPEG:
  ffmpeg -i photo.avif -c:v libjpeg -q:v 2 output.jpg
```

## Manual Conversion

If you prefer to convert images manually before processing, you can use ffmpeg directly:

```bash
# Convert HEIC to JPEG
ffmpeg -i photo.heic -c:v libjpeg -q:v 2 photo.jpg

# Convert AVIF to JPEG
ffmpeg -i photo.avif -c:v libjpeg -q:v 2 photo.jpg

# Convert JPEG XL to JPEG
ffmpeg -i photo.jxl -c:v libjpeg -q:v 2 photo.jpg

# Using ImageMagick as alternative
convert photo.heic photo.jpg
convert photo.avif photo.jpg
convert photo.jxl photo.jpg
```

## Performance

Conversion happens once before processing and adds minimal overhead:

- **Conversion Time**: Typically 1-3 seconds depending on image size and system performance
- **Quality Loss**: Minimal with high-quality JPEG settings (q=2)
- **Disk Usage**: Temporary file is cleaned up immediately after loading

## Troubleshooting

### "ffmpeg not found"
Make sure ffmpeg is installed and in your PATH. Test with `ffmpeg -version`.

### "Conversion failed"
Check that:
1. The input file exists and is readable
2. You have sufficient disk space in the temp directory
3. The file is a valid image in the expected format

### "Out of disk space"
Conversion creates a temporary file. Ensure you have at least as much free disk space as your image file size.

## Technical Details

### Temporary File Location
- **macOS/Linux**: `/tmp/`
- **Windows**: `%TEMP%\`
- Files follow pattern: `spatial_maker_convert_{format}_{timestamp}.jpg`

### Supported Input Formats for Conversion
The following formats can be converted automatically:
- AVIF (AV1 Image Format)
- JXL (JPEG XL)
- HEIC (High Efficiency Image Container)
- HEIF (High Efficiency Image Format)

### Why JPEG as Intermediate Format?
- Fast, universal support across all image processing libraries
- Excellent quality-to-file-size ratio
- Lossless conversion possible with quality setting 2
- Minimal performance impact on downstream processing