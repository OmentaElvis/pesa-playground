<script lang="ts">
	import {
		listAccountsFullTransactionLogs,
		type FullTransactionLog,
		type PaybillAccountDetails,
		type TillAccountDetails
	} from '$lib/api';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { formatAmount } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { ArrowDownLeft, ArrowUpRight } from 'lucide-svelte';

	type AccountInfo = {
		name: string;
		type: 'Paybill' | 'Till';
		number: string;
	};

	let {
		paybills = [] as PaybillAccountDetails[],
		tills = [] as TillAccountDetails[],
		transactions: transactionsProp = null as FullTransactionLog[] | null
	} = $props();

	let localTransactions: FullTransactionLog[] = $state([]);
	let isLoading = $state(true);

	const displayTransactions = $derived(transactionsProp ?? localTransactions);

	const accountDetailsMap: Map<number, AccountInfo> = $derived.by(() => {
		const map = new Map<number, AccountInfo>();
		for (const p of paybills) {
			map.set(p.account_id, {
				name: `Paybill (${p.paybill_number})`,
				type: 'Paybill',
				number: p.paybill_number.toString()
			});
		}
		for (const t of tills) {
			map.set(t.account_id, {
				name: t.location_description || `Till (${t.till_number})`,
				type: 'Till',
				number: t.till_number.toString()
			});
		}
		return map;
	});

	async function loadTransactions() {
		// Only fetch if transactions are not passed as a prop
		if (transactionsProp !== null) {
			isLoading = false;
			return;
		}

		const accountIds = [...paybills.map((p) => p.account_id), ...tills.map((t) => t.account_id)];
		if (accountIds.length === 0) {
			localTransactions = [];
			isLoading = false;
			return;
		}
		isLoading = true;
		try {
			localTransactions = await listAccountsFullTransactionLogs(accountIds, 20);
		} catch (error) {
			console.error('Failed to load transactions:', error);
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		loadTransactions();
	});

	function getDisplayName(log: FullTransactionLog, direction: 'from' | 'to'): string {
		const id = direction === 'from' ? log.from_id : log.to_id;
		const defaultName = direction === 'from' ? log.from_name : log.to_name;

		// Look up specific till/paybill names from the map.
		if (id !== null && accountDetailsMap.has(id)) {
			return accountDetailsMap.get(id)!.name;
		}
		// Fallback to the name provided in the log (which will be "System" if from_id is null).
		return defaultName;
	}
</script>

<div>
	{#if isLoading}
		<div class="flex items-center justify-center p-8">
			<p>Loading transactions...</p>
		</div>
	{:else if displayTransactions.length === 0}
		<div class="p-8 text-center text-muted-foreground">
			<p>No transactions found for this project's accounts.</p>
		</div>
	{:else}
		<div class="overflow-x-auto rounded-md border">
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead class="w-[40px]"></TableHead>
						<TableHead>Date</TableHead>
						<TableHead>Transaction ID</TableHead>
						<TableHead>From</TableHead>
						<TableHead>To</TableHead>
						<TableHead class="text-right">Amount</TableHead>
						<TableHead>Status</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{#each displayTransactions as log (log.transaction_id)}
						<TableRow>
							<TableCell>
								<div
									class="flex items-center justify-center"
									class:text-green-500={log.direction === 'Credit'}
									class:text-red-500={log.direction === 'Debit'}
								>
									{#if log.direction === 'Credit'}
										<ArrowDownLeft class="h-5 w-5" />
									{:else}
										<ArrowUpRight class="h-5 w-5" />
									{/if}
								</div>
							</TableCell>
							<TableCell>{new Date(log.transaction_date).toLocaleString()}</TableCell>
							<TableCell class="font-mono text-xs font-bold">{log.transaction_id}</TableCell>
							<TableCell>
								{#if log.from_id !== null}
									<a href="/accounts/{log.from_id}" class="hover:underline">
										{getDisplayName(log, 'from')}
									</a>
								{:else if log.from_name === 'System'}
									<a href="/accounts/system" class="hover:underline">System</a>
								{:else}
									{getDisplayName(log, 'from')}
								{/if}
							</TableCell>
							<TableCell>
								{#if log.to_id !== null && log.to_name != 'Unknown'}
									<a href="/accounts/{log.to_id}" class="hover:underline">
										{getDisplayName(log, 'to')}
									</a>
								{:else if log.to_name === 'System'}
									<a href="/accounts/system" class="hover:underline">System</a>
								{:else}
									{getDisplayName(log, 'to')}
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
	{/if}
</div>
