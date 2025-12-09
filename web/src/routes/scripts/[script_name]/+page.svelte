<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { scriptsRead, scriptsSave, scriptsExecute, scriptsDelete } from '$lib/scripts';
	import { Button } from '$lib/components/ui/button';
	import { toast } from 'svelte-sonner';
	import { Save, Play, Trash2 } from 'lucide-svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { lua } from '@codemirror/legacy-modes/mode/lua';
	import { mode } from 'mode-watcher';
	import { basicSetup } from 'codemirror';
	import { EditorView } from '@codemirror/view';
	import { Compartment, EditorState } from '@codemirror/state';
	import { StreamLanguage } from '@codemirror/language';
	import { vscodeDarkInit, vscodeLightInit } from '@uiw/codemirror-theme-vscode';
	import transport from '$lib/lsp-transport';
	import { LSPClient, languageServerExtensions } from '@codemirror/lsp-client';

	let client = new LSPClient({ extensions: languageServerExtensions() }).connect(transport);

	let darkTheme = vscodeDarkInit({
		settings: {
			fontFamily: 'JetBrainsMonoNL-Medium',
			fontSize: '14px'
		}
	});

	let lightTheme = vscodeLightInit({
		settings: {
			fontFamily: 'JetBrainsMonoNL-Medium',
			fontSize: '14px'
		}
	});

	let scriptName: string = $derived(page.params.script_name || '');
	let scriptContent = $state('');
	let scriptOutput = $state('');
	let isLoading = $state(true);
	let activeTab = $state('editor');
	let target: Element | undefined = $state();
	let editor: EditorView | undefined = $state(undefined);
	const themeCompartment = new Compartment();
	let height: number = $state(0);
	let editorHeight = $derived(height - 80);

	$effect(() => {
		if (target) {
			editor = new EditorView({
				parent: target,
				state: EditorState.create({
					extensions: [
						basicSetup,
						client.plugin(`file://${scriptName}]`),
						StreamLanguage.define(lua),
						themeCompartment.of(mode.current === 'light' ? lightTheme : darkTheme)
					],
					doc: scriptContent
				})
			});
		}
	});

	function setTheme(isDark: boolean) {
		editor?.dispatch({
			effects: themeCompartment.reconfigure(isDark ? darkTheme : lightTheme)
		});
	}

	$effect(() => {
		if (editor && mode) {
			setTheme(mode.current === 'dark');
		}
	});

	onMount(async () => {
		try {
			scriptContent = await scriptsRead(scriptName);
		} catch (e) {
			// This is likely a new file, so we start with empty content.
			// The user can then save it.
			scriptContent = `-- New script: ${scriptName}\n\n`;
			toast.info('Creating a new script', {
				description: `Save the file to create ${scriptName}.`
			});
		} finally {
			isLoading = false;
		}
	});

	async function handleSave() {
		try {
			await scriptsSave(scriptName, scriptContent);
			toast.success(`Script '${scriptName}' saved.`);
		} catch (e) {
			toast.error(`Failed to save script: ${scriptName}`, {
				description: e as string
			});
		}
	}

	async function handleExecute() {
		try {
			scriptOutput = await scriptsExecute(scriptContent);
			toast.success('Script executed successfully.');
		} catch (e) {
			scriptOutput = e as string;
			toast.error('Script execution failed', { description: e as string });
		} finally {
			// Switch to the output tab to show the result
			activeTab = 'output';
		}
	}

	async function handleDelete() {
		if (confirm(`Are you sure you want to delete ${scriptName}?`)) {
			try {
				await scriptsDelete(scriptName);
				toast.success(`Script '${scriptName}' deleted.`);
				// Redirect to the main scripts page after deletion
				window.location.href = '/scripts';
			} catch (e) {
				toast.error(`Failed to delete script: ${scriptName}`, {
					description: e as string
				});
			}
		}
	}

	$effect(() => {
		if (!target) return;
		// @ts-ignore
		let ed: HTMLDivElement = target.getElementsByClassName('cm-editor')[0];
		ed.style.height = `${editorHeight}px`;
	});
</script>

<div class="flex h-full flex-col gap-4 p-4" bind:clientHeight={height}>
	{#if isLoading}
		<div class="flex flex-grow items-center justify-center">
			<p>Loading script...</p>
		</div>
	{:else}
		<Tabs.Root bind:value={activeTab} class="flex flex-grow flex-col">
			<div class="flex items-center justify-between border-b pb-4">
				<Tabs.List class="">
					<Tabs.Trigger value="editor">Editor</Tabs.Trigger>
					<Tabs.Trigger value="output">Output</Tabs.Trigger>
				</Tabs.List>
				<div class="flex-1"></div>
				<div class="flex items-center gap-2">
					<Button onclick={handleSave} variant="secondary" size="sm">
						<Save class="mr-2 h-4 w-4" />
						Save
					</Button>
					<Button onclick={handleExecute} size="sm">
						<Play class="mr-2 h-4 w-4" />
						Run
					</Button>
					<Button onclick={handleDelete} variant="destructive" size="sm">
						<Trash2 class="mr-2 h-4 w-4" />
						Delete
					</Button>
				</div>
			</div>
			<Tabs.Content value="editor">
				<div style={`height: ${editorHeight}px;`}>
					<div bind:this={target} class="editor-wrapper"></div>
				</div>
			</Tabs.Content>
			<Tabs.Content value="output" class="flex-grow pt-4">
				<div class="h-full overflow-y-auto rounded-md bg-muted p-4">
					<pre class="font-mono text-sm whitespace-pre-wrap">{scriptOutput ||
							'Script output will appear here.'}</pre>
				</div>
			</Tabs.Content>
		</Tabs.Root>
	{/if}
</div>

<style>
	:global(.cm-scroller) {
		overflow: auto;
	}
	:global(.cm-editor) {
		overflow: auto;
	}
</style>
