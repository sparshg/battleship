<script lang="ts">
	import { ClipboardCopy } from 'lucide-svelte';

	let joinCode = $state('');

	let {
		class: className = '',
		roomCode,
		createRoom,
		joinRoom,
		leaveRoom
	}: {
		roomCode: string;
		createRoom: () => void;
		joinRoom: (code: string) => void;
		leaveRoom: () => void;
		class: string;
	} = $props();
</script>

<div
	class="{className} rounded-lg glass bg-primary flex flex-col items-center justify-center font-mono"
>
	<div class="space-y-4 max-w-[70%]">
		{#if roomCode}
		<div class="text-center text-lg text-primary-content">Share this room code</div>
			<div class="space-x-2 flex flex-row justify-center items-center">
				<div
					class="text-3xl font-bold tracking-widest text-secondary-content font-mono bg-secondary py-3 rounded-full px-12"
				>
					{roomCode}
				</div>

				<button
					class="btn btn-secondary btn-circle size-16"
					onclick={() => navigator.clipboard.writeText(roomCode)}
				>
					<ClipboardCopy />
				</button>
			</div>
		{:else}
			<button onclick={() => createRoom()} class="w-full btn btn-neutral text-xl">
				Create Room
			</button>
		{/if}
		<div class="text-center text-lg text-primary-content">OR</div>
		{#if !roomCode}
			<div class="space-y-2">
				<input
					type="text"
					placeholder="Enter code"
					maxlength="4"
					bind:value={joinCode}
					class="input input-bordered input-primary uppercase tracking-widest placeholder-primary text-neutral text-center font-bold text-xl lg:text-3xl w-full glass"
				/>
				<button
					onclick={() => joinRoom(joinCode)}
					class="w-full btn btn-outline btn-neutral text-neutral hover:border-neutral hover:bg-transparent text-xl"
				>
					Join Room
				</button>
			</div>
		{:else}
			<div class="space-x-2 flex flex-row justify-center items-center">
				<button class="btn btn-error text-2xl px-12 py-3 h-fit" onclick={leaveRoom}>
					Leave room
				</button>
			</div>
		{/if}
	</div>
</div>
