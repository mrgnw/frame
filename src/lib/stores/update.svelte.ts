import { type Update } from '@tauri-apps/plugin-updater';

const initialState = {
	isAvailable: false,
	version: '',
	body: '',
	updateObject: null as Update | null,
	showDialog: false,
	isChecking: false,
	isInstalling: false,
	progress: 0,
	error: null as string | null
};

export const updateStore = $state({ ...initialState });

export function resetUpdateStore() {
	Object.assign(updateStore, initialState);
}
