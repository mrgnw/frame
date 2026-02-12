import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { SpatialConfig } from '../types';

export interface SpatialProgressEvent {
	id: string;
	progress: number;
	stage: string;
}

export interface SpatialCompletedEvent {
	id: string;
	outputPath: string;
}

export interface SpatialErrorEvent {
	id: string;
	error: string;
}

export interface SpatialLogEvent {
	id: string;
	line: string;
}

export interface SpatialStartedEvent {
	id: string;
}

export async function startSpatial(id: string, filePath: string, config: SpatialConfig) {
	try {
		await invoke('queue_spatial', { id, filePath, config });
	} catch (error) {
		console.error('Failed to queue spatial conversion:', error);
		throw error;
	}
}

export async function cancelSpatial(id: string) {
	try {
		await invoke('cancel_spatial', { id });
	} catch (error) {
		console.error('Failed to cancel spatial conversion:', error);
		throw error;
	}
}

export async function setupSpatialListeners(
	onProgress: (payload: SpatialProgressEvent) => void,
	onCompleted: (payload: SpatialCompletedEvent) => void,
	onError: (payload: SpatialErrorEvent) => void,
	onLog: (payload: SpatialLogEvent) => void,
	onStarted: (payload: SpatialStartedEvent) => void
): Promise<UnlistenFn> {
	const unlistenStarted = await listen<SpatialStartedEvent>('spatial-started', (event) => {
		onStarted(event.payload);
	});

	const unlistenProgress = await listen<SpatialProgressEvent>('spatial-progress', (event) => {
		onProgress(event.payload);
	});

	const unlistenCompleted = await listen<SpatialCompletedEvent>('spatial-completed', (event) => {
		onCompleted(event.payload);
	});

	const unlistenError = await listen<SpatialErrorEvent>('spatial-error', (event) => {
		onError(event.payload);
	});

	const unlistenLog = await listen<SpatialLogEvent>('spatial-log', (event) => {
		onLog(event.payload);
	});

	return () => {
		unlistenStarted();
		unlistenProgress();
		unlistenCompleted();
		unlistenError();
		unlistenLog();
	};
}
