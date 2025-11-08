<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		C2BResponseType,
		createTillAccount,
		updateTillAccount,
		type CreateTillAccountData,
		type TillAccountDetails,
		type UpdateTillAccountData
	} from '$lib/api';
	import { PlusCircle, Save, LoaderCircle } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';
	import C2BParametersForm from './C2BParametersForm.svelte';

	let { tillAccounts, businessId, isCreating, editingId, refresh, create, edit, cancel } = $props<{
		tillAccounts: TillAccountDetails[];
		businessId: number;
		isCreating: boolean;
		editingId: string | null;
		refresh: () => void,
		create: () => void,
		edit: (params: {id: number}) => void,
		cancel: () => void,
	}>();

	let saving = $state(false);
	let formData: (CreateTillAccountData & { account_id?: number }) | null = $state(null);

	$effect(() => {
		if (isCreating) {
			formData = {
				business_id: businessId,
				initial_balance: 0,
				store_number: 0,
				till_number: 0,
				location_description: '',
				response_type: C2BResponseType.Completed,
				validation_url: '',
				confirmation_url: ''
			};
		} else if (editingId) {
			const account = tillAccounts.find((acc: TillAccountDetails) => acc.account_id === parseInt(editingId!));
			if (account) {
				formData = { ...account, initial_balance: 0 };
			}
		} else {
			formData = null;
		}
	});

	async function handleSubmit() {
		if (!formData) return;

		saving = true;
		try {
			if (editingId) {
				const data: UpdateTillAccountData = {
					till_number: formData.till_number,
					store_number: formData.store_number,
					location_description: formData.location_description || undefined,
					response_type: formData.response_type,
					confirmation_url: formData.confirmation_url || undefined,
					validation_url: formData.validation_url || undefined
				};
				await updateTillAccount(parseInt(editingId), data);
				toast.success('Till account updated successfully.');
			} else {
				await createTillAccount({
					...formData,
					business_id: businessId
				});
				toast.success('Till account created successfully.');
			}
			refresh();
		} catch (err) {
			toast.error(`Failed: ${err}`);
		} finally {
			saving = false;
		}
	}
</script>

<Card>
	<CardHeader class="flex flex-row items-center justify-between">
		<div class="space-y-1">
			<CardTitle>Till Accounts</CardTitle>
			<CardDescription>Till accounts associated with this business.</CardDescription>
		</div>
		<Button onclick={() => create()}><PlusCircle class="mr-2 h-4 w-4" /> Add</Button>
	</CardHeader>
	<CardContent>
		{#if tillAccounts.filter((acc: TillAccountDetails) => acc.business_id === businessId).length > 0}
			<div class="space-y-4">
				{#each tillAccounts.filter((acc: TillAccountDetails) => acc.business_id === businessId) as account}
					<div
						onclick={() => edit({ id: account.account_id })}
						class="w-full p-2 hover:bg-muted cursor-pointer rounded-md"
						role="button"
						tabindex="0"
						onkeydown={(e) => e.key === 'Enter' && edit({ id: account.account_id })}
					>
						<div class="flex justify-between items-center w-full">
							<div class="text-left">
								<p class="text-sm font-medium">
									{account.till_number}
								</p>
								<p class="text-xs text-muted-foreground">
									Created: {new Date(account.created_at).toLocaleDateString()}
								</p>
							</div>
							<div>
								<p class="text-lg font-bold">
									{new Intl.NumberFormat('en-US', {
										style: 'currency',
										currency: 'KES'
									}).format(account.balance / 100)}
								</p>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<p>No till accounts found for this business.</p>
		{/if}
	</CardContent>
</Card>

{#if formData}
	<Dialog.Root open={isCreating || !!editingId} onOpenChange={(open) => !open && cancel()}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>{!!editingId ? 'Edit' : 'Add New'} Till Account</Dialog.Title>
			</Dialog.Header>
			<div class="grid gap-4 py-4">
				<div class="grid grid-cols-4 items-center gap-4">
					<Label for="tillNumber" class="text-right">Till Number</Label>
					<Input
						id="tillNumber"
						type="number"
						class="col-span-3"
						bind:value={formData.till_number}
					/>
				</div>
				{#if !editingId}
					<div class="grid grid-cols-4 items-center gap-4">
						<Label for="initialBalance" class="text-right">Initial Balance</Label>
						<Input
							id="initialBalance"
							type="number"
							class="col-span-3"
							bind:value={formData.initial_balance}
						/>
					</div>
				{/if}
				<div class="grid grid-cols-4 items-center gap-4">
					<Label for="storeNumber" class="text-right">Store Number</Label>
					<Input
						id="storeNumber"
						type="number"
						class="col-span-3"
						bind:value={formData.store_number}
					/>
				</div>
				<div class="grid grid-cols-4 items-center gap-4">
					<Label for="locationDescription" class="text-right">Location Description</Label>
					<Input
						id="locationDescription"
						type="text"
						class="col-span-3"
						bind:value={formData.location_description}
					/>
				</div>
				<C2BParametersForm
					bind:response_type={formData.response_type}
					bind:validation_url={formData.validation_url}
					bind:confirmation_url={formData.confirmation_url}
				/>
			</div>
			<Dialog.Footer>
				<Button onclick={handleSubmit} disabled={saving}>
					{#if saving}
						<LoaderCircle class="animate-spin h-3 w-4 mr-2" />
					{:else if editingId}
						<Save class="mr-2 h-4 w-4" /> Update
					{:else}
						<PlusCircle class="mr-2 h-4 w-4" /> Add
					{/if}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
