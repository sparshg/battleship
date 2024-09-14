<script lang="ts">
	import Board from '$lib/board.svelte';
	import Header from '$lib/header.svelte';
	import { State } from '$lib/state.svelte';

	let gameState = new State();
</script>

<div class="min-h-screen bg-base-300 py-8 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<Header />

		<main class="bg-base-100 shadow-xl rounded-xl overflow-hidden">
			<div class="p-6 space-y-6">
				<div class="flex justify-between items-center">
					<h2 class="text-2xl font-semibold">
						{gameState.phase === 'placement' ? 'Place Your Ships' : 'Battle Phase'}
					</h2>
					<div class="flex space-x-4">
						<div class="text-blue-600">Your Ships: {5}</div>
						<div class="text-red-600">Enemy Ships: {5}</div>
					</div>
				</div>

				<div class="grid md:grid-cols-2 gap-8">
					<div>
						<h3 class="text-lg font-medium mb-2">Your Board</h3>
						<Board board={gameState.playerBoard} callback={() => {}} />
					</div>
					<div>
						<h3 class="text-lg font-medium mb-2">Opponent's Board</h3>
						<Board
							board={gameState.opponentBoard}
							callback={(i, j) => gameState.opponentBoard.set(i, j, 'h')}
						/>
					</div>
				</div>

				<div class="flex justify-center space-x-4">
					{#if gameState.phase === 'placement'}
						<button class="btn btn-primary" onclick={() => gameState.playerBoard.randomize()}
							>Randomize</button
						>
					{:else}
						<button class="btn btn-primary">Fire!</button>
					{/if}
					<button class="btn btn-outline" onclick={() => gameState.createRoom()}>Create Room</button
					>
					<input
						type="text"
						bind:value={gameState.room}
						placeholder="Code"
						class="input input-bordered w-full max-w-20"
					/>
					<button class="btn btn-outline">Join Room</button>
				</div>
			</div>
		</main>

		<footer class="mt-8 text-center text-gray-500">
			<p>&copy; 2024 Battleship Online. All rights reserved.</p>
		</footer>
	</div>
</div>
