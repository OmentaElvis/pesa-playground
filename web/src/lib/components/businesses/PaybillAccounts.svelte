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
		createPaybillAccount,
		updatePaybillAccount,
		type CreatePaybillAccountData,
		type PaybillAccountDetails,
		type UpdatePaybillAccountData
	} from '$lib/api';
	import { PlusCircle, Save } from 'lucide-svelte';
	import { toast } from 'svelte-sonner';
	import C2BParametersForm from './C2BParametersForm.svelte';

	let { paybillAccounts, businessId, isCreating, editingId, refresh, create, edit, cancel } = $props<{
		paybillAccounts: PaybillAccountDetails[];
		businessId: number;
		isCreating: boolean;
		editingId: string | null;
		refresh: () => void,
		create: () => void,
		edit: (params: {id: number}) => void,
		cancel: () => void,
	}>();

	let saving = $state(false);
	let formData: (CreatePaybillAccountData & { account_id?: number }) | null = $state(null);

	$effect(() => {
		if (isCreating) {
			formData = {
				business_id: businessId,
				initial_balance: 0,
				paybill_number: 0,
				account_validation_regex: '',
				validation_url: '',
				confirmation_url: '',
				response_type: C2BResponseType.Completed
			};
		} else if (editingId) {
			const account = paybillAccounts.find((acc: PaybillAccountDetails) => acc.account_id === parseInt(editingId!));
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
				const data: UpdatePaybillAccountData = {
					paybill_number: formData.paybill_number,
					account_validation_regex: formData.account_validation_regex || undefined,
					validation_url: formData.validation_url || undefined,
					confirmation_url: formData.confirmation_url || undefined,
					response_type: formData.response_type
				};
				await updatePaybillAccount(parseInt(editingId), data);
				toast.success('Paybill account updated successfully.');
			} else {
				await createPaybillAccount({
					...formData,
					business_id: businessId,
					account_validation_regex: formData.account_validation_regex || undefined,
					validation_url: formData.validation_url || undefined,
					confirmation_url: formData.confirmation_url || undefined
				});
				toast.success('Paybill account created successfully.');
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
			<CardTitle>Paybill Accounts</CardTitle>
			<CardDescription>Accounts associated with this business.</CardDescription>
		</div>
		<Button onclick={() => create()}><PlusCircle class="mr-2 h-4 w-4" /> Add</Button>
	</CardHeader>
	<CardContent>
		{#if paybillAccounts.filter((acc: PaybillAccountDetails) => acc.business_id === businessId).length > 0}
			<div class="space-y-4">
				{#each paybillAccounts.filter((acc: PaybillAccountDetails) => acc.business_id === businessId) as account}
					<div
						onclick={() => edit({ id: account.account_id })}
						class="w-full cursor-pointer hover:bg-muted p-2 rounded-md"
						role="button"
						tabindex="0"
						onkeydown={(e) => e.key === 'Enter' && edit({ id: account.account_id })}
					>
						<div class="flex justify-between items-center w-full">
							<div class="text-left">
								<p class="text-sm font-medium">
									{account.paybill_number}
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
			<p>No paybill accounts found for this business.</p>
		{/if}
	</CardContent>
</Card>

{#if formData}
	<Dialog.Root open={isCreating || !!editingId} onOpenChange={(open) => !open && cancel()}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>{!!editingId ? 'Edit' : 'Add New'} Paybill Account</Dialog.Title>
			</Dialog.Header>
			<div class="grid gap-4 py-4">
				<div class="grid grid-cols-4 items-center gap-4">
					<Label for="paybillNumber" class="text-right">Paybill Number</Label>
					<Input
						id="paybillNumber"
						type="number"
						class="col-span-3"
						bind:value={formData.paybill_number}
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
					<Label for="accountValidationRegex" class="text-right">Account Validation Regex</Label>
					<Input
						id="accountValidationRegex"
						type="text"
						class="col-span-3"
						bind:value={formData.account_validation_regex}
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
					{#if editingId}
						<Save class="mr-2 h-4 w-4" /> Update
					{:else}
						<PlusCircle class="mr-2 h-4 w-4" /> Add
					{/if}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
