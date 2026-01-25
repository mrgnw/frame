<script lang="ts">
	import { fly, fade } from 'svelte/transition';
	import { X } from 'lucide-svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { checkForAppUpdate } from '$lib/services/update';
	import { updateStore } from '$lib/stores/update.svelte';

	let {
		maxConcurrency,
		onUpdate,
		onClose
	}: {
		maxConcurrency: number;
		onUpdate: (value: number) => void | Promise<void>;
		onClose: () => void;
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

<button
	class="absolute inset-0 z-60 cursor-default bg-background/60 backdrop-blur-sm"
	transition:fade={{ duration: 300 }}
	onclick={onClose}
	aria-label="Close settings"
></button>

<div
	class="border-gray-alpha-200 absolute top-0 right-0 bottom-0 z-70 w-80 rounded-l-xl border-l bg-background/60 shadow-xl backdrop-blur-md"
	transition:fly={{ x: 320, duration: 300, opacity: 1 }}
>
	<div class="flex items-center justify-between border-b border-gray-alpha-100 px-4 py-3">
		<h2 class="text-[10px] font-medium tracking-widest text-foreground uppercase">Settings</h2>
		<button onclick={onClose} class="text-gray-alpha-600 transition-colors hover:text-foreground">
			<X size={16} />
		</button>
	</div>

	<div class="space-y-6 p-4">
		<div class="space-y-2">
			<Label for="max-concurrency" variant="section">Max Concurrency</Label>
			<div class="flex items-center gap-2">
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
						disabled={isSaving}
					/>
				</div>
				<Button
					onclick={handleSave}
					disabled={isSaving || localValue.current === String(maxConcurrency)}
					variant="outline"
				>
					{isSaving ? 'Saving...' : 'Apply'}
				</Button>
			</div>
		</div>

		<div class="space-y-2">
			<Label variant="section">App Updates</Label>
			<div class="flex flex-col gap-2">
				<Button
					variant="outline"
					class="w-full justify-start"
					onclick={handleCheckUpdate}
					disabled={isCheckingForUpdate}
				>
					{isCheckingForUpdate ? 'Checking...' : 'Check for Updates'}
				</Button>
				{#if checkStatus}
					<span class="text-[10px] text-ds-blue-600">{checkStatus}</span>
				{/if}
			</div>
		</div>
	</div>
</div>
