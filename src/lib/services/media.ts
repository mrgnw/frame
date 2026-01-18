import { invoke } from "@tauri-apps/api/core";
import type { SourceMetadata } from "$lib/types";

export async function probeMedia(filePath: string): Promise<SourceMetadata> {
  return invoke("probe_media", { filePath });
}
