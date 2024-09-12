<script lang="ts">
	import Cell from '$lib/cell.svelte';
	import type { Board } from '$lib/state.svelte';

	const callback = (i: number, j: number) => {
		if (!isOpponent) return;
		console.log(`Cell clicked at row ${i}, column ${j}`);
		board[i][j] = 'hit';
	};
	let board: Board = $state(Array(10).fill(Array(10).fill('empty')));

	let { isOpponent } = $props<{ isOpponent: boolean }>();
</script>

<div class="grid grid-cols-10 gap-1 bg-blue-100 p-2 rounded-lg">
	{#each board as row, i}
		{#each row as cell, j}
			<Cell {cell} {isOpponent} callback={() => callback(i, j)} />
		{/each}
	{/each}
</div>
