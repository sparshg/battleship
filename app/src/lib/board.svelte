<script lang="ts">
	import { Board } from '$lib/state.svelte';
	import { Crosshair, Ship } from 'lucide-svelte';

	let { board, callback }: { board: Board; callback: (i: number, j: number) => void } = $props();
</script>

<div class="grid grid-cols-10 gap-1 bg-primary p-2 rounded-lg">
	{#each board.board as row, i}
		{#each row as cell, j}
			<button
				class="aspect-square bg-blue-950 flex items-center justify-center {!board.isOpponent
					? 'cursor-default'
					: ''}"
				onclick={() => callback(i, j)}
			>
				{#if cell === 's'}
					<Ship class="size-3/5 text-blue-500" />
				{:else if cell === 'h'}
					<Crosshair class="size-3/5 text-red-500" />
				{:else if cell === 'm'}
					<div class="size-3/5 bg-gray-300 rounded-full"></div>
				{/if}
			</button>
		{/each}
	{/each}
</div>
