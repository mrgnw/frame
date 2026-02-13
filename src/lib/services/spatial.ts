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

export interface ModelDownloadProgressEvent {
	encoderSize: string;
	bytesDownloaded: number;
	totalBytes: number;
	progress: number;
}

export interface ModelDownloadCompleteEvent {
	encoderSize: string;
}

export interface ModelDownloadErrorEvent {
	encoderSize: string;
	error: string;
}

export async function checkSpatialModels(): Promise<Record<string, boolean>> {
	return invoke('check_spatial_models');
}

export async function downloadSpatialModel(encoderSize: string): Promise<void> {
	return invoke('download_spatial_model', { encoderSize });
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

export async function setupModelDownloadListeners(
	onProgress: (payload: ModelDownloadProgressEvent) => void,
	onComplete: (payload: ModelDownloadCompleteEvent) => void,
	onError: (payload: ModelDownloadErrorEvent) => void
): Promise<UnlistenFn> {
	const unlistenProgress = await listen<ModelDownloadProgressEvent>(
		'spatial-model-download-progress',
		(event) => onProgress(event.payload)
	);

	const unlistenComplete = await listen<ModelDownloadCompleteEvent>(
		'spatial-model-download-complete',
		(event) => onComplete(event.payload)
	);

	const unlistenError = await listen<ModelDownloadErrorEvent>(
		'spatial-model-download-error',
		(event) => onError(event.payload)
	);

	return () => {
		unlistenProgress();
		unlistenComplete();
		unlistenError();
	};
}
