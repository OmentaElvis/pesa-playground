<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import TransactionCostsTab from '$lib/components/settings/TransactionCostsTab.svelte';
	import DangerAction from '$lib/components/shared/DangerAction.svelte';
	import * as Card from '$lib/components/ui/card';
	import { toast } from 'svelte-sonner';
	import { clearAllData } from '$lib/api';
	import KeymapRow from '$lib/components/keymap/KeymapRow.svelte';
	import { getKeymapManager, getAllKeymapActions } from '$lib/keymap';
	import { Button } from '$lib/components/ui/button';

	const keymapManager = getKeymapManager();
	const allKeymapActions = getAllKeymapActions();

	async function handlePurge() {
		try {
			await clearAllData();
			toast.success('All data has been purged. The application will now reload.');
			setTimeout(() => {
				window.location.reload();
			}, 1500);
		} catch (error: any) {
			toast.error(`Failed to purge data: ${error}`);
		}
	}

	function handleResetAllKeybindings() {
		keymapManager.resetAllKeybindings();
		toast.info('All keybindings have been reset to default.');
	}
</script>

<div class="p-4">
	<h1 class="mb-4 text-2xl font-bold">Settings</h1>

	<Tabs.Root value="general" class="w-full">
		<Tabs.List>
			<Tabs.Trigger value="transaction-costs">Transaction Costs</Tabs.Trigger>
			<Tabs.Trigger value="keymaps">Keymaps</Tabs.Trigger>
			<Tabs.Trigger value="general">General</Tabs.Trigger>
			<!-- Add other settings tabs here -->
		</Tabs.List>
		<Tabs.Content value="keymaps">
			<Card.Root>
				<Card.Header>
					<Card.Title>Keymap Reference</Card.Title>
					<Card.Description>
						Customize your keyboard shortcuts. Click a shortcut to start editing.
					</Card.Description>
				</Card.Header>
				<Card.Content class="grid max-w-2xl gap-4">
					{#each Array.from(allKeymapActions.values()) as action (action.id)}
						<KeymapRow {action} />
					{/each}
					<div class="mt-4">
						<Button variant="outline" onclick={handleResetAllKeybindings}>
							Reset All Keybindings to Default
						</Button>
					</div>
				</Card.Content>
			</Card.Root>
		</Tabs.Content>
		<Tabs.Content value="general">
			<Card.Root>
				<Card.Header>
					<Card.Title>General Settings</Card.Title>
					<Card.Description>General application settings.</Card.Description>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div>
						<h3 class="mb-2 text-lg font-semibold">Danger Zone</h3>
						<DangerAction
							title="Purge All Data"
							description="This will delete all businesses, projects, users, and transaction history. The application will reload in a clean state."
							buttonLabel="Purge and Restart"
							dialogTitle="Are you absolutely sure?"
							dialogDescription="This action is irreversible. All your data will be permanently deleted."
							onConfirm={handlePurge}
						/>
					</div>
				</Card.Content>
			</Card.Root>
		</Tabs.Content>
		<Tabs.Content value="transaction-costs">
			<TransactionCostsTab />
		</Tabs.Content>
	</Tabs.Root>
</div>
