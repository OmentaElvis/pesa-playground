<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Separator } from '$lib/components/ui/separator';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import {
		getBusiness,
		updateBusiness,
		deleteBusiness,
		getPaybillAccountsByBusinessId,
		getTillAccountsByBusinessId,
		getProjectsByBusinessId,
		getOperatorsByBusinessId,
		type PaybillAccountDetails,
		type TillAccountDetails,
		type ProjectSummary,
		type BusinessDetails,
		type BusinessOperator,
		revenueSettlements,
		transfer,
		TransactionType
	} from '$lib/api';
	import { goto } from '$app/navigation';
	import {
		ArrowLeftRight,
		ChevronsLeftRightEllipsis,
		DollarSign,
		HandCoins,
		Plus,
		ArrowRightLeft,
		LoaderCircle,
		PiggyBank,
		Save,
		Settings,
		Wallet,
		WalletMinimal,
		Users
	} from 'lucide-svelte';
	import PaybillAccounts from '$lib/components/businesses/PaybillAccounts.svelte';
	import TillAccounts from '$lib/components/businesses/TillAccounts.svelte';
	import Projects from '$lib/components/businesses/Projects.svelte';
	import Operators from '$lib/components/businesses/Operators.svelte';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';
	import { formatAmount } from '$lib/utils';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import DangerAction from '$lib/components/shared/DangerAction.svelte';
	import { toast } from 'svelte-sonner';
	import TransactionList from '$lib/components/TransactionList.svelte';

	let business: BusinessDetails | null = $state(null);
	let paybillAccounts: PaybillAccountDetails[] = $state([]);
	let tillAccounts: TillAccountDetails[] = $state([]);
	let projects: ProjectSummary[] = $state([]);
	let operators: BusinessOperator[] = $state([]);

	let businessId: number | undefined = $state(undefined);
	let updatingBusiness = $state(false);
	let reconciling = $state(false);

	let showDepositDialog = $state(false);
	let showMoveFundsDialog = $state(false);
	let depositAmount = $state(0);
	let moveFundsAmount = $state(0);

	$effect(() => {
		if (page.params.id) {
			businessId = parseInt(page.params.id);
		}
	});

	async function loadBusinessDetails() {
		if (businessId) {
			business = await getBusiness(businessId);
			paybillAccounts = await getPaybillAccountsByBusinessId(businessId);
			tillAccounts = await getTillAccountsByBusinessId(businessId);
			projects = await getProjectsByBusinessId(businessId);
			operators = await getOperatorsByBusinessId(businessId);
		}
	}

	async function handleUpdateBusiness() {
		if (business) {
			try {
				updatingBusiness = true;
				await updateBusiness(business.id, {
					name: business.name
				});
				await loadBusinessDetails();
				toast.success('Business details updated!');
			} catch (error) {
				console.log(`Update business error: ${error}`);
				toast.error(`Update business error: ${error}`);
			} finally {
				updatingBusiness = false;
			}
		}
	}

	async function handleDeleteBusiness() {
		if (business) {
			try {
				await deleteBusiness(business.id);
				toast.success(`Business "${business.name}" deleted`);
				goto('/projects');
			} catch (err) {
				toast.error(`Error deleting business: ${err}`);
			}
		}
	}

	onMount(async () => {
		await loadBusinessDetails();
	});

	// URL state management
	let currentTab = $state('accounts');

	$effect(() => {
		const url = new URL(page.url);
		if (url.searchParams.get('biz_tab') !== currentTab) {
			url.searchParams.set('biz_tab', currentTab);
			goto(url, { replaceState: true, keepFocus: true, noScroll: true });
		}
	});

	function clearActionParams() {
		const url = new URL(page.url);
		url.searchParams.delete('biz_action');
		url.searchParams.delete('biz_edit_paybill');
		url.searchParams.delete('biz_edit_till');
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	function setAction(action: string, id?: number) {
		const url = new URL(page.url);
		url.searchParams.set('biz_action', action);
		if (id) {
			if (action === 'edit_paybill') {
				url.searchParams.set('biz_edit_paybill', id.toString());
			} else if (action === 'edit_till') {
				url.searchParams.set('biz_edit_till', id.toString());
			}
		}
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	async function reconcileFunds() {
		if (!business) return;
		try {
			await revenueSettlements(business.id);
			business = await getBusiness(business.id);
			toast.success('Funds reconciled');
		} catch (err) {
			toast.error(`Failed to reconcile business funds: ${err}`);
		}
	}

	async function onRefreshOperators() {
		if (businessId) {
			operators = await getOperatorsByBusinessId(businessId);
		}
	}

	async function handleDeposit() {
		try {
			showDepositDialog = false;

			if (!business) return;

			let txn = await transfer(
				0,
				business.mmf_account.account_id,
				depositAmount * 100,
				TransactionType.Deposit,
				{
					type: 'AccountSetupFunding',
					data: {
						account_type: 'Mmf'
					}
				}
			);

			toast.success(`Success ${txn.id}: Deposited ${depositAmount} to Working (MMF) account.`);
			loadBusinessDetails();
		} catch (err) {
			toast.error(`Failed to deposit funds to mmf account: ${err}`);
		} finally {
			depositAmount = 0;
		}
	}

	async function handleMoveFunds() {
		try {
			showMoveFundsDialog = false;

			if (!business) return;

			let txn = await transfer(
				business.mmf_account.account_id,
				business.utility_account.account_id,
				moveFundsAmount * 100,
				TransactionType.TopupUtility
			);
			loadBusinessDetails();

			toast.success(`Success ${txn.id}: Moved Ksh ${moveFundsAmount} to utility account.`);
		} catch (err) {
			toast.error(`Failed to move funds: ${err}`);
		} finally {
			moveFundsAmount = 0;
		}
	}

	onMount(() => {
		const tab = page.url.searchParams.get('biz_tab');
		if (tab && tab !== currentTab) {
			currentTab = tab;
		}
	});
</script>

<main class="container mx-auto space-y-6 p-6">
	{#if business && businessId}
		<div class="flex items-start justify-between">
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground">{business.name}</h1>
				<p class="text-sm text-muted-foreground">Manage business information.</p>
				<div>Shortcode: <Badge>#{business.short_code}</Badge></div>
			</div>
			<div class="hidden flex-col items-end md:flex">
				<div>
					<p class="text-sm text-muted-foreground">Utility Balance</p>
					<p class="text-3xl font-bold text-green-600 dark:text-green-500">
						{formatAmount(business.utility_account.balance / 100)}
					</p>
				</div>
				<div class="flex items-center">
					<PiggyBank class="mr-2 h-4 w-4 text-muted-foreground" />
					<p class="text-lg font-semibold">{formatAmount(business.mmf_account.balance / 100)}</p>
				</div>
			</div>
		</div>
		<div class="grid grid-cols-3 gap-4 max-md:grid-cols-2 max-sm:grid-cols-1">
			<Card.Root class="flex flex-col">
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><PiggyBank /> Working account</Card.Title>
					<Card.Description>
						Funds parked in the working (MMF) account for liquidity and balance management.
					</Card.Description>
				</Card.Header>
				<Card.Content class="flex flex-grow flex-col justify-end">
					<div class="flex gap-2">
						<Button size="sm" variant="outline" onclick={() => (showDepositDialog = true)}>
							<Plus class="mr-1 h-4 w-4" /> Deposit
						</Button>
						<Button size="sm" variant="outline" onclick={() => (showMoveFundsDialog = true)}>
							<ArrowRightLeft class="mr-1 h-4 w-4" /> Move
						</Button>
					</div>
				</Card.Content>
				<Card.Footer>
					<p class="font-bold">{formatAmount(business.mmf_account.balance / 100)}</p>
				</Card.Footer>
			</Card.Root>
			<Card.Root class="flex flex-col">
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Wallet /> Utility account</Card.Title>
					<Card.Description>
						This is the primary operational account. All incoming payments to the business are
						deposited here (like C2B paybill/till), and all outgoing payments (like B2C
						disbursements) are made from this account.
					</Card.Description>
				</Card.Header>
				<Card.Content class="flex-grow" />
				<Card.Footer>
					<p class="font-bold">{formatAmount(business.utility_account.balance / 100)}</p>
				</Card.Footer>
			</Card.Root>
			<Card.Root class="flex flex-col">
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><HandCoins /> Business charges</Card.Title>
					<Card.Description>
						Cumulative charges applied to business transactions. Needs tobe reconciled.
					</Card.Description>
				</Card.Header>
				<Card.Content class="flex-grow" />
				<Card.Footer class="flex items-center">
					<p
						class="flex-1 cursor-pointer font-bold {business.charges_amount < 0
							? 'text-red-500'
							: 'text-green-500'}"
					>
						{formatAmount(business.charges_amount || 0)}
					</p>
					<Button
						disabled={(business.charges_amount >= 0 && business.utility_account.balance == 0) ||
							reconciling}
						onclick={reconcileFunds}
					>
						{#if reconciling}
							<LoaderCircle class="animate-spin" />
						{:else}
							<ArrowLeftRight />
						{/if}
						Reconcile
					</Button>
				</Card.Footer>
			</Card.Root>
		</div>
		<Separator />
		<Tabs.Root bind:value={currentTab} class="">
			<Tabs.List>
				<Tabs.Trigger value="accounts"><WalletMinimal /> Accounts</Tabs.Trigger>
				<Tabs.Trigger value="operators"><Users /> Operators</Tabs.Trigger>
				<Tabs.Trigger value="projects"><ChevronsLeftRightEllipsis /> Projects</Tabs.Trigger>
				<Tabs.Trigger value="transactions"><DollarSign /> Transactions</Tabs.Trigger>
				<Tabs.Trigger value="settings"><Settings /> Settings</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="accounts">
				<h3 class="mt-6 text-lg font-medium">Associated Accounts</h3>
				<div class="grid gap-4 md:grid-cols-2">
					<PaybillAccounts
						{paybillAccounts}
						{businessId}
						refresh={() => {
							loadBusinessDetails();
							clearActionParams();
						}}
						create={() => setAction('new_paybill')}
						edit={(e) => setAction('edit_paybill', e.id)}
						isCreating={page.url.searchParams.get('biz_action') === 'new_paybill'}
						editingId={page.url.searchParams.get('biz_edit_paybill')}
						cancel={clearActionParams}
					/>
					<TillAccounts
						{tillAccounts}
						{businessId}
						refresh={() => {
							loadBusinessDetails();
							clearActionParams();
						}}
						create={() => setAction('new_till')}
						edit={(e) => setAction('edit_till', e.id)}
						isCreating={page.url.searchParams.get('biz_action') === 'new_till'}
						editingId={page.url.searchParams.get('biz_edit_till')}
						cancel={clearActionParams}
					/>
				</div>
			</Tabs.Content>
			<Tabs.Content value="operators">
				<Operators {operators} businessId={business.id} onrefresh={onRefreshOperators} />
			</Tabs.Content>
			<Tabs.Content value="projects">
				<h3 class="mt-6 text-lg font-medium">Associated Projects</h3>
				<Projects {projects} {businessId} on:refresh={loadBusinessDetails} />
			</Tabs.Content>
			<Tabs.Content value="transactions">
				<TransactionList
					scope={{
						type: 'Business',
						id: businessId
					}}
				/>
			</Tabs.Content>
			<Tabs.Content value="settings" class="flex flex-col gap-4">
				<form>
					<Card.Root>
						<Card.Header>
							<Card.Title>Update business</Card.Title>
						</Card.Header>
						<Card.Content>
							<div class="grid gap-2">
								<Label for="name">Business Name</Label>
								<Input id="name" type="text" bind:value={business.name} />
							</div>
							<div class="mt-2 grid gap-2">
								<Label for="shortCode">Short Code</Label>
								<Input id="shortCode" type="text" bind:value={business.short_code} />
							</div>
						</Card.Content>
						<Card.Footer>
							<Button
								onclick={handleUpdateBusiness}
								disabled={updatingBusiness || business.name == '' || business.short_code == ''}
								class="mt-4"
							>
								{#if !updatingBusiness}
									<Save />
								{:else}
									<LoaderCircle class="animate-spin" />
								{/if}
								Update Business
							</Button>
						</Card.Footer>
					</Card.Root>
				</form>
				<DangerAction
					title="Delete Business"
					description="This will delete this businesses, associated projects and transaction history. This action cannot be undone."
					buttonLabel="Delete"
					dialogTitle="Are you absolutely sure?"
					dialogDescription="This action is irreversible. All your data will be permanently deleted."
					onConfirm={handleDeleteBusiness}
				></DangerAction>
			</Tabs.Content>
		</Tabs.Root>

		<AlertDialog.Root bind:open={showDepositDialog}>
			<AlertDialog.Content>
				<AlertDialog.Header>
					<AlertDialog.Title>Deposit Funds</AlertDialog.Title>
					<AlertDialog.Description>
						<div class="space-y-4">
							<p>Choose an amount to deposit or enter a custom amount.</p>
							<div class="flex flex-wrap gap-2">
								<Button size="sm" onclick={() => (depositAmount = depositAmount + 100)}>
									+100
								</Button>
								<Button size="sm" onclick={() => (depositAmount = depositAmount + 500)}>
									+500
								</Button>
								<Button size="sm" onclick={() => (depositAmount = depositAmount * 2)}>x2</Button>
								<Button size="sm" onclick={() => (depositAmount = 0)}>Clear</Button>
							</div>
							<div class="grid gap-2">
								<Label for="deposit-amount">Amount</Label>
								<Input
									id="deposit-amount"
									type="number"
									min="0"
									bind:value={depositAmount}
									placeholder="Enter amount"
								/>
							</div>
						</div>
					</AlertDialog.Description>
				</AlertDialog.Header>
				<AlertDialog.Footer>
					<AlertDialog.Cancel onclick={() => (depositAmount = 0)}>Cancel</AlertDialog.Cancel>
					<AlertDialog.Action onclick={handleDeposit} disabled={depositAmount <= 0}>
						Deposit
					</AlertDialog.Action>
				</AlertDialog.Footer>
			</AlertDialog.Content>
		</AlertDialog.Root>

		<AlertDialog.Root bind:open={showMoveFundsDialog}>
			<AlertDialog.Content>
				<AlertDialog.Header>
					<AlertDialog.Title>Move Funds to Utility Account</AlertDialog.Title>
					<AlertDialog.Description>
						<div class="grid gap-2">
							<Label for="transfer-amount">Amount</Label>
							<Input
								id="transfer-amount"
								type="number"
								min="1"
								max={business.mmf_account.balance / 100}
								bind:value={moveFundsAmount}
								placeholder="Enter amount"
							/>
						</div>
					</AlertDialog.Description>
				</AlertDialog.Header>
				<AlertDialog.Footer>
					<AlertDialog.Cancel onclick={() => (moveFundsAmount = 0)}>Cancel</AlertDialog.Cancel>
					<AlertDialog.Action
						onclick={handleMoveFunds}
						disabled={moveFundsAmount <= 0 || moveFundsAmount > business.mmf_account.balance / 100}
					>
						Transfer
					</AlertDialog.Action>
				</AlertDialog.Footer>
			</AlertDialog.Content>
		</AlertDialog.Root>
	{:else}
		<p><LoaderCircle class="animate-spin" /> Loading business details...</p>
	{/if}
</main>
