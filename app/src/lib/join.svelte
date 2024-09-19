<script lang="ts">
	import { ClipboardCopy } from 'lucide-svelte';

	let joinCode = $state('');

	let {
		class: className = '',
		roomCode,
		createRoom,
		joinRoom
	}: {
		roomCode: string;
		createRoom: () => void;
		joinRoom: (code: string) => void;
		class: string;
	} = $props();
</script>

<div
	class="{className} rounded-lg glass bg-primary flex flex-col items-center justify-center font-mono"
>
	<div class="space-y-4 max-w-[70%]">
		{#if roomCode}
			<div class="space-x-2 flex flex-row justify-center items-center">
				<div
					class="text-3xl font-bold tracking-widest text-accent-content font-mono bg-accent py-3 rounded-full px-12"
				>
					{roomCode}
				</div>

				<button
					class="btn btn-accent btn-circle size-16"
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
				class="w-full btn btn-outline btn-neutral text-neutral hover:text-neutral hover:bg-transparent text-xl"
			>
				Join Room
			</button>
		</div>
	</div>
</div>
