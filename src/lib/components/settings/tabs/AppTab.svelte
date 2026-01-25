<script lang="ts">
	import Input from '$lib/components/ui/Input.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { checkForAppUpdate } from '$lib/services/update';
	import { updateStore } from '$lib/stores/update.svelte';

	let {
		maxConcurrency,
		disabled = false,
		onUpdate
	}: {
		maxConcurrency: number;
		disabled?: boolean;
		onUpdate: (value: number) => void | Promise<void>;
	} = $props();

	let localValue = $derived.by(() => {
		let value = $state(String(maxConcurrency));
		return {
			get current() {
				return value;
			},
			set current(v) {
				value = v;
			}
		};
	});

	let isSaving = $state(false);
	let isCheckingForUpdate = $state(false);
	let checkStatus = $state('');

	async function handleSave() {
		const parsed = Number(localValue.current);
		isSaving = true;
		try {
			await onUpdate(parsed);
		} finally {
			isSaving = false;
		}
	}

	async function handleCheckUpdate() {
		isCheckingForUpdate = true;
		checkStatus = '';
		try {
			const result = await checkForAppUpdate();
			if (result.available) {
				updateStore.isAvailable = true;
				updateStore.version = result.version || '';
				updateStore.body = result.body || '';
				updateStore.updateObject = result.updateObject;
				updateStore.showDialog = true;
				checkStatus = 'Update available!';
			} else {
				checkStatus = 'You are on the latest version.';
			}
		} catch (e) {
			checkStatus = 'Error checking for updates.';
			console.error(e);
		} finally {
			isCheckingForUpdate = false;
			setTimeout(() => {
				checkStatus = '';
			}, 3000);
		}
	}
</script>

<div class="space-y-4">
	<div class="space-y-2">
		<Label for="max-concurrency" variant="section">Max Concurrency</Label>
		<div class="flex items-end gap-2">
			<div class="flex-1">
				<Input
					id="max-concurrency"
					type="text"
					inputmode="numeric"
					value={localValue.current}
					oninput={(e) => {
						const sanitized = e.currentTarget.value.replace(/[^0-9]/g, '');
						if (sanitized !== e.currentTarget.value) {
							e.currentTarget.value = sanitized;
						}
						localValue.current = sanitized;
					}}
					placeholder="2"
					disabled={disabled || isSaving}
				/>
			</div>
			<Button
				onclick={handleSave}
				disabled={disabled || isSaving || localValue.current === String(maxConcurrency)}
				variant="outline"
				class="h-7.5"
			>
				{isSaving ? 'Saving...' : 'Apply'}
			</Button>
		</div>
	</div>

	<div class="space-y-2">
		<Label variant="section">App Updates</Label>
		<div class="flex items-center gap-3">
			<Button
				variant="outline"
				class="h-7.5 w-32"
				onclick={handleCheckUpdate}
				disabled={isCheckingForUpdate || disabled}
			>
				{isCheckingForUpdate ? 'Checking...' : 'Check for Updates'}
			</Button>
			{#if checkStatus}
				<span class="text-gray-alpha-600 text-[10px]">{checkStatus}</span>
			{/if}
		</div>
	</div>
</div>
