import { invoke } from '@tauri-apps/api/core';
import { dialogStore } from '$lib/stores/dialog.svelte';

export interface NativeDialogFilter {
	name: string;
	extensions: string[];
}

export interface NativeFileDialogOptions {
	title?: string;
	filters?: NativeDialogFilter[];
	multiple?: boolean;
	directory?: boolean;
	defaultPath?: string;
	recursive?: boolean;
}

export async function openNativeFileDialog(
	options: NativeFileDialogOptions = {}
): Promise<string | string[] | null> {
	dialogStore.isActive = true;

	await new Promise((resolve) => setTimeout(resolve, 32));

	try {
		const result = await invoke<string[]>('open_native_file_dialog', { options });

		if (!result || result.length === 0) {
			return null;
		}

		if (options.multiple) {
			return result;
		}

		return result[0];
	} finally {
		dialogStore.isActive = false;
	}
}

export interface NativeAskDialogOptions {
	title?: string;
	message: string;
	kind?: 'info' | 'warning' | 'error' | 'question';
	okLabel?: string;
	cancelLabel?: string;
}

export async function askNativeDialog(options: NativeAskDialogOptions): Promise<boolean> {
	dialogStore.isActive = true;

	await new Promise((resolve) => setTimeout(resolve, 32));

	try {
		return await invoke<boolean>('ask_native_dialog', { options });
	} finally {
		dialogStore.isActive = false;
	}
}
