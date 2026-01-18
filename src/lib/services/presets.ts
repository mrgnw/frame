import { Store } from "@tauri-apps/plugin-store";
import { v4 as uuidv4 } from "uuid";

import type { ConversionConfig, PresetDefinition } from "$lib/types";

const STORE_PATH = "presets.dat";
const PRESETS_KEY = "presets";

export const DEFAULT_PRESETS: PresetDefinition[] = [
  {
    id: "balanced-mp4",
    name: "Balanced MP4",
    builtIn: true,
    config: {
      container: "mp4",
      videoCodec: "libx264",
      audioCodec: "aac",
      resolution: "original",
      crf: 23,
      preset: "medium",
    },
  },
  {
    id: "archive-hq",
    name: "Archive H.265",
    builtIn: true,
    config: {
      container: "mkv",
      videoCodec: "libx265",
      audioCodec: "ac3",
      resolution: "original",
      crf: 18,
      preset: "slow",
    },
  },
  {
    id: "web-share",
    name: "Web Share",
    builtIn: true,
    config: {
      container: "webm",
      videoCodec: "vp9",
      audioCodec: "libopus",
      resolution: "720p",
      crf: 30,
      preset: "medium",
    },
  },
  {
    id: "audio-only",
    name: "Audio Only",
    builtIn: true,
    config: {
      container: "mp3",
      videoCodec: "libx264",
      audioCodec: "mp3",
      resolution: "original",
      crf: 23,
      preset: "medium",
    },
  },
];

let storePromise: Promise<Store> | null = null;

async function getStore(): Promise<Store> {
  if (!storePromise) {
    storePromise = Store.load(STORE_PATH, {
      defaults: {
        [PRESETS_KEY]: [],
      },
    });
  }

  return storePromise;
}

export async function loadCustomPresets(): Promise<PresetDefinition[]> {
  try {
    const store = await getStore();
    const presets = await store.get<PresetDefinition[]>(PRESETS_KEY);
    return presets ?? [];
  } catch (error) {
    console.error("Failed to load presets from store", error);
    return [];
  }
}

export async function saveCustomPresets(
  presets: PresetDefinition[],
): Promise<void> {
  const store = await getStore();
  await store.set(PRESETS_KEY, presets);
  await store.save();
}

export function createCustomPreset(
  name: string,
  config: ConversionConfig,
): PresetDefinition {
  return {
    id: uuidv4(),
    name: name.trim() || "Untitled Preset",
    config: cloneConfig(config),
  };
}

export function cloneConfig(config: ConversionConfig): ConversionConfig {
  return JSON.parse(JSON.stringify(config));
}

export function getDefaultConfig(): ConversionConfig {
  return cloneConfig(DEFAULT_PRESETS[0].config);
}
