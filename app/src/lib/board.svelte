<script lang="ts">
	import { Board } from '$lib/state.svelte';
	import { Crosshair, Ship } from 'lucide-svelte';

	let { board, callback }: { board: Board; callback: (i: number, j: number) => void } = $props();
</script>

<div class="grid grid-cols-10 ml-4">
	{#each [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] as i}
		<div class="text-center">{i}</div>
	{/each}
</div>

<div class="flex flex-row">
	<div class="grid grid-rows-10 items-center mr-1">
		{#each ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'] as i}
			<div class="text">{i}</div>
		{/each}
	</div>
	<div
		class="grid grid-cols-10 gap-0.5 lg:gap-1 bg-primary-content p-1 lg:p-1.5 rounded-lg size-full"
	>
		{#each board.board as row, i}
			{#each row as cell, j}
				<button
					class="aspect-square {cell === 'm'
						? 'bg-secondary'
						: cell === 'h'
							? 'bg-accent'
							: 'bg-primary'} flex items-center justify-center {!board.isOpponent
						? 'cursor-default'
						: ''}"
					onclick={() => callback(i, j)}
				>
					{#if cell === 's'}
						<Ship class="size-3/5 text-primary-content" />
					{:else if cell === 'h'}
						<Crosshair class="size-3/5 text-accent-content" />
					{/if}
				</button>
			{/each}
		{/each}
	</div>
</div>
