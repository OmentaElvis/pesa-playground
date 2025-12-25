<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { PlusCircle, Trash2, LoaderCircle, CopyIcon } from 'lucide-svelte';
	import { createOperator, deleteOperator, type BusinessOperator } from '$lib/api';
	import { Input } from '../ui/input';
	import { toast } from 'svelte-sonner';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import * as Field from '$lib/components/ui/field/index.js';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { copyToClipboard } from '$lib/utils';

	let {
		operators,
		businessId,
		onrefresh
	}: {
		operators: BusinessOperator[];
		businessId: number;
		onrefresh?: () => Promise<void>;
	} = $props();

	let isCreating = $state(false);
	let username = $state('');
	let password = $state('');
	let loading = $state(false);
	let showDeleteDialog = $state(false);
	let operatorToDelete: BusinessOperator | null = $state(null);

	function create() {
		isCreating = true;
	}

	function handleCancel() {
		isCreating = false;
		username = '';
		password = '';
	}

	async function handleCreateOperator() {
		try {
			loading = true;
			await createOperator(businessId, username, password);
			toast.success('Operator created');
			if (onrefresh) await onrefresh();
			handleCancel();
		} catch (err) {
			toast.error(`Failed to create operator: ${err}`);
		} finally {
			loading = false;
		}
	}

	function confirmDelete(operator: BusinessOperator) {
		operatorToDelete = operator;
		showDeleteDialog = true;
	}

	async function handleDeleteOperator() {
		if (!operatorToDelete) return;
		try {
			await deleteOperator(operatorToDelete.id);
			toast.success('Operator deleted');
			if (onrefresh) await onrefresh();
		} catch (err) {
			toast.error(`Failed to delete operator: ${err}`);
		} finally {
			showDeleteDialog = false;
			operatorToDelete = null;
		}
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title class="flex items-center">
			<div class="flex-1">Operators</div>
			<Button onclick={create} class="cursor-pointer" disabled={isCreating}>
				<PlusCircle /> Add
			</Button>
		</Card.Title>
		<Card.Description>Business operators who can initiate transactions.</Card.Description>
	</Card.Header>
	<Card.Content>
		<Table.Root>
			<Table.Caption>A list of available operators</Table.Caption>
			<Table.Header>
				<Table.Row>
					<Table.Head class="w-[100px] font-bold">Username</Table.Head>
					<Table.Head class="font-bold">Password</Table.Head>
					<Table.Head></Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each operators as operator}
					<Table.Row>
						<Table.Cell class="font-medium">{operator.username}</Table.Cell>
						<Table.Cell>
							<InputGroup.Root>
								<InputGroup.Input value={operator.password} readonly />
								<InputGroup.Addon align="inline-end">
									<Button
										variant="ghost"
										class="cursor-pointer"
										onclick={() => copyToClipboard(operator.password)}
									>
										<CopyIcon />
									</Button>
								</InputGroup.Addon>
							</InputGroup.Root>
						</Table.Cell>
						<Table.Cell>
							<Button
								onclick={() => confirmDelete(operator)}
								class="cursor-pointer"
								variant="ghost"
								size="icon"
							>
								<Trash2 class="text-destructive" />
							</Button>
						</Table.Cell>
					</Table.Row>
				{:else}
					<Table.Cell colspan={3} class="text-center">No operators found.</Table.Cell>
				{/each}
			</Table.Body>
		</Table.Root>
	</Card.Content>
</Card.Root>

<AlertDialog.Root bind:open={showDeleteDialog}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
			<AlertDialog.Description>
				This action cannot be undone. This will permanently delete the operator.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action onclick={handleDeleteOperator}>Continue</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<AlertDialog.Root bind:open={isCreating}>
	<form class="p-4">
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title>Create operator</AlertDialog.Title>
				<AlertDialog.Description>
					Add an operator. This is used by most business to outgoing accounts transactions.
				</AlertDialog.Description>
			</AlertDialog.Header>
			<div>
				<div class="grid gap-4">
					<Field.Group>
						<Field.Field>
							<Field.Label for="operator-username">Username</Field.Label>
							<Input
								id="operator-username"
								bind:value={username}
								required
								onkeydown={(e) => {
									if (e.key === 'Escape') {
										handleCancel();
									}
								}}
							/>
						</Field.Field>
						<Field.Field>
							<Field.Label for="operator-password">Password</Field.Label>
							<Input
								id="operator-password"
								type="password"
								bind:value={password}
								required
								onkeydown={(e) => {
									if (e.key === 'Escape') {
										handleCancel();
									}
								}}
							/>
						</Field.Field>
					</Field.Group>
				</div>
			</div>
			<AlertDialog.Footer>
				<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
				<AlertDialog.Action
					type="submit"
					disabled={password == '' || username == ''}
					onclick={handleCreateOperator}
					class="flex items-center"
				>
					{#if loading}
						<LoaderCircle class="animate-spin" /> Creating
					{:else}
						Create
					{/if}
				</AlertDialog.Action>
			</AlertDialog.Footer>
		</AlertDialog.Content>
	</form>
</AlertDialog.Root>
