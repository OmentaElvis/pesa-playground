<script lang="ts">
	import { Play, Cog, Loader2, CircleX } from 'lucide-svelte';
	import { getSandboxes, sandboxes, type SandboxStatus } from '$lib/stores/sandboxStatus';
	import { startSandbox, stopSandbox } from '$lib/api';
	import { onDestroy } from 'svelte';
	import * as Kbd from '$lib/components/ui/kbd';

	export let id: number;
	let status: SandboxStatus = 'off';
	let port: number | null = null;
	let error: string | null = null;

	function setStatus(s: SandboxStatus) {
		status = s;
	}

	async function toggle() {
		if (status === 'starting') return;

		if (status === 'on') {
			try {
				await stopSandbox(id);
				setStatus('off');
				await getSandboxes();
				port = null;
				error = null;
			} catch (e) {
				setStatus('error');
				error = e instanceof Error ? e.message : String(e);
			}
		} else {
			setStatus('starting');
			error = null;
			try {
				const addr = await startSandbox(id);
				port = parseInt(addr.split(':').pop() || '0');
			} catch (e) {
				setStatus('error');
				error = e instanceof Error ? e.message : String(e);
			}
		}
	}

	const unsub = sandboxes.subscribe((map)=> {
	  let info = map.get(id);
	  if (!info) {
	  	setStatus("off");
	  } else {
	  	setStatus(info.status);
	  	if (info.status == "error") {
	  		error = info.error || null;
	  	}

	  	if (info.status == "on" ) {
	  		port = info.port;
	  	}
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
		on:click={toggle}
	>
		<div
			class="absolute top-0 flex h-6 w-6 items-center justify-center rounded-full bg-white shadow-md transition-all duration-300"
			class:translate-x-8={status === 'on'}
		>
			{#if status === 'starting'}
				<Loader2 class="h-4 w-4 animate-spin text-yellow-600" />
			{:else if status === 'on'}
				<Cog class="h-4 w-4 animate-spin text-gray-700" />
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
