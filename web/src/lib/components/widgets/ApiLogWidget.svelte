<script lang="ts">
	import { apiLogStore } from '$lib/stores/apiLogStore';
	import type { ApiLog } from '$lib/api';
	import LogSheet from '$lib/components/LogSheet.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Badge } from '$lib/components/ui/badge';
	import { slide } from 'svelte/transition';

	// Initialize the store. This will only run the fetch logic once.
	apiLogStore.init();

	// Subscribe to the logs from the store.
	const logs = apiLogStore;

	let selectedLog = $state<ApiLog | null>(null);
	let isSheetOpen = $state(false);

	function handleLogClick(log: ApiLog) {
		selectedLog = log;
		isSheetOpen = true;
	}

	const getStatusColor = (status: number) => {
		if (status >= 200 && status < 300)
			return 'bg-green-200/50 text-green-800 dark:bg-green-800 dark:text-white dark:font-bold';
		if (status >= 400 && status < 500)
			return 'bg-orange-200/50 text-orange-600 dark:bg-orange-800 dark:text-white dark:font-bold';
		if (status >= 500)
			return 'bg-red-200/50 text-red-500 dark:bg-red-800 dark:font-bold dark:text-white';
		return 'bg-gray-500 hover:bg-gray-600';
	};
</script>

<div class="flex h-[calc(100vh-150px)] flex-col">
	<ScrollArea class="flex-1">
		<div class="space-y-2 p-2">
			{#each $logs as log (log.id)}
				<button
					class="w-full cursor-pointer rounded-md p-2 text-left text-sm transition-colors duration-150 hover:bg-muted"
					onclick={() => handleLogClick(log)}
					transition:slide|local={{ duration: 300 }}
				>
					<div class="flex items-center gap-2">
						<span class="pl-2 text-xs font-semibold"><pre>{log.method}</pre></span>
						<span class="text-xs break-all text-muted-foreground">{log.path}</span>
					</div>
					<div class="space-between flex w-full items-center justify-center gap-2">
						<Badge class="{getStatusColor(log.status_code)} font-mono text-xs">
							{log.status_code}
						</Badge>
						<a href={`/projects/${log.project_id}`} class="text-xs text-blue-500 hover:underline">
							#{log.project_id}
						</a>
						<div class="mt-1 flex-1 text-right font-mono text-xs text-muted-foreground">
							{new Date(log.created_at).toLocaleTimeString()}
						</div>
					</div>
				</button>
			{:else}
				<div class="text-center text-muted-foreground p-8 text-sm">No API logs recorded yet.</div>
			{/each}
		</div>
	</ScrollArea>
</div>

{#if selectedLog}
	<LogSheet bind:open={isSheetOpen} log={selectedLog} />
{/if}
