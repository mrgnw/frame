<div align="center">
  <img src="./icon.png" width="128" height="128" alt="Frame Icon" />
  <h1>Frame</h1>
</div>

<div align="center">

[English](./README.md) | [简体中文](./readme/zh-CN.md) | [日本語](./readme/ja-JP.md) | [한국어](./readme/ko-KR.md) | [Español](./readme/es-ES.md) | [Русский](./readme/ru-RU.md) | [Français](./readme/fr-FR.md) | [Deutsch](./readme/de-DE.md) | [Italiano](./readme/it-IT.md)

</div>

<div align="center">
	<img src="https://img.shields.io/badge/Tauri-v2-orange?style=flat-square&logo=tauri" alt="Tauri" />
	<img src="https://img.shields.io/badge/Svelte-v5-red?style=flat-square&logo=svelte" alt="Svelte" />
	<img src="https://img.shields.io/badge/Rust-Edition_2024-black?style=flat-square&logo=rust" alt="Rust" />
	<img src="https://img.shields.io/badge/TypeScript-5.9.3-blue?style=flat-square&logo=typescript" alt="TypeScript" />
	<img src="https://img.shields.io/badge/Tailwind_CSS-v4-38bdf8?style=flat-square&logo=tailwindcss" alt="Tailwind" />
	<img src="https://img.shields.io/badge/license-GPL--3.0-green?style=flat-square" alt="License" />
</div>

**Frame** is a high-performance media conversion utility built on the Tauri v2 framework. It provides a native interface for FFmpeg operations, allowing for granular control over video and audio transcoding parameters. The application leverages a Rust-based backend for concurrent task management and process execution, coupled with a Svelte 5 frontend for configuration and state monitoring.

This fork adds **Spatial Video** conversion: turn any 2D video into a stereoscopic 3D spatial video for Apple Vision Pro using AI depth estimation.

<br />
<div align="center">
  <img src="./preview.png" alt="Frame Application Preview" width="800" />
</div>
<br />

> [!WARNING]  
> **Unsigned Application Notice**
> Since the application is currently unsigned, your operating system will flag it:
>
> - **macOS:** The system will flag the app and its sidecar binaries with a quarantine attribute. To run the app, remove the attribute manually:
>   ```bash
>   xattr -dr com.apple.quarantine /Applications/Frame.app
>   ```
> - **Windows:** Windows SmartScreen may prevent the application from starting. Click **"More info"** and then **"Run anyway"** to proceed.

## Features

### Media Conversion Core

- **Container Support:** `mp4`, `mkv`, `webm`, `mov`, `mp3`, `m4a`, `wav`, `flac`.
- **Video Encoders:**
  - `libx264` (H.264 / AVC)
  - `libx265` (H.265 / HEVC)
  - `vp9` (Google VP9)
  - `prores` (Apple ProRes)
  - `libsvtav1` (Scalable Video Technology AV1)
  - **Hardware Acceleration:** `h264_videotoolbox` (Apple Silicon), `h264_nvenc` (NVIDIA).
- **Audio Encoders:** `aac`, `ac3` (Dolby Digital), `libopus`, `mp3`, `alac` (Apple Lossless), `flac` (Free Lossless Audio Codec), `pcm_s16le` (WAV).
- **Bitrate Control:** Constant Rate Factor (CRF) or Target Bitrate (kbps).
- **Scaling:** Bicubic, Lanczos, Bilinear, Nearest Neighbor.
- **Metadata Probing:** Automated extraction of stream details (codec, duration, bitrate, channel layout) via `ffprobe`.
- **AI Upscaling:** Integrated `Real-ESRGAN` for high-quality video upscaling (x2, x4).

### Spatial Video (macOS only)

