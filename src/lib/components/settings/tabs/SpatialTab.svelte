<script lang="ts">
	import type { SpatialConfig } from '$lib/types';
	import Button from '$lib/components/ui/Button.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Slider from '$lib/components/ui/Slider.svelte';
	import Checkbox from '$lib/components/ui/Checkbox.svelte';

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
</script>

<div class="space-y-4">
	<div class="space-y-3">
		<Label variant="section">Depth Model</Label>
		<div class="grid grid-cols-3 gap-2">
			{#each ENCODER_SIZES as size (size.id)}
				<Button
					variant={config.encoderSize === size.id ? 'selected' : 'outline'}
					onclick={() => onUpdate({ encoderSize: size.id })}
					{disabled}
					class="w-full"
				>
					<div class="flex flex-col items-center gap-0.5">
						<span>{size.label}</span>
						<span class="text-[9px] opacity-50">{size.desc}</span>
					</div>
				</Button>
			{/each}
		</div>
	</div>

	<div class="space-y-2 pt-2">
		<div class="flex items-end justify-between">
			<Label for="max-disparity">3D Intensity</Label>
			<div
				class="rounded border border-blue-600 bg-blue-900/20 px-1.5 text-[10px] font-medium text-blue-600"
			>
				{config.maxDisparity}px
			</div>
		</div>
		<div class="py-2">
			<Slider
				id="max-disparity"
				min={10}
				max={80}
				step={5}
				value={config.maxDisparity}
				oninput={(e) => onUpdate({ maxDisparity: parseInt(e.currentTarget.value) })}
				{disabled}
			/>
		</div>
		<div class="flex justify-between text-[9px] text-gray-alpha-600">
			<span>Subtle</span>
			<span>Extreme</span>
		</div>
	</div>

	<div class="space-y-3 pt-2">
		<Label variant="section">Options</Label>
		<div class="space-y-2">
			<div class="flex items-start gap-2">
				<Checkbox
					id="skip-downscale"
					checked={config.skipDownscale}
					onchange={() => onUpdate({ skipDownscale: !config.skipDownscale })}
					{disabled}
				/>
				<div class="space-y-0.5">
					<Label for="skip-downscale">Skip downscale</Label>
					<p class="text-[9px] text-gray-alpha-600">
						Keep original resolution instead of downscaling to 1080p@24fps
					</p>
				</div>
			</div>
		</div>
	</div>

	<div class="mt-4 rounded-md border border-gray-alpha-100 bg-gray-alpha-50 p-3">
		<p class="text-[10px] text-gray-alpha-600">
			Converts 2D video to Apple Vision Pro spatial video using AI depth estimation. Requires
			macOS with <code class="text-[9px]">spatial</code> CLI
			(<code class="text-[9px]">brew install spatial</code>) and
			<code class="text-[9px]">spatial-maker</code>
			(<code class="text-[9px]">uv tool install spatial-maker</code>).
		</p>
	</div>
</div>
