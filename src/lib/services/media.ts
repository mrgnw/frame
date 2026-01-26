import { invoke } from '@tauri-apps/api/core';
import type { SourceMetadata } from '$lib/types';

export async function probeMedia(filePath: string): Promise<SourceMetadata> {
	return invoke('probe_media', { filePath });
}

export function isAudioCodecAllowed(codec: string, container: string): boolean {
	if (container === 'mp3') return codec === 'mp3';
	if (container === 'wav') return codec === 'pcm_s16le';
	if (container === 'flac') return codec === 'flac';
	if (container === 'm4a') return codec === 'aac' || codec === 'alac';
	if (container === 'mp4') return ['aac', 'ac3', 'libopus', 'mp3', 'alac'].includes(codec);
	if (container === 'mov') return true;
	if (container === 'mkv') return true;
	if (container === 'webm') return ['libopus', 'vorbis'].includes(codec);
	return true;
}

export function getDefaultAudioCodec(container: string): string {
	switch (container) {
		case 'mp3':
			return 'mp3';
		case 'wav':
			return 'pcm_s16le';
		case 'flac':
			return 'flac';
		case 'm4a':
			return 'aac';
		case 'webm':
			return 'libopus';
		default:
			return 'aac';
	}
}
