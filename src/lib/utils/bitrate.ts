import type { ConversionConfig, SourceMetadata } from "$lib/types";

const RESOLUTION_HEIGHTS: Record<string, number> = {
  "480p": 480,
  "720p": 720,
  "1080p": 1080,
};

const BASE_BITRATES: Array<{ height: number; kbps: number }> = [
  { height: 2160, kbps: 25000 },
  { height: 1440, kbps: 16000 },
  { height: 1080, kbps: 8000 },
  { height: 720, kbps: 5000 },
  { height: 480, kbps: 2500 },
  { height: 360, kbps: 1500 },
];

const AUDIO_BITRATES: Record<string, number> = {
  aac: 128,
  ac3: 192,
  libopus: 96,
  mp3: 128,
};

const CODEC_SCALE: Record<string, number> = {
  libx264: 1,
  h264: 1,
  libx265: 0.65,
  h265: 0.65,
  vp9: 0.7,
  "libvpx-vp9": 0.7,
  prores: 1.6,
};

export function parseDurationToSeconds(duration?: string): number | null {
  if (!duration) return null;
  const match = duration.match(/(\d{2}):(\d{2}):(\d{2})\.(\d{2})/);
  if (match) {
    const [, hh, mm, ss, cs] = match;
    const hours = parseInt(hh, 10);
    const minutes = parseInt(mm, 10);
    const seconds = parseInt(ss, 10);
    const centiseconds = parseInt(cs, 10);
    return hours * 3600 + minutes * 60 + seconds + centiseconds / 100;
  }
  const seconds = parseFloat(duration);
  if (!isNaN(seconds)) {
    return seconds;
  }
  return null;
}

// ...

function parseSourceBitrate(metadata?: SourceMetadata): number | null {
  const raw = metadata?.bitrate;
  if (!raw) return null;

  const clean = raw.replace(/[^0-9.]/g, "");
  const value = parseFloat(clean);
  if (Number.isNaN(value)) return null;

  // Heuristic: If value > 100,000, assume it's in bits per second (bps) and convert to kbps.
  // Example: 2,000,000 bps -> 2000 kbps.
  // If it's 2000, assume it's already kbps (legacy or low bitrate).
  if (value > 100_000) {
    return value / 1000;
  }
  return value;
}

function crfScale(crf: number): number {
  const diff = 23 - crf;
  return Math.pow(2, diff / 6);
}

function audioBitrate(codec: string): number {
  return AUDIO_BITRATES[codec.toLowerCase()] ?? 128;
}

export interface OutputEstimate {
  videoKbps: number;
  audioKbps: number;
  totalKbps: number;
  sizeMb?: number;
}

export function estimateOutput(
  config: ConversionConfig,
  metadata?: SourceMetadata,
): OutputEstimate {
  const isAudioOnly = config.container.toLowerCase() === "mp3";

  let videoKbps = 0;
  if (!isAudioOnly) {
    if (config.videoBitrateMode === "bitrate") {
      videoKbps = parseFloat(config.videoBitrate) || 0;
    } else {
      const height = inferTargetHeight(config, metadata);
      const sourceHeight =
        parseResolutionHeight(metadata?.resolution) || height;
      const sourceVideoBitrate = parseSourceBitrate(metadata);

      let baseKbps = 0;
      if (sourceVideoBitrate) {
        const scaleFactor = Math.pow(height / sourceHeight, 1.75);
        baseKbps = sourceVideoBitrate * scaleFactor;
      } else {
        baseKbps =
          baseVideoBitrate(height) * codecScaleFactor(config.videoCodec || "");
      }

      videoKbps = baseKbps * crfScale(config.crf);
      videoKbps = Math.max(400, videoKbps);
    }
  }

  const audioKbps =
    parseFloat(config.audioBitrate) || audioBitrate(config.audioCodec);
  const totalKbps = videoKbps + audioKbps;

  const durationSeconds = parseDurationToSeconds(metadata?.duration);
  const sizeMb =
    durationSeconds && totalKbps
      ? (totalKbps * durationSeconds) / 8 / 1024
      : undefined;

  return {
    videoKbps: Math.round(videoKbps),
    audioKbps: Math.round(audioKbps),
    totalKbps: Math.round(totalKbps),
    sizeMb: sizeMb ? Math.max(sizeMb, 1) : undefined,
  };
}

export function formatFileSize(sizeMb?: number): string {
  if (!sizeMb) return "—";
  if (sizeMb >= 1024) {
    return `${(sizeMb / 1024).toFixed(1)} GB`;
  }
  return `${sizeMb.toFixed(1)} MB`;
}
