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
		d="M200,32H160a16,16,0,0,0-16,16V208a16,16,0,0,0,16,16h40a16,16,0,0,0,16-16V48A16,16,0,0,0,200,32Zm0,176H160V48h40ZM96,32H56A16,16,0,0,0,40,48V208a16,16,0,0,0,16,16H96a16,16,0,0,0,16-16V48A16,16,0,0,0,96,32Zm0,176H56V48H96Z"
	></path>
</svg>
