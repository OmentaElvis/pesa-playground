<script lang="ts">
	import { onMount } from 'svelte';
	import {
		getPaybillAccounts,
		getTillAccounts,
		getUsers,
		listAccountsFullTransactionLogs,
		transfer,
		TransactionType,
		type FullTransactionLog,
		type TransactionStats,
		getTransactionStats
	} from '$lib/api';
	import { LoaderCircle, PiggyBank, Scale, Landmark } from 'lucide-svelte';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { formatAmount } from '$lib/utils';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';

	// Main page state
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Stats
	let systemBalance = $state('...');
	let totalFees = $state(0);
	let totalVolume = $state(0);
	let totalTransactions = $state(0);
	let stats: TransactionStats | null = $state(null);

	// Data stores
	let allTransactions = $state<FullTransactionLog[]>([]);
	let allAccounts = $state<{ label: string; value: string }[]>([]);

	// Deposit form state
	let depositAmount = $state(10000);
	let selectedAccountId = $state<string | undefined>(undefined);
	let isDepositing = $state(false);

	async function loadSystemData() {
		isLoading = true;
		try {
			const paybills = await getPaybillAccounts();
			const tills = await getTillAccounts();
			const users = await getUsers();
			stats = await getTransactionStats();

			const allAccountIds = [
				...paybills.map((p) => p.account_id),
				...tills.map((t) => t.account_id),
				...users.map((u) => u.id)
			];

			const userAccounts = users.map((u) => ({
				label: `User: ${u.name} (${u.phone})`,
				value: u.id.toString()
			}));
			const paybillAccounts = paybills.map((p) => ({
				label: `Paybill: ${p.paybill_number}`,
				value: p.account_id.toString()
			}));
			const tillAccounts = tills.map((t) => ({
				label: `Till: ${t.till_number}`,
				value: t.account_id.toString()
			}));
			allAccounts = [...userAccounts, ...paybillAccounts, ...tillAccounts];
			if (allAccounts.length > 0) {
				selectedAccountId = allAccounts[0].value;
			}

			// Fetch all transaction logs for all accounts
			const txns = await listAccountsFullTransactionLogs(allAccountIds);
			allTransactions = txns;
			totalTransactions = stats.total_count;

			// --- Correct Balance Calculation ---
			let deposits = 0;
			let fees = 0;
			let volume = 0;

			for (const txn of txns) {
				if (txn.from_id === null) {
					deposits += txn.transaction_amount;
				}
				fees += txn.fee;
				volume += txn.transaction_amount;
			}

			const initialBalance = 999999999999999;
			const newBalance = initialBalance - deposits + stats.total_fees;

			totalFees = stats.total_fees / 100;
			totalVolume = stats.total_volume / 100;
			systemBalance = (newBalance / 100).toLocaleString('en-US', {
				minimumFractionDigits: 2,
				maximumFractionDigits: 2
			});
		} catch (e: any) {
			console.error('Failed to load system data:', e);
			error = `An unexpected error occurred: ${e.message}`;
		} finally {
			isLoading = false;
		}
	}

	async function handleDeposit() {
		if (!selectedAccountId || !depositAmount || depositAmount <= 0) {
			toast.error('Please select an account and enter a valid amount.');
			return;
		}
		isDepositing = true;
		try {
			await transfer(
				null,
				parseInt(selectedAccountId, 10),
				depositAmount * 100,
				TransactionType.Deposit
			);
			toast.success(
				`Successfully deposited ${formatAmount(depositAmount)} to account ${selectedAccountId}.`
			);
			// Refresh data to show new balance and transaction
			await loadSystemData();
		} catch (e: any) {
			console.error('Deposit failed:', e);
			toast.error(`Deposit failed: ${e.message}`);
		} finally {
			isDepositing = false;
		}
	}

	onMount(loadSystemData);
</script>

