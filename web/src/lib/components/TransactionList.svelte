<script lang="ts">
	import {
		getTransactionHistory,
		type HistoryFilter,
		type TransactionHistoryEntry,
		type HistoryScope,
		TransactionStatus,
		type SortDirection,
		getBusiness,
		getTransactionDirection
	} from '$lib/api';
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow
	} from '$lib/components/ui/table';
	import { debounce, formatAmount } from '$lib/utils';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import {
		ArrowDownLeft,
		ArrowUpRight,
		ArrowUpDown,
		RefreshCw,
		CheckCircle,
		CircleX,
		Loader,
		Undo2,
		ArrowLeftRight,
		RefreshCcw
	} from 'lucide-svelte';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Pagination from '$lib/components/ui/pagination/index.js';

	type SortKey = 'date' | 'amount' | 'status';

	let {
		scope,
		transactions = $bindable<TransactionHistoryEntry[]>([]),
		isLoading = $bindable<boolean>(false),
		showIcons: showIcon = true
	}: {
		scope: HistoryScope;
		transactions?: TransactionHistoryEntry[];
		isLoading?: boolean;
		showIcons?: boolean;
	} = $props();

	const displayTransactions: TransactionHistoryEntry[] = $derived(transactions);
	let sortKey: SortKey = $state('date');
	let sortDirection = $state<SortDirection>('Desc');
	let pagination = $state({ pageIndex: 1, pageSize: 20 });
	let perspective: number | number[] | null | undefined = $state();

	$effect(() => {
		(async () => {
			if (!scope) {
				perspective = null;
				return;
			}

			switch (scope.type) {
				case 'User':
					perspective = scope.id;
					break;
				case 'All':
					perspective = 0;
					break;
				case 'Business':
					if (scope.id) {
						try {
							const businessDetails = await getBusiness(scope.id);
							const accountIds = [];
							if (businessDetails.mmf_account) {
								accountIds.push(businessDetails.mmf_account.account_id);
							}
							if (businessDetails.utility_account) {
								accountIds.push(businessDetails.utility_account.account_id);
							}
							perspective = accountIds;
						} catch (error) {
							console.error(`Failed to fetch business details for id: ${scope.id}`, error);
							perspective = null; // Set to null on error
						}
					} else {
						perspective = null;
					}
					break;
				default:
					perspective = null;
			}
		})();
	});

	let searchQuery = $state('');
	let debouncedSearchQuery = $state('');
	const updateQuery = debounce((v: string) => (debouncedSearchQuery = v), 500);
	$effect(() => updateQuery(searchQuery));

	let uniqueStatuses = $derived.by(() => {
		return Object.values(TransactionStatus).filter(
			(value) => typeof value == 'string'
		) as TransactionStatus[];
	});

	let activeStatusFilters = $state<TransactionStatus[]>([]);

	async function loadTransactions() {
		isLoading = true;
		try {
			const filter: HistoryFilter = {
				scope,
				pagination: {
					limit: pagination.pageSize,
					offset: (pagination.pageIndex - 1) * pagination.pageSize
				},
				sorting: {
					by: sortKey,
					direction: sortDirection
				},
				filters: {
					statuses: activeStatusFilters.length > 0 ? activeStatusFilters : undefined,
					search_query: debouncedSearchQuery || undefined
				}
			};
			transactions = await getTransactionHistory(filter);
		} catch (error) {
			console.error('Failed to load transactions:', error);
			transactions = [];
		} finally {
			isLoading = false;
		}
	}

	function handleSort(key: SortKey) {
		if (sortKey === key) {
			sortDirection = sortDirection === 'Asc' ? 'Desc' : 'Asc';
		} else {
			sortKey = key;
			sortDirection = 'Desc';
		}
	}

	$effect(() => {
		loadTransactions();
	});
</script>

