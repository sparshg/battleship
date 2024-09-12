<script lang="ts">
	import Board from '$lib/board.svelte';
	import Header from '$lib/header.svelte';
	import { State } from '$lib/state.svelte';

	let gameState = new State();
</script>

<div class="min-h-screen bg-gray-100 py-8 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<Header />

		<main class="bg-white shadow-xl rounded-lg overflow-hidden">
			<div class="p-6 space-y-6">
				<div class="flex justify-between items-center">
					<h2 class="text-2xl font-semibold text-gray-700">
						{gameState.phase === 'placement' ? 'Place Your Ships' : 'Battle Phase'}
					</h2>
					<div class="flex space-x-4">
						<div class="text-blue-600">Your Ships: {5}</div>
						<div class="text-red-600">Enemy Ships: {5}</div>
					</div>
				</div>

				<div class="grid md:grid-cols-2 gap-8">
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-2">Your Board</h3>
						<Board isOpponent={false} />
					</div>
					<div>
						<h3 class="text-lg font-medium text-gray-700 mb-2">Opponent's Board</h3>
						<Board isOpponent={true} />
					</div>
				</div>

				<div class="flex justify-center space-x-4">
					{#if gameState.phase === 'placement'}
						<button class="btn btn-primary">Rotate Ship</button>
					{:else}
						<button class="btn btn-primary">Fire!</button>
					{/if}
					<button class="btn btn-outline">Reset Game</button>
				</div>
			</div>
		</main>

		<footer class="mt-8 text-center text-gray-500">
			<p>&copy; 2024 Battleship Online. All rights reserved.</p>
		</footer>
	</div>
</div>
