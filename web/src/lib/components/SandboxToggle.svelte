<script lang="ts">
	import { Play, Cog, Loader2, CircleX } from 'lucide-svelte';
	import { sandboxes, type SandboxStatus } from '$lib/stores/sandboxStatus';
	import { startSandbox, stopSandbox } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';
	import * as Kbd from '$lib/components/ui/kbd';
	import { toast } from 'svelte-sonner';
	import { spin } from '$lib/transitions/spin';

	let {
		id,
		port = $bindable(),
		host = $bindable()
	}: { id: number; port: number; host: string } = $props();
	let status: SandboxStatus = $state('off');
	let error: string | null = $state(null);

	async function toggle() {
		if (status === 'starting') {
			return;
		}

		try {
			if (status === 'on') {
				await stopSandbox(id);
			} else if (status === 'off') {
				await startSandbox(id);
			} else if (status === 'error') {
				await startSandbox(id);
			}
		} catch (e: any) {
			toast.error(`API Error: ${e.message ?? String(e)}`);
		}
	}

	onMount(() => {
		port = 8000 + id;
		host = '127.0.0.1';
	});

	const unsub = sandboxes.subscribe((map) => {
		const info = map.get(id);
		if (!info) {
			status = 'off';
			error = null;
		} else {
			status = info.status;
			error = info.error || null;
			port = info.port;
			host = info.host;
		}
	});

	onDestroy(() => {
		unsub();
	});
</script>

<div class="flex items-center gap-4">
	<button
		class="relative h-6 w-12 cursor-pointer rounded-full transition-colors duration-300"
		class:bg-green-700={status === 'on'}
		class:bg-gray-400={status === 'off'}
		class:bg-yellow-400={status === 'starting'}
		class:bg-red-400={status === 'error'}
		onclick={toggle}
	>
		<div
			class="absolute top-0 flex h-6 w-6 items-center justify-center rounded-full bg-white shadow-md transition-all duration-300"
			class:translate-x-8={status === 'on'}
		>
			{#if status === 'starting'}
				<Loader2 class="h-4 w-4 animate-spin text-yellow-600" />
			{:else if status === 'on'}
				<div in:spin={{ degrees: 360, duration: 700 }}><Cog class="h-4 w-4 text-gray-700" /></div>
			{:else if status === 'error'}
				<CircleX class="w-4 text-red-500" />
			{:else}
				<Play class="h-4 w-4 text-gray-500" />
			{/if}
		</div>
	</button>
	{#if status === 'on'}
		<div class="text-xs font-bold text-green-700">Running on port {port}</div>
	{:else if status === 'starting'}
		<div class="text-xs text-yellow-700">Starting...</div>
	{:else if status === 'error' || error}
		<div class="text-xs text-red-700">Error: {error}</div>
	{/if}
	<Kbd.Root>Ctrl+Shift+{id}</Kbd.Root>
</div>