<div class="space-y-4">
	<div class="flex items-center justify-between gap-2">
		<Input placeholder="Search transactions..." bind:value={searchQuery} class="max-w-sm" />
		<div class="flex items-center gap-2">
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" class="ml-auto">Filter by Status</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content>
					<DropdownMenu.Label>Filter by Status</DropdownMenu.Label>
					<DropdownMenu.Separator />
					{#each uniqueStatuses as status}
						<DropdownMenu.CheckboxItem
							checked={activeStatusFilters.includes(status)}
							onCheckedChange={(checked) => {
								if (checked) {
									activeStatusFilters = [...activeStatusFilters, status];
								} else {
									activeStatusFilters = activeStatusFilters.filter((s) => s !== status);
								}
							}}
						>
							{status}
						</DropdownMenu.CheckboxItem>
					{/each}
					{#if activeStatusFilters.length > 0}
						<DropdownMenu.Separator />
						<DropdownMenu.Item onclick={() => (activeStatusFilters = [])}>
							Clear Filters
						</DropdownMenu.Item>
					{/if}
				</DropdownMenu.Content>
			</DropdownMenu.Root>
			<Button variant="ghost" size="icon" onclick={loadTransactions}>
				<RefreshCw class="h-4 w-4" />
			</Button>
		</div>
	</div>

	<div class="overflow-x-auto rounded-md border">
		<Table>
			<TableHeader>
				<TableRow>
					<TableHead class="w-[40px]"></TableHead>
					<TableHead>
						<Button variant="ghost" onclick={() => handleSort('date')}>
							Date <ArrowUpDown class="ml-2 h-4 w-4" />
						</Button>
					</TableHead>
					<TableHead>Transaction ID</TableHead>
					<TableHead>Context</TableHead>
					<TableHead>Sender</TableHead>
					<TableHead>Receiver</TableHead>
					<TableHead class="text-right">
						<Button variant="ghost" onclick={() => handleSort('amount')}>
							Amount <ArrowUpDown class="ml-2 h-4 w-4" />
						</Button>
					</TableHead>
					<TableHead>
						<Button variant="ghost" onclick={() => handleSort('status')}>
							Status <ArrowUpDown class="ml-2 h-4 w-4" />
						</Button>
					</TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{#if isLoading}
					<TableRow>
						<TableCell colspan={8} class="h-24 text-center">Loading...</TableCell>
					</TableRow>
				{:else if displayTransactions.length === 0}
					<TableRow>
						<TableCell colspan={8} class="h-24 text-center">No results found.</TableCell>
					</TableRow>
				{/if}
				{#each displayTransactions as log (`${log.transaction_id}${log.receiver_id}`)}
					<TableRow>
						<TableCell>
							{#if showIcon}
								{@const direction = getTransactionDirection(log, perspective)}
								<div
									class="flex items-center justify-center"
									class:text-green-500={direction === 'Inflow'}
									class:text-red-500={direction === 'Outflow'}
								>
									{#if direction === 'Inflow'}
										<ArrowDownLeft class="h-5 w-5" />
									{:else if direction == 'Outflow'}
										<ArrowUpRight class="h-5 w-5" />
									{:else if direction == 'Internal'}
										<RefreshCcw class="h-5 w-5 text-yellow-500" />
									{:else}
										<ArrowLeftRight class="h-5 w-5 text-muted-foreground" />
									{/if}
								</div>
							{/if}
						</TableCell>
						<TableCell>{new Date(log.date).toLocaleString()}</TableCell>
						<TableCell class="font-mono text-xs font-bold">{log.transaction_id}</TableCell>
						<TableCell>
							{#if log.notes}
								<div class="flex flex-col text-xs">
									{#if log.notes.type === 'PaybillPayment'}
										<span>Paybill: {log.notes.data.paybill_number}</span>
										<span class="text-muted-foreground">Ref: {log.notes.data.bill_ref_number}</span>
									{:else if log.notes.type === 'TillPayment'}
										<span>Till: {log.notes.data.till_number}</span>
									{:else if log.notes.type === 'AccountSetupFunding'}
										<span>Setup: {log.notes.data.account_type}</span>
									{/if}
								</div>
							{:else}
								{log.transaction_type}
							{/if}
						</TableCell>
						<TableCell>
							{#if log.sender_id}
								<a href="/accounts/{log.sender_id}" class="hover:underline">
									{log.sender_name}
								</a>
							{:else}
								<a href="/accounts/system" class="hover:underline">{log.sender_name}</a>
							{/if}
						</TableCell>
						<TableCell>
							<a href="/accounts/{log.receiver_id}" class="hover:underline">
								{log.receiver_name}
							</a>
						</TableCell>
						<TableCell class="text-right font-mono">
							{formatAmount(log.amount / 100)}
						</TableCell>
						<TableCell>
							<Badge variant="outline">
								{#if log.status == TransactionStatus.Completed}
									<CheckCircle class="text-green-500" />
								{:else if log.status == TransactionStatus.Failed}
									<CircleX class="text-red-500" />
								{:else if log.status == TransactionStatus.Pending}
									<Loader />
								{:else if log.status == TransactionStatus.Reversed}
									<Undo2 class="text-yellow-500" />
								{/if}
								{log.status}
							</Badge>
						</TableCell>
					</TableRow>
				{/each}
			</TableBody>
		</Table>
	</div>
	<Pagination.Root count={pagination.pageSize} bind:page={pagination.pageIndex}>
		{#snippet children({ pages, currentPage })}
			<Pagination.Content>
				<Pagination.Item>
					<Pagination.Previous />
				</Pagination.Item>
				{#each pages as page (page.key)}
					{#if page.type === 'ellipsis'}
						<Pagination.Item>
							<Pagination.Ellipsis />
						</Pagination.Item>
					{:else}
						<Pagination.Item>
							<Pagination.Link {page} isActive={currentPage === page.value}>
								{page.value}
							</Pagination.Link>
						</Pagination.Item>
					{/if}
				{/each}
				<Pagination.Item>
					<Pagination.Ellipsis />
				</Pagination.Item>
				<Pagination.Item>
					<Pagination.Next />
				</Pagination.Item>
			</Pagination.Content>
		{/snippet}
	</Pagination.Root>
</div>
