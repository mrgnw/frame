import { type Update } from '@tauri-apps/plugin-updater';

export class UpdateStore {
	isAvailable = $state(false);
	version = $state('');
	body = $state('');
	updateObject = $state<Update | null>(null);
	showDialog = $state(false);
	isChecking = $state(false);
	isInstalling = $state(false);
	progress = $state(0);
	error = $state<string | null>(null);

	reset() {
		this.isAvailable = false;
		this.version = '';
		this.body = '';
		this.updateObject = null;
		this.showDialog = false;
		this.isChecking = false;
		this.isInstalling = false;
		this.progress = 0;
		this.error = null;
	}
}

export const updateStore = new UpdateStore();
