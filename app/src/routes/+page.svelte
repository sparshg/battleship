<script lang="ts">
	import Board from '$lib/board.svelte';
	import Header from '$lib/header.svelte';
	import Join from '$lib/join.svelte';
	import { State } from '$lib/state.svelte';

	const hostname = window.location.hostname;
	let gameState = new State(hostname);
</script>

<div class="min-h-screen bg-base-300 py-8 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<Header />

		<main class="bg-base-100 shadow-xl rounded-xl overflow-hidden">
			<div class="p-6 space-y-6">
				<div class="flex justify-between items-center">
					<h2 class="text-2xl font-semibold rounded-full bg-base-300 py-3 px-6">
						{gameState.hasNotStarted() ? 'Place your ships' : 'Battle Phase'}
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
						<div class="relative">
							<Board board={gameState.opponentBoard} callback={(i, j) => gameState.attack(i, j)} />
							{#if gameState.hasNotStarted()}
								<Join
									class="absolute top-[24px] left-[15px] w-[calc(100%-15px)] h-[calc(100%-24px)]"
									roomCode={gameState.room}
									createRoom={() => gameState.createRoom()}
									joinRoom={(code) => gameState.joinRoom(code)}
								/>
							{/if}
						</div>
					</div>
				</div>
			</div>
		</main>

		<footer class="mt-8 text-center text-gray-500">
			<p>&copy; 2024 Battleship Online. All rights reserved.</p>
		</footer>
	</div>
</div>
