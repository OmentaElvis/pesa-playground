<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/stores';
	import { scriptsList } from '$lib/scripts';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import { File, Plus } from 'lucide-svelte';
	import { goto } from '$app/navigation';
	import { Pane, PaneGroup } from 'paneforge';
	import { lspStart, lspStop } from '$lib/api';

	let scripts: string[] = $state([]);
	let newScriptName = $state('');
	let { children } = $props();

	onMount(async () => {
		await lspStart();
		await loadScripts();
	});

	onDestroy(async () => {
		// await lspStop();
	});

	async function loadScripts() {
		try {
			scripts = await scriptsList();
		} catch (e) {
			toast.error('Failed to load scripts', { description: e as string });
		}
	}

	function handleNewScript() {
		if (!newScriptName) {
			toast.error('Script name cannot be empty.');
			return;
		}
		// Normalize the name for the URL
		const normalizedName = newScriptName.trim().toLowerCase().replace(/\s+/g, '-');
		goto(`/scripts/${normalizedName}`);
		newScriptName = '';
	}
</script>

<PaneGroup direction="horizontal" class="h-full w-full">
	<Pane id="script-list" defaultSize={20} minSize={15} class="h-full">
		<div class="flex h-full flex-col gap-2 bg-muted/40 p-4">
			<h2 class="mb-2 text-lg font-semibold">Scripts</h2>
			<div class="flex-grow overflow-y-auto">
				<nav class="flex flex-col gap-1">
					{#each scripts as script}
						<a
							href="/scripts/{script}"
							class="flex items-center gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-all hover:text-primary"
							class:bg-muted={$page.params.script_name === script}
							class:text-primary={$page.params.script_name === script}
						>
							<File class="h-4 w-4" />
							{script}
						</a>
					{/each}
				</nav>
			</div>
			<div class="mt-auto flex flex-col gap-2">
				<Input
					bind:value={newScriptName}
					placeholder="New Script Name..."
					onkeydown={(e) => e.key === 'Enter' && handleNewScript()}
				/>
				<Button onclick={handleNewScript} size="sm">
					<Plus class="mr-2 h-4 w-4" />
					New Script
				</Button>
			</div>
		</div>
	</Pane>
	<Pane id="script-content" class="h-full">
		{@render children()}
	</Pane>
</PaneGroup>
