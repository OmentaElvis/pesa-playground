<script lang="ts">
	import * as Kbd from '$lib/components/ui/kbd/index.js';
	import { Button } from '$lib/components/ui/button';
	import { Pencil, Undo2 } from 'lucide-svelte';
	import { getKeymapManager, eventToShortcutString, type KeymapAction } from '$lib/keymap';
	import { onMount, onDestroy } from 'svelte';
	import { toast } from 'svelte-sonner';

	let { action }: { action: KeymapAction } = $props();

	let isEditing = $state(false);
	let currentShortcut = $derived(action.shortcut);
	let pendingShortcut: string | null = $state(null);
	let keymapManager = getKeymapManager();

	const keys = $derived(currentShortcut.split('+'));

	function startEditing() {
		isEditing = true;
		pendingShortcut = null;
		window.addEventListener('keydown', captureShortcut, true); // Use capture phase
	}

	function stopEditing() {
		isEditing = false;
		window.removeEventListener('keydown', captureShortcut, true);
	}

	function captureShortcut(event: KeyboardEvent) {
		// Prevent default browser behavior for shortcuts like Alt+Left
		event.preventDefault();
		event.stopPropagation();

		const shortcutString = eventToShortcutString(event);
		if (shortcutString) {
			pendingShortcut = shortcutString;
			// Stop capturing after a valid shortcut is captured
			window.removeEventListener('keydown', captureShortcut, true);
		}
	}

	function saveShortcut() {
		if (pendingShortcut && pendingShortcut !== currentShortcut) {
			const success = keymapManager.updateKeybinding(action.id, pendingShortcut);
			if (success) {
				currentShortcut = pendingShortcut;
				action.shortcut = pendingShortcut; // Update the action object directly
				toast.success(`Keybinding for "${action.name}" updated to "${pendingShortcut}"`);
			} else {
				toast.error(
					`Failed to update keybinding for "${action.name}". It might be conflicting with another shortcut.`
				);
			}
		}
		stopEditing();
	}

	function resetShortcut() {
		keymapManager.resetKeybinding(action.id);
		currentShortcut = action.defaultShortcut;
		action.shortcut = action.defaultShortcut; // Update the action object directly
		toast.info(`Keybinding for "${action.name}" reset to default.`);
		stopEditing();
	}

	// Make sure to clean up event listener if component is destroyed while in editing mode
	onDestroy(() => {
		window.removeEventListener('keydown', captureShortcut, true);
	});
</script>

<div class="grid grid-cols-3 items-center gap-2">
	<p class="text-sm text-muted-foreground">{action.name}</p>
	<div class="col-span-1 text-left">
		<Kbd.Root>
			{#if isEditing}
				{#if pendingShortcut}
					{#each pendingShortcut.split('+') as key, i}
						<span class="uppercase">{key}</span>
						{#if i < pendingShortcut.split('+').length - 1}
							<span class="mx-1">+</span>
						{/if}
					{/each}
				{:else}
					<span class="text-muted-foreground">Press a key combination...</span>
				{/if}
			{:else}
				{#each keys as key, i}
					<span class="uppercase">{key}</span>
					{#if i < keys.length - 1}
						<span class="mx-1">+</span>
					{/if}
				{/each}
			{/if}
		</Kbd.Root>
	</div>
	<div class="flex justify-end gap-2">
		{#if isEditing}
			<Button size="sm" variant="outline" onclick={stopEditing}>Cancel</Button>
			<Button size="sm" onclick={saveShortcut} disabled={!pendingShortcut}>Save</Button>
		{:else}
			<Button size="sm" variant="ghost" onclick={startEditing}><Pencil class="h-4 w-4" /></Button>
			<Button
				size="sm"
				variant="ghost"
				onclick={resetShortcut}
				disabled={currentShortcut === action.defaultShortcut}
			>
				<Undo2 class="h-4 w-4" />
			</Button>
		{/if}
	</div>
</div>
