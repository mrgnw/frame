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
		d="M200,40a8,8,0,0,0-8,8V64H64V48a8,8,0,0,0-16,0V64A32,32,0,0,0,16,96v24a32,32,0,0,0,32,32h40a32,32,0,0,0,31.81-28.56A8,8,0,0,1,127.73,118h.54a8,8,0,0,1,7.92,5.44A32,32,0,0,0,168,152h40a32,32,0,0,0,32-32V96A32,32,0,0,0,208,64V48A8,8,0,0,0,200,40ZM88,136H48a16,16,0,0,1-16-16V96A16,16,0,0,1,48,80H96a16,16,0,0,1,16,16v24A16,16,0,0,1,96,136Zm136-16a16,16,0,0,1-16,16H168a16,16,0,0,1-16-16V96a16,16,0,0,1,16-16h48a16,16,0,0,1,16,16Z"
	></path>
</svg>
