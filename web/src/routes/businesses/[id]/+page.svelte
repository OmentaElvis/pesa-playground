<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
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
		type PaybillAccountDetails,
		type TillAccountDetails,
		type ProjectSummary,
		type BusinessDetails,
		revenueSettlements
	} from '$lib/api';
	import { goto } from '$app/navigation';
	import {
		ArrowLeftRight,
		ChevronsLeftRightEllipsis,
		DollarSign,
		HandCoins,
		LoaderCircle,
		PiggyBank,
		Save,
		Settings,
		Wallet,
		WalletMinimal
	} from 'lucide-svelte';
	import PaybillAccounts from '$lib/components/businesses/PaybillAccounts.svelte';
	import TillAccounts from '$lib/components/businesses/TillAccounts.svelte';
	import Projects from '$lib/components/businesses/Projects.svelte';
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

	let businessId: number | undefined = $state(undefined);
	let updatingBusiness = $state(false);
	let reconciling = $state(false);

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

	onMount(() => {
		const tab = page.url.searchParams.get('biz_tab');
		if (tab && tab !== currentTab) {
			currentTab = tab;
		}
	});
</script>

<main class="container mx-auto space-y-6 p-6">
	{#if business && businessId}
		<div>
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground">{business.name}</h1>
				<p class="text-sm text-muted-foreground">Manage business information.</p>
				<div>Shortcode: <Badge>#{business.short_code}</Badge></div>
			</div>
		</div>
		<div class="grid grid-cols-3 gap-4 max-md:grid-cols-2 max-sm:grid-cols-1">
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><PiggyBank /> Working account</Card.Title>
					<Card.Description>
						Funds parked in the working (MMF) account for liquidity and balance management.
					</Card.Description>
				</Card.Header>
				<Card.Content>
					<p class="font-bold">{formatAmount(business.mmf_account.balance / 100)}</p>
				</Card.Content>
			</Card.Root>
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><Wallet /> Utility account</Card.Title>
					<Card.Description>
						Operational balance used to process business payment transactions.
					</Card.Description>
				</Card.Header>
				<Card.Content>
					<p class="font-bold">{formatAmount(business.utility_account.balance / 100)}</p>
				</Card.Content>
			</Card.Root>
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2"><HandCoins /> Business charges</Card.Title>
					<Card.Description>
						Cumulative charges applied to business transactions. Needs tobe reconciled.
					</Card.Description>
				</Card.Header>
				<Card.Content class="flex items-center">
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
				</Card.Content>
			</Card.Root>
		</div>
		<Separator />
		<Tabs.Root bind:value={currentTab} class="">
			<Tabs.List>
				<Tabs.Trigger value="accounts"><WalletMinimal /> Accounts</Tabs.Trigger>
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
	{:else}
		<p><LoaderCircle class="animate-spin" /> Loading business details...</p>
	{/if}
</main>
