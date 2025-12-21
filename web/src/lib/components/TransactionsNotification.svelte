<script lang="ts">
	import {
		listen,
		resolveAccountAndNavigate,
		type FullTransactionLog,
		type UnlistenFn
	} from '$lib/api';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { formatAmount } from '$lib/utils';
	import { Bell } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { transactionLogStore } from '$lib/stores/transactionLogStore';
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';

	function getTransactionLogDescription(log: FullTransactionLog): string {
		if (log.transaction_type === 'deposit') {
			return 'Deposit';
		} else if (log.direction === 'Inflow') {
			return 'Received';
		} else if (log.direction === 'Outflow') {
			if (log.transaction_type === 'send_money') {
				return 'Sent';
			} else if (log.transaction_type === 'paybill' || log.transaction_type === 'buy_goods') {
				return 'Paid';
			} else if (log.transaction_type === 'withdraw') {
				return 'Withdrawn';
			}
		}
		return 'Transacted'; // Fallback
	}

	function getTransactionLogSummarySentence(log: FullTransactionLog): string {
		const formattedAmount = formatAmount(log.transaction_amount / 100);

		if (log.transaction_type === 'deposit') {
			return `Deposit of ${formattedAmount} to ${log.to_name}`;
		} else if (log.direction === 'Inflow') {
			return `${log.to_name} received ${formattedAmount} from ${log.from_name}`;
		} else if (log.direction === 'Outflow') {
			if (log.transaction_type === 'send_money') {
				return `${log.from_name} sent ${formattedAmount} to ${log.to_name}`;
			} else if (log.transaction_type === 'paybill') {
				return `${log.from_name} paid ${formattedAmount} to Pay Bill ${log.to_name}`;
			} else if (log.transaction_type === 'buy_goods') {
				return `${log.from_name} paid ${formattedAmount} to Buy Goods ${log.to_name}`;
			} else if (log.transaction_type === 'withdraw') {
				return `${log.from_name} withdrew ${formattedAmount}`;
			}
		}
		return `Transaction of ${formattedAmount} between ${log.from_name} and ${log.to_name}`; // Fallback
	}

	let unlistenFunctions: UnlistenFn[] = [];
	onMount(() => {
		listen<FullTransactionLog>('new_transaction', (e) => {
			console.log('New transaction log received', e.payload);
			transactionLogStore.add(e.payload);
		})
			.then((un) => {
				unlistenFunctions.push(un);
			})
			.catch((err) => {
				console.error('Failed to set up new_transaction listener:', err);
			});
	});

	onDestroy(() => {
		unlistenFunctions.forEach((unlisten) => unlisten());
	});
</script>

<Popover.Root>
	<Popover.Trigger>
		<Button variant="ghost" size="icon" class="relative">
			<Bell />
			{#if $transactionLogStore.length > 0}
				<div
					class="absolute top-1 right-1 h-3 w-3 rounded-full border-2 border-muted bg-red-500"
				></div>
			{/if}
		</Button>
	</Popover.Trigger>
	<Popover.Content class="w-96">
		<div class="mb-2 flex items-center justify-between">
			<h3 class="font-medium">Unread Transactions</h3>
			<Button variant="link" size="sm" onclick={() => transactionLogStore.reset()}>
				Clear All
			</Button>
		</div>
		<ScrollArea class="h-72">
			<div class="flex flex-col gap-2">
				{#each $transactionLogStore as log (log.transaction_id + log.direction)}
					{@const account_id_to_visit = log.direction === 'Inflow' ? log.to_id : log.from_id}
					{#if account_id_to_visit}
						<button
							class="block rounded-md p-2 text-left text-sm hover:bg-secondary"
							onclick={() => {
								resolveAccountAndNavigate(account_id_to_visit, goto);
								transactionLogStore.remove(log.transaction_id);
							}}
						>
							<div class="font-semibold">{getTransactionLogSummarySentence(log)}</div>
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-1">
									<span class="font-semibold">{getTransactionLogDescription(log)}</span>
									<span
										class:text-green-500={log.direction === 'Inflow'}
										class:text-red-500={log.direction === 'Outflow'}
									>
										{log.direction === 'Inflow' ? '+' : '-'}{formatAmount(
											log.transaction_amount / 100
										)}
									</span>
								</div>
								<span class="text-xs text-muted-foreground">
									{new Date(log.transaction_date).toLocaleTimeString()}
								</span>
							</div>
						</button>
					{/if}
				{:else}
					<div class="text-center text-muted-foreground p-4">No new transactions.</div>
				{/each}
			</div>
		</ScrollArea>
	</Popover.Content>
</Popover.Root>
