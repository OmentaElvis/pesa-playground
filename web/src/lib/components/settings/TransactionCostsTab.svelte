<script lang="ts">
	import { onMount } from 'svelte';
	import { PlusCircle, Edit, Trash2 } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		type TransactionCost,
		type TransactionCostData,
		createTransactionCost,
		deleteTransactionCost,
		listTransactionCosts,
		updateTransactionCost
	} from '$lib/api';

	let costs: (TransactionCost | { type: 'header'; name: string })[] = [];
	let showDialog = false;
	let showDeleteAlert = false;
	let selectedCost: TransactionCost | null = null;
	let costData: TransactionCostData = {
		transaction_type: '',
		min_amount: 0,
		max_amount: 0,
		fee_fixed: undefined,
		fee_percentage: undefined
	};

	async function loadCosts() {
		const fetchedCosts = await listTransactionCosts();
		// Group costs by transaction_type
		const grouped: { [key: string]: TransactionCost[] } = {};
		fetchedCosts.forEach((cost) => {
			if (!grouped[cost.transaction_type]) {
				grouped[cost.transaction_type] = [];
			}
			grouped[cost.transaction_type].push(cost);
		});

		// Create a flat array with grouping headers
		const processedCosts: (TransactionCost | { type: 'header'; name: string })[] = [];
		Object.keys(grouped)
			.sort()
			.forEach((type) => {
				processedCosts.push({ type: 'header', name: type });
				grouped[type]
					.sort((a, b) => a.min_amount - b.min_amount)
					.forEach((cost) => {
						processedCosts.push(cost);
					});
			});
		costs = processedCosts; // Assign the processed array to the reactive state
	}

	onMount(loadCosts);

	function openNewDialog() {
		selectedCost = null;
		costData = {
			transaction_type: '',
			min_amount: 0,
			max_amount: 0,
			fee_fixed: undefined,
			fee_percentage: undefined
		};
		showDialog = true;
	}

	function openEditDialog(cost: TransactionCost) {
		selectedCost = cost;
		costData = { ...cost };
		showDialog = true;
	}

	function openDeleteAlert(cost: TransactionCost) {
		selectedCost = cost;
		showDeleteAlert = true;
	}

	async function handleSave() {
		if (selectedCost) {
			await updateTransactionCost(selectedCost.id, costData);
		} else {
			await createTransactionCost(costData);
		}
		showDialog = false;
		await loadCosts();
	}

	async function handleDelete() {
		if (selectedCost) {
			await deleteTransactionCost(selectedCost.id);
			showDeleteAlert = false;
			await loadCosts();
		}
	}
</script>

<div class="p-4">
	<div class="mb-4 flex items-center justify-between">
		<h1 class="text-2xl font-bold">Transaction Cost Rules</h1>
		<Button onclick={openNewDialog}>
			<PlusCircle class="mr-2" />
			Add New Rule
		</Button>
	</div>

	<Table.Root>
		<Table.Header>
			<Table.Row>
				<Table.Head>Type</Table.Head>
				<Table.Head>Min Amount (KES)</Table.Head>
				<Table.Head>Max Amount (KES)</Table.Head>
				<Table.Head>Fixed Fee (KES)</Table.Head>
				<Table.Head>Percentage Fee (%)</Table.Head>
				<Table.Head>Actions</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each costs as item}
				{#if 'type' in item && item.type === 'header'}
					<Table.Row class=" font-semibold">
						<Table.Cell colspan={6}>{item.name}</Table.Cell>
					</Table.Row>
				{:else}
					<Table.Row>
						<Table.Cell></Table.Cell>
						<!-- Empty cell for alignment under the type heading -->
						<Table.Cell>{(item as TransactionCost).min_amount}</Table.Cell>
						<Table.Cell>{(item as TransactionCost).max_amount}</Table.Cell>
						<Table.Cell>{(item as TransactionCost).fee_fixed ?? 'N/A'}</Table.Cell>
						<Table.Cell>{(item as TransactionCost).fee_percentage ?? 'N/A'}</Table.Cell>
						<Table.Cell>
							<Button
								variant="ghost"
								size="icon"
								onclick={() => openEditDialog(item as TransactionCost)}
							>
								<Edit />
							</Button>
							<Button
								variant="ghost"
								size="icon"
								onclick={() => openDeleteAlert(item as TransactionCost)}
							>
								<Trash2 />
							</Button>
						</Table.Cell>
					</Table.Row>
				{/if}
			{/each}
		</Table.Body>
	</Table.Root>
</div>

<Dialog.Root bind:open={showDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>{selectedCost ? 'Edit' : 'New'} Transaction Cost Rule</Dialog.Title>
		</Dialog.Header>
		<div class="grid gap-4 py-4">
			<div class="grid grid-cols-4 items-center gap-4">
				<Label for="type" class="text-right">Type</Label>
				<Input id="type" bind:value={costData.transaction_type} class="col-span-3" />
			</div>
			<div class="grid grid-cols-4 items-center gap-4">
				<Label for="min-amount" class="text-right">Min Amount (KES)</Label>
				<Input id="min-amount" type="number" bind:value={costData.min_amount} class="col-span-3" />
			</div>
			<div class="grid grid-cols-4 items-center gap-4">
				<Label for="max-amount" class="text-right">Max Amount (KES)</Label>
				<Input id="max-amount" type="number" bind:value={costData.max_amount} class="col-span-3" />
			</div>
			<div class="grid grid-cols-4 items-center gap-4">
				<Label for="fee-fixed" class="text-right">Fixed Fee (KES)</Label>
				<Input id="fee-fixed" type="number" bind:value={costData.fee_fixed} class="col-span-3" />
			</div>
			<div class="grid grid-cols-4 items-center gap-4">
				<Label for="fee-percentage" class="text-right">Percentage Fee (%)</Label>
				<Input
					id="fee-percentage"
					type="number"
					bind:value={costData.fee_percentage}
					class="col-span-3"
				/>
			</div>
		</div>
		<Dialog.Footer>
			<Button onclick={handleSave}>Save</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={showDeleteAlert}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Are you sure?</AlertDialog.Title>
			<AlertDialog.Description>
				This action cannot be undone. This will permanently delete the transaction cost rule.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action onclick={handleDelete}>Delete</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
