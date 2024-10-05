<script lang="ts">
	import Board from '$lib/board.svelte';
	import Header from '$lib/header.svelte';
	import Join from '$lib/join.svelte';
	import { State } from '$lib/state.svelte';
	import { Users } from 'lucide-svelte';

	let gameState = new State();

	function leaveRoom() {
		gameState.socket.emit('leave');
		gameState = new State();
	}
</script>

<div class="min-h-screen bg-base-300 py-8 px-4 sm:px-6 lg:px-8">
	<div class="max-w-7xl mx-auto">
		<Header />

		<main class="bg-base-100 shadow-xl rounded-xl overflow-hidden">
			<div class="p-6 space-y-6">
				<div class="flex justify-between items-center">
					<h2 class="text-2xl font-semibold rounded-full bg-base-300 py-3 px-6">
						{gameState.hasNotStarted()
							? 'Place your ships'
							: gameState.turn >= 0
								? 'Make a guess'
								: 'Waiting for opponent'}
					</h2>
					{#if gameState.room}
						<div class="flex flex-row h-full items-center space-x-4">
							<button
								class="rounded-full bg-base-300 px-4 uppercase font-mono font-bold tracking-wide text-xl py-2.5 tooltip tooltip-bottom"
								data-tip="Copy"
								onclick={() => navigator.clipboard.writeText(gameState.room)}
							>
								{gameState.room}
							</button>
							<div class="rounded-full bg-base-300 px-4 flex items-center space-x-2 py-3">
								<div
									class="size-3 bg-green-500 rounded-full shadow-[0_0_10px] shadow-green-500"
								></div>
								<div class="font-mono font-bold">{gameState.users}</div>
								<Users />
							</div>
							<button class="btn btn-error text-xl" onclick={leaveRoom}>Leave</button>
						</div>
					{/if}
				</div>

				<div class="grid md:grid-cols-2 gap-8">
					<div>
						<h3 class="text-lg font-medium mb-2">Your Board</h3>

						<Board
							class={gameState.turn < 0 ? 'scale-[1.01]' : 'opacity-60'}
							board={gameState.playerBoard}
							callback={() => {}}
						/>
					</div>
					<div>
						<h3 class="text-lg font-medium mb-2">Opponent's Board</h3>
						<div class="relative">
							<Board
								class={gameState.turn >= 0 ? 'scale-[1.01]' : 'opacity-60'}
								board={gameState.opponentBoard}
								callback={(i, j) => gameState.attack(i, j)}
							/>

							{#if gameState.game_over}
								<div
									class="absolute inset-0 flex items-center justify-center bg-black bg-opacity-50 pointer-events-none"
								>
									<div class="p-6 bg-base-300 rounded-xl text-center">
										<h3 class="text-2xl font-semibold">Game Over</h3>
										<p class="text-lg">
											{gameState.turn >= 0 ? 'You win!' : 'You lose!'}
										</p>
										<button
											class="btn btn-primary mt-4 pointer-events-auto"
											onclick={() => {
												let room = gameState.room;
												leaveRoom();
												gameState.joinRoom(room);
											}}
										>
											Play Again
										</button>
										<button class="btn btn-secondary mt-4 ml-4 pointer-events-auto" onclick={leaveRoom}>Leave</button>
									</div>
								</div>
							{/if}
							{#if gameState.hasNotStarted()}
								<Join
									class="absolute top-[24px] left-[15px] w-[calc(100%-15px)] h-[calc(100%-24px)]"
									roomCode={gameState.room}
									createRoom={() => gameState.createRoom()}
									joinRoom={(code) => gameState.joinRoom(code)}
									{leaveRoom}
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