- **2D to 3D conversion** using [Depth Anything V2](https://github.com/DepthAnything/Depth-Anything-V2) depth estimation and DIBR (Depth Image Based Rendering).
- **Apple Vision Pro output** as MV-HEVC spatial video via the [`spatial`](https://blog.mikeswanson.com/spatial_docs/) CLI.
- **Configurable depth model** size (small/medium/large) and 3D intensity.
- **Progress tracking** with per-frame updates through the pipeline stages.

### Architecture & Workflow

- **Concurrent Processing:** Async task queue manager implemented in Rust (`tokio::mpsc`) limiting concurrent FFmpeg processes (default: 2).
- **Real-time Telemetry:** Stream parsing of FFmpeg `stderr` for accurate progress tracking and log output.
- **Preset Management:** Configuration persistence for reusable conversion profiles.

## Build from Source

### 1. Prerequisites

- **Rust:** [Install Rust](https://www.rust-lang.org/tools/install)
- **Bun (or Node.js):** [Install Bun](https://bun.sh/)
- **OS Dependencies:** Follow the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system.

### 2. Clone and install

```bash
git clone https://github.com/mrgnw/frame.git
cd frame
bun install
```

### 3. Setup sidecar binaries

Frame requires FFmpeg/FFprobe sidecar binaries and Real-ESRGAN sidecar assets for AI upscaling:

```bash
bun run setup:ffmpeg
bun run setup:upscaler
```

### 4. Setup spatial video (macOS only)

The spatial video feature requires three additional things: the `spatial` CLI, the `spatial-maker` Python pipeline, and depth model checkpoints.

**Install the spatial CLI** (converts side-by-side stereo to MV-HEVC):

```bash
brew install spatial
```

**Install spatial-maker** (the AI depth estimation pipeline):

```bash
uv tool install spatial-maker
```

> Requires [uv](https://docs.astral.sh/uv/getting-started/installation/). This installs `spatial-maker` from [PyPI](https://pypi.org/project/spatial-maker/) along with its dependencies (PyTorch, OpenCV, Depth Anything V2).

**Download depth model checkpoints** (you only need the sizes you plan to use):

```bash
mkdir -p ~/.spatial-maker/checkpoints

# Small (fastest, 24.8M params) - recommended to start
curl -L -o ~/.spatial-maker/checkpoints/depth_anything_v2_vits.pth \
  https://huggingface.co/depth-anything/Depth-Anything-V2-Small/resolve/main/depth_anything_v2_vits.pth

# Base (balanced, 97.5M params)
curl -L -o ~/.spatial-maker/checkpoints/depth_anything_v2_vitb.pth \
  https://huggingface.co/depth-anything/Depth-Anything-V2-Base/resolve/main/depth_anything_v2_vitb.pth

# Large (best quality, 335.3M params)
curl -L -o ~/.spatial-maker/checkpoints/depth_anything_v2_vitl.pth \
  https://huggingface.co/depth-anything/Depth-Anything-V2-Large/resolve/main/depth_anything_v2_vitl.pth
```

### 5. Run

```bash
bun tauri dev
```

Production build:

```bash
bun tauri build
```

## Usage

### Standard Conversion

1. **Add files** using the + button or drag and drop.
2. **Configure** in the settings panel on the right:
   - **Source:** View detected file metadata.
   - **Output:** Select container format and output filename.
   - **Video:** Configure codec, bitrate/CRF, resolution, and framerate.
   - **Audio:** Select codec, bitrate, channels, and specific tracks.
   - **Presets:** Save and load reusable conversion profiles.
3. Click **Start** to begin conversion.
4. Monitor progress and logs in real time.

### Spatial Video

1. **Add a video file** as above.
2. Open the **Spatial tab** (glasses icon) in the settings panel to configure:
   - **Depth Model:** Small (fast), Medium (balanced), or Large (best quality).
   - **3D Intensity:** Controls the stereo disparity in pixels (20-30 subtle, 30-40 moderate, 40-50 strong).
   - **Skip Downscale:** Keep original resolution instead of normalizing to 1080p@24fps.
3. Click the **Spatial** button in the titlebar (next to Start).
4. The pipeline runs through 4 stages: downscale, depth estimation + stereo, audio mux, and MV-HEVC packaging.
5. Output is a `.mov` file viewable on Apple Vision Pro.

## Acknowledgments & Third-Party Code

- **Real-ESRGAN**: Copyright (c) 2021, Xintao Wang. Licensed under [BSD 3-Clause](https://github.com/xinntao/Real-ESRGAN/blob/master/LICENSE).
- **FFmpeg**: Licensed under [GPLv3](https://www.ffmpeg.org/legal.html).
- **Depth Anything V2**: Yang et al. Licensed under [Apache 2.0](https://github.com/DepthAnything/Depth-Anything-V2/blob/main/LICENSE).
- **spatial CLI**: Mike Swanson. [Documentation](https://blog.mikeswanson.com/spatial_docs/).
- **Frame** (upstream): [66HEX/frame](https://github.com/66HEX/frame). Licensed under GPL-3.0.

## License

GPLv3 License. See [LICENSE](LICENSE) for details.
