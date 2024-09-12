<script lang="ts">
	import { Ship, Crosshair } from 'lucide-svelte';

	let { cell, isOpponent, callback } = $props<{
		cell: string;
		isOpponent: boolean;
		callback: () => void;
	}>();
</script>

<button class="aspect-square bg-blue-200 flex items-center justify-center" onclick={callback}>
	{#if cell === 'ship' && !isOpponent}
		<Ship class="size-3/5 text-blue-500" />
	{:else if cell === 'hit'}
		<Crosshair class="size-3/5 text-red-500" />
	{:else if cell === 'miss'}
		<div class="size-3/5 bg-gray-300 rounded-full"></div>
	{/if}
</button>
