import { invoke } from '@tauri-apps/api/core';

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
	const result = await invoke<string[]>('open_native_file_dialog', { options });

	if (!result || result.length === 0) {
		return null;
	}

	if (options.multiple) {
		return result;
	}

	return result[0];
}
