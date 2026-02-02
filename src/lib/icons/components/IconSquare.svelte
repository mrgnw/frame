<script lang="ts">
	import type { SVGAttributes } from 'svelte/elements';
	let {
		size = 24,
		class: className,
		...props
	}: { size?: number | string; class?: string } & SVGAttributes<SVGElement> = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	width={size}
	height={size}
	fill="currentColor"
	viewBox="0 0 256 256"
	class={className}
	{...props}
>
	<path
		d="M208,32H48A16,16,0,0,0,32,48V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V48A16,16,0,0,0,208,32Zm0,176H48V48H208V208Z"
	></path>
</svg>
