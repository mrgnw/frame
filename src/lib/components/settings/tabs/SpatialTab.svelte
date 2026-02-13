<script lang="ts">
	import { onMount } from 'svelte';
	import type { SpatialConfig } from '$lib/types';
	import Label from '$lib/components/ui/Label.svelte';
	import Slider from '$lib/components/ui/Slider.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import { IconCheck, IconArrowDown } from '$lib/icons';
	import {
		checkSpatialModels,
		downloadSpatialModel,
		setupModelDownloadListeners
	} from '$lib/services/spatial';

	const ENCODER_SIZES = [
		{ id: 's', label: 'Small', desc: 'Fastest, good quality' },
		{ id: 'm', label: 'Medium', desc: 'Balanced speed/quality' },
		{ id: 'l', label: 'Large', desc: 'Best quality, slowest' }
	] as const;

	let {
		config,
		disabled = false,
		onUpdate
	}: {
		config: SpatialConfig;
		disabled?: boolean;
		onUpdate: (config: Partial<SpatialConfig>) => void;
	} = $props();

	let modelStatus = $state<Record<string, boolean>>({ s: false, m: false, l: false });
	let downloading = $state<Record<string, number | null>>({ s: null, m: null, l: null });

	onMount(() => {
		checkSpatialModels()
			.then((status) => {
				modelStatus = status;
			})
			.catch(() => {});

		let cleanup: (() => void) | undefined;
		setupModelDownloadListeners(
			(payload) => {
				downloading = { ...downloading, [payload.encoderSize]: payload.progress };
			},
			(payload) => {
				downloading = { ...downloading, [payload.encoderSize]: null };
				modelStatus = { ...modelStatus, [payload.encoderSize]: true };
			},
			(payload) => {
				downloading = { ...downloading, [payload.encoderSize]: null };
				console.error('Model download failed:', payload.error);
			}
		).then((unlisten) => {
			cleanup = unlisten;
		});

		return () => cleanup?.();
	});

	function handleModelClick(sizeId: string) {
		if (modelStatus[sizeId]) {
			onUpdate({ encoderSize: sizeId as 's' | 'm' | 'l' });
		} else if (downloading[sizeId] === null) {
			onUpdate({ encoderSize: sizeId as 's' | 'm' | 'l' });
			downloadSpatialModel(sizeId).catch(() => {});
		}
	}
</script>

<div class="space-y-3">
	<div class="flex items-start gap-2">
		<Checkbox
			id="spatial-enabled"
			checked={config.enabled}
			onchange={() => onUpdate({ enabled: !config.enabled })}
			{disabled}
		/>
		<div class="space-y-0.5">
			<Label for="spatial-enabled">Enable spatial encoding</Label>
			<p class="text-[9px] text-gray-alpha-600">
				Convert to Apple Vision Pro spatial video after conversion
			</p>
		</div>
	</div>

	<div
		class="space-y-3 transition-opacity"
		class:pointer-events-none={!config.enabled}
		class:opacity-40={!config.enabled}
	>
		<div class="space-y-2">
			<Label variant="section">Depth Model</Label>
			<div class="grid grid-cols-3 gap-1.5">
				{#each ENCODER_SIZES as size (size.id)}
					{@const available = modelStatus[size.id]}
					{@const progress = downloading[size.id]}
					{@const isActive = config.encoderSize === size.id}
					<button
						class="relative flex flex-col items-center gap-0.5 overflow-hidden rounded-md border px-2 py-1.5 text-[11px] transition-colors
							{isActive
							? 'border-blue-600 bg-blue-900/20 text-blue-400'
							: 'border-gray-alpha-200 bg-gray-alpha-50 text-gray-alpha-700 hover:border-gray-alpha-300 hover:bg-gray-alpha-100'}"
						disabled={disabled || (progress !== null && progress !== undefined)}
						onclick={() => handleModelClick(size.id)}
					>
						{#if progress !== null && progress !== undefined}
							<div
								class="absolute inset-x-0 bottom-0 h-0.5 bg-blue-600 transition-all"
								style="width: {progress}%"
							></div>
						{/if}
						<div class="flex items-center gap-1">
							<span class="font-medium">{size.label}</span>
							{#if available}
								<IconCheck size={10} class="text-green-500" />
							{:else if progress !== null && progress !== undefined}
								<span class="text-[8px] text-blue-400">{Math.round(progress)}%</span>
							{:else}
								<IconArrowDown size={10} class="opacity-40" />
							{/if}
						</div>
						<span class="text-[8px] opacity-50">{size.desc}</span>
					</button>
				{/each}
			</div>
		</div>

		<div class="space-y-1.5">
			<div class="flex items-end justify-between">
				<Label for="max-disparity">3D Intensity</Label>
				<div
					class="rounded border border-blue-600 bg-blue-900/20 px-1.5 text-[10px] font-medium text-blue-600"
				>
					{config.maxDisparity}px
				</div>
			</div>
			<div class="py-1">
				<Slider
					id="max-disparity"
					min={10}
					max={80}
					step={5}
					value={config.maxDisparity}
					oninput={(e) => onUpdate({ maxDisparity: parseInt(e.currentTarget.value) })}
					disabled={disabled || !config.enabled}
				/>
			</div>
			<div class="flex justify-between text-[8px] text-gray-alpha-600">
				<span>Subtle</span>
				<span>Extreme</span>
			</div>
		</div>
	</div>
</div>