<main class="container mx-auto space-y-6 p-6">
	{#if isLoading}
		<div class="flex h-64 flex-col items-center justify-center gap-4">
			<LoaderCircle class="h-8 w-8 animate-spin text-primary" />
			<p class="text-muted-foreground">Calculating system state...</p>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardHeader>
				<CardTitle>Error</CardTitle>
			</CardHeader>
			<CardContent>
				<p>{error}</p>
			</CardContent>
		</Card>
	{:else}
		<div class="space-y-8">
			<div>
				<h1 class="text-3xl font-bold tracking-tight">System Overview</h1>
				<p class="text-muted-foreground">A top-level view of the entire simulated economy.</p>
			</div>

			<!-- Stats Grid -->
			<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
				<Card>
					<CardHeader class="flex flex-row items-center justify-between pb-2">
						<CardTitle class="text-sm font-medium">System Balance</CardTitle>
						<Landmark class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">KES {systemBalance}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader class="flex flex-row items-center justify-between pb-2">
						<CardTitle class="text-sm font-medium">Total Fees Collected</CardTitle>
						<PiggyBank class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">{formatAmount(totalFees)}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader class="flex flex-row items-center justify-between pb-2">
						<CardTitle class="text-sm font-medium">Total Transaction Volume</CardTitle>
						<Scale class="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div class="text-2xl font-bold">{formatAmount(totalVolume)}</div>
						<p class="text-xs text-muted-foreground">{totalTransactions} transactions</p>
					</CardContent>
				</Card>
			</div>

			<!-- Deposit Card -->
			<Card>
				<CardHeader>
					<CardTitle>Deposit Funds</CardTitle>
					<CardDescription>
						Add funds to any account from the system's main balance.
					</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
						<div class="space-y-2">
							<Label for="account">Target Account</Label>
							<Select.Root bind:value={selectedAccountId} type="single">
								<Select.Trigger class="w-full">
									{#if selectedAccountId}
										{allAccounts.find((a) => a.value === selectedAccountId)?.label}
									{:else}
										Select an account...
									{/if}
								</Select.Trigger>
								<Select.Content>
									{#each allAccounts as account}
										<Select.Item value={account.value}>{account.label}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>
						<div class="space-y-2">
							<Label for="amount">Amount (KES)</Label>
							<Input id="amount" type="number" bind:value={depositAmount} />
						</div>
					</div>
				</CardContent>
				<CardFooter>
					<Button onclick={handleDeposit} disabled={isDepositing} class="w-full md:w-auto">
						{#if isDepositing}
							<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
							Depositing...
						{:else}
							Deposit
						{/if}
					</Button>
				</CardFooter>
			</Card>

			<!-- Transaction Ledger Card -->
			<Card>
				<CardHeader>
					<CardTitle>System-Wide Transaction Ledger</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="overflow-x-auto rounded-md border">
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Date</TableHead>
									<TableHead>ID</TableHead>
									<TableHead>From</TableHead>
									<TableHead>To</TableHead>
									<TableHead class="text-right">Amount</TableHead>
									<TableHead>Status</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{#each allTransactions as log (log.transaction_id)}
									<TableRow>
										<TableCell>{new Date(log.transaction_date).toLocaleString()}</TableCell>
										<TableCell class="font-mono text-xs font-bold">{log.transaction_id}</TableCell>
										<TableCell>
											{#if log.from_id !== null}
												<a href="/accounts/{log.from_id}" class="hover:underline">
													{log.from_name}
												</a>
											{:else}
												<a href="/accounts/system" class="hover:underline">
													{log.from_name}
												</a>
											{/if}
										</TableCell>
										<TableCell>
											{#if log.to_id !== null && log.to_name !== 'Unknown'}
												<a href="/accounts/{log.to_id}" class="hover:underline">
													{log.to_name}
												</a>
											{:else}
												<!-- This case should not happen for a deposit -->
												{log.to_name}
											{/if}
										</TableCell>
										<TableCell class="text-right font-mono">
											{formatAmount(log.transaction_amount / 100)}
										</TableCell>
										<TableCell>
											<Badge
												variant={log.status === 'Completed'
													? 'default'
													: log.status === 'Failed'
														? 'destructive'
														: 'outline'}
											>
												{log.status}
											</Badge>
										</TableCell>
									</TableRow>
								{/each}
							</TableBody>
						</Table>
					</div>
				</CardContent>
			</Card>
		</div>
	{/if}
</main>
