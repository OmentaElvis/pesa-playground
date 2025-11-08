<script lang="ts">
	import { Play, Cog, Loader2, CircleX } from 'lucide-svelte';
	import { getSandboxes, sandboxStatus } from '$lib/stores/sandboxStatus';
	import { sandboxStatus as apiSandboxStatus, startSandbox, stopSandbox } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';

	export let id: number;
	let status: 'off' | 'starting' | 'on' | 'error' = 'off';
	let port: number | null = null;
	let error: string | null = null;
	let pollInterval: ReturnType<typeof setInterval>;

	function setStatus(s: 'off' | 'starting' | 'on' | 'error') {
		status = s;
		$sandboxStatus = s;
	}

	async function refresh() {
		try {
			const res = await apiSandboxStatus(id);
			if (res.status === 'on') {
				setStatus('on');
				port = res.port;
				error = null;
			} else if (res.status == 'error') {
				error = res.error || 'The sandbox encountered unknown error';
				setStatus('error');
				port = null;
			} else {
				setStatus('off');
				port = null;
			}
		} catch (e) {
			setStatus('error');
			error = e instanceof Error ? e.message : String(e);
		}
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
				setTimeout(() => refresh(), 2000);
				await getSandboxes();
				// status = "on";
			} catch (e) {
				setStatus('error');
				error = e instanceof Error ? e.message : String(e);
			}
		}
	}

	onMount(() => {
		refresh();
		pollInterval = setInterval(() => {
			refresh();
		}, 10000);
	});

	onDestroy(() => {
		clearInterval(pollInterval);
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
</div>
