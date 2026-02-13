import {
	setupSpatialListeners,
	startSpatial as startSpatialService,
	cancelSpatial
} from '$lib/services/spatial';
import { FileStatus, type FileItem, type SpatialConfig, DEFAULT_SPATIAL_CONFIG } from '$lib/types';

export interface SpatialCallbacks {
	onFilesUpdate: (updater: (files: FileItem[]) => FileItem[]) => void;
	onLogsUpdate: (updater: (logs: Record<string, string[]>) => Record<string, string[]>) => void;
	getFiles: () => FileItem[];
	getIsProcessing: () => boolean;
	setIsProcessing: (value: boolean) => void;
}

export function createSpatialQueue(callbacks: SpatialCallbacks) {
	let spatialConfig = $state<SpatialConfig>({ ...DEFAULT_SPATIAL_CONFIG });

	function setupListeners() {
		const unlistenPromise = setupSpatialListeners(
			(payload) => {
				callbacks.onFilesUpdate((files) =>
					files.map((f) => {
						if (f.id === payload.id) {
							const status =
								f.status === FileStatus.QUEUED ? FileStatus.CONVERTING : f.status;
							return { ...f, status, progress: payload.progress };
						}
						return f;
					})
				);
			},
			(payload) => {
				callbacks.onFilesUpdate((files) =>
					files.map((f) =>
						f.id === payload.id ? { ...f, status: FileStatus.COMPLETED, progress: 100 } : f
					)
				);
				checkAllDone();
			},
			(payload) => {
				callbacks.onFilesUpdate((files) =>
					files.map((f) =>
						f.id === payload.id
							? { ...f, status: FileStatus.ERROR, conversionError: payload.error }
							: f
					)
				);
				checkAllDone();
			},
			(payload) => {
				callbacks.onLogsUpdate((logs) => {
					const current = logs[payload.id] || [];
					return { ...logs, [payload.id]: [...current, payload.line] };
				});
			},
			(payload) => {
				callbacks.onFilesUpdate((files) =>
					files.map((f) => {
						if (f.id === payload.id && f.status === FileStatus.QUEUED) {
							return { ...f, status: FileStatus.CONVERTING, progress: 0 };
						}
						return f;
					})
				);
			}
		);

		return () => {
			unlistenPromise?.then((unlisten) => unlisten());
		};
	}

	function checkAllDone() {
		const files = callbacks.getFiles();
		const allDone = files.every(
			(f) =>
				f.status === FileStatus.COMPLETED ||
				f.status === FileStatus.ERROR ||
				f.status === FileStatus.IDLE
		);

		if (allDone && callbacks.getIsProcessing()) {
			callbacks.setIsProcessing(false);
		}
	}

	async function startSpatialConversion() {
		const files = callbacks.getFiles();
		const pendingFiles = files.filter(
			(f) =>
				f.isSelectedForConversion &&
				f.status !== FileStatus.CONVERTING &&
				f.status !== FileStatus.QUEUED &&
				f.status !== FileStatus.COMPLETED
		);

		if (pendingFiles.length === 0) return;

		callbacks.setIsProcessing(true);

		const pendingIds = pendingFiles.map((f) => f.id);

		callbacks.onLogsUpdate((logs) => {
			const newLogs = { ...logs };
			pendingFiles.forEach((f) => {
				if (!newLogs[f.id]) {
					newLogs[f.id] = [];
				}
			});
			return newLogs;
		});

		callbacks.onFilesUpdate((files) =>
			files.map((f) => {
				const isPending = pendingIds.includes(f.id);
				return isPending ? { ...f, status: FileStatus.QUEUED, progress: 0 } : f;
			})
		);

		const enqueueErrors: Record<string, string> = {};

		for (const file of pendingFiles) {
			try {
				await startSpatialService(file.id, file.path, spatialConfig);
			} catch (error) {
				const message = error instanceof Error ? error.message : String(error);
				enqueueErrors[file.id] = message;
			}
		}

		const errorIds = Object.keys(enqueueErrors);
		if (errorIds.length > 0) {
			callbacks.onFilesUpdate((files) =>
				files.map((f) =>
					errorIds.includes(f.id)
						? {
								...f,
								status: FileStatus.ERROR,
								progress: 0,
								conversionError: enqueueErrors[f.id]
							}
						: f
				)
			);
			callbacks.onLogsUpdate((logs) => {
				const next = { ...logs };
				errorIds.forEach((id) => {
					const current = next[id] || [];
					next[id] = [
						...current,
						`[ERROR] Failed to queue spatial conversion: ${enqueueErrors[id]}`
					];
				});
				return next;
			});
			checkAllDone();
		}
	}

	async function queueSpatialForFile(id: string, filePath: string) {
		callbacks.onFilesUpdate((files) =>
			files.map((f) =>
				f.id === id ? { ...f, status: FileStatus.QUEUED, progress: 0 } : f
			)
		);
		callbacks.onLogsUpdate((logs) => {
			const current = logs[id] || [];
			return { ...logs, [id]: [...current, '[SPATIAL] Queuing spatial conversion...'] };
		});

		try {
			await startSpatialService(id, filePath, spatialConfig);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			callbacks.onFilesUpdate((files) =>
				files.map((f) =>
					f.id === id
						? { ...f, status: FileStatus.ERROR, conversionError: message }
						: f
				)
			);
			callbacks.onLogsUpdate((logs) => {
				const current = logs[id] || [];
				return {
					...logs,
					[id]: [...current, `[ERROR] Failed to queue spatial conversion: ${message}`]
				};
			});
			checkAllDone();
		}
	}

	async function cancelTask(id: string) {
		try {
			await cancelSpatial(id);
		} catch (e) {
			console.error('Failed to cancel spatial task', e);
		}
	}

	function updateConfig(updates: Partial<SpatialConfig>) {
		spatialConfig = { ...spatialConfig, ...updates };
	}

	return {
		get config() {
			return spatialConfig;
		},
		setupListeners,
		startSpatialConversion,
		queueSpatialForFile,
		cancelTask,
		checkAllDone,
		updateConfig
	};
}
