<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import TransactionCostsTab from '$lib/components/settings/TransactionCostsTab.svelte';
	import DangerAction from '$lib/components/shared/DangerAction.svelte';
	import * as Card from '$lib/components/ui/card';
	import { toast } from 'svelte-sonner';
	import { clearAllData, LogLevel, Theme } from '$lib/api';
	import KeymapRow from '$lib/components/keymap/KeymapRow.svelte';
	import { getKeymapManager, getAllKeymapActionsStore } from '$lib/keymap';
	import { Button } from '$lib/components/ui/button';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Item from '$lib/components/ui/item/index.js';
	import { settings } from '$lib/stores/settings';
	import { ArrowRight, MoonIcon, ScrollText, SunIcon, TestTubeDiagonal } from 'lucide-svelte';

	const keymapManager = getKeymapManager();
	const allKeymapActions = getAllKeymapActionsStore();

	let logLevels: LogLevel[] = [LogLevel.Debug, LogLevel.Info, LogLevel.Warn, LogLevel.Error];
	let themes: Theme[] = [Theme.Light, Theme.Dark];

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
			<Tabs.Trigger value="general">General</Tabs.Trigger>
			<Tabs.Trigger value="transaction-costs">Transaction Costs</Tabs.Trigger>
			<Tabs.Trigger value="keymaps">Keymaps</Tabs.Trigger>
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
					{#each Array.from($allKeymapActions.values()) as action (action.id)}
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
					<Item.Root variant="outline" class="border-green-500 bg-green-500/10">
						<Item.Media>
							<TestTubeDiagonal />
						</Item.Media>
						<Item.Content>
							<Item.Title>Self Diagnostics</Item.Title>
							<Item.Description>
								Perform self diagnostics to verify integrity of the app, database and daraja APIs.
							</Item.Description>
						</Item.Content>
						<Item.Actions>
							<Button variant="outline" size="sm" href="/self_test">Open <ArrowRight /></Button>
						</Item.Actions>
						<Item.Actions />
					</Item.Root>
					<!-- log level -->
					<Item.Root variant="outline">
						<Item.Media>
							<ScrollText />
						</Item.Media>
						<Item.Content>
							<Item.Title>Log level</Item.Title>
							<Item.Description>The level of logging by the app backend.</Item.Description>
						</Item.Content>
						<Item.Actions>
							<Select.Root type="single" bind:value={$settings.server_log_level}>
								<Select.Trigger>
									Level - {$settings.server_log_level}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each logLevels as level}
											<Select.Item value={level}>{level}</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
						</Item.Actions>
						<Item.Actions />
					</Item.Root>
					<!-- theme -->
					<Item.Root variant="outline">
						<Item.Media>
							<SunIcon
								class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90"
							/>
							<MoonIcon
								class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0"
							/>
						</Item.Media>
						<Item.Content>
							<Item.Title>Theme</Item.Title>
							<Item.Description>The theme to use for the application.</Item.Description>
						</Item.Content>
						<Item.Actions>
							<Select.Root type="single" bind:value={$settings.theme}>
								<Select.Trigger>
									Theme - {$settings.theme}
								</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each themes as theme}
											<Select.Item value={theme}>{theme}</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
						</Item.Actions>
						<Item.Actions />
					</Item.Root>
					<hr class="my-4" />
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
