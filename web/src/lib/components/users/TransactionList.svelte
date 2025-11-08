<script lang="ts">
	import type { FullTransactionLog, UserDetails } from '$lib/api';
	import { MessageSquare, ArrowUpRight, ArrowDownLeft } from 'lucide-svelte';
	import {
		formatTransactionAmount,
		formatTransactionDate,
		TransactionStatus,
		TransactionType,
		resolveAccountAndNavigate
	} from '$lib/api';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { goto } from '$app/navigation';

	export let transactions: FullTransactionLog[];
	export let user: UserDetails | null = null;

	function getTransactionIcon(transaction: FullTransactionLog) {
		if (transaction.status === TransactionStatus.Completed) {
			if (transaction.direction == 'Debit') {
				return ArrowUpRight;
			} else if ((transaction.direction = 'Credit')) {
				return ArrowDownLeft;
			}
		}
		return MessageSquare;
	}

	function getTransactionColor(transaction: FullTransactionLog) {
		switch (transaction.direction) {
			case 'Credit':
				return 'text-green-600';
			case 'Debit':
				return 'text-red-700';
			default:
				return 'text-blue-600';
		}
	}

	function formatValues(transaction: FullTransactionLog) {
		const amount = formatTransactionAmount(transaction.transaction_amount);
		const new_balance = formatTransactionAmount(transaction.new_balance);
		const date = formatTransactionDate(transaction.transaction_date);
		const time = new Date(transaction.transaction_date).toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
		const transactionId = transaction.transaction_id;
		const cost = formatTransactionAmount(transaction.fee);

		return { amount, new_balance, date, time, transactionId, cost };
	}

	function isSentTransaction(transaction: FullTransactionLog): boolean {
		if (!user) return false;
		return transaction.from_name === user.name;
	}
</script>

<div class="mx-4 h-full overflow-auto">
	<ScrollArea class="mx-auto h-full max-w-xl">
		{#each transactions as transaction}
			<div class="m-4 flex {isSentTransaction(transaction) ? 'justify-end' : 'justify-start'}">
				<div
					class="max-w-[85%] rounded-lg p-3 shadow-md {isSentTransaction(transaction)
						? 'rounded-br-none bg-green-800 text-blue-900 text-white'
						: 'rounded-bl-none'}"
				>
					<div class="mb-1 flex items-center gap-2">
						<svelte:component
							this={getTransactionIcon(transaction)}
							size={16}
							class={getTransactionColor(transaction)}
						/>
						<span
							class="text-xs {!isSentTransaction(transaction)
								? 'text-gray-800 dark:text-white'
								: 'text-white'}"
						>
							{formatTransactionDate(transaction.transaction_date)}
						</span>
					</div>
					<p class="text-sm leading-relaxed">
						{#if transaction.transaction_type == TransactionType.SendMoney && transaction.direction == 'Debit'}
							{@const { date, time, amount, cost, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							sent to
							<button
								class="cursor-pointer hover:underline"
								onclick={() => resolveAccountAndNavigate(transaction.to_id, goto)}
							>
								<b>{transaction.to_name}</b>
							</button>
							on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
						{:else if transaction.transaction_type == TransactionType.SendMoney && transaction.direction == 'Credit'}
							{@const { date, time, amount, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							received from
							<button
								class="cursor-pointer hover:underline"
								onclick={() => resolveAccountAndNavigate(transaction.from_id, goto)}
							>
								<b>{transaction.from_name}</b>
							</button>
							on {date} at {time}. New M-PESA balance is {new_balance}.
						{:else if transaction.transaction_type == TransactionType.Deposit}
							{@const { date, time, amount, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed. Deposit
							<b>{amount}</b>
							on {date} at {time}. New M-PESA balance is {new_balance}.
						{:else if transaction.transaction_type == TransactionType.Paybill}
							{@const { date, time, amount, cost, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							paid to Pay Bill
							<button
								class="cursor-pointer hover:underline"
								onclick={() => resolveAccountAndNavigate(transaction.to_id, goto)}
							>
								<b>{transaction.to_name}</b>
							</button>
							for account {transaction.from_name || 'N/A'} on {date} at {time}. New M-PESA balance
							is {new_balance}. Transaction cost, {cost}
						{:else if transaction.transaction_type == TransactionType.BuyGoods}
							{@const { date, time, amount, cost, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							paid to Buy Goods
							<button
								class="cursor-pointer hover:underline"
								onclick={() => resolveAccountAndNavigate(transaction.to_id, goto)}
							>
								<b>{transaction.to_name}</b>
							</button>
							on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
						{:else if transaction.transaction_type == TransactionType.Withdraw}
							{@const { date, time, amount, cost, new_balance, transactionId } =
								formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							withdrawn at {transaction.from_name || 'Agent'} on {date} at {time}. New M-PESA
							balance is {new_balance}. Transaction cost, {cost}
						{:else}
							{@const { date, time, amount, cost, transactionId } = formatValues(transaction)}
							<b>{transactionId}</b>
							Confirmed.
							<b>{amount}</b>
							transaction on {date} at {time}. Transaction ID: {transaction.transaction_id}.
							Transaction cost, {cost}
						{/if}
					</p>
					<div class="mt-1 text-right text-xs font-medium">
						{formatTransactionAmount(transaction.transaction_amount)}
					</div>
				</div>
			</div>
		{:else}
			<div class="flex flex-col items-center justify-center h-full text-gray-500">
				<MessageSquare size={48} class="mb-4" />
				<p class="text-lg font-medium">No transactions yet</p>
				<p class="text-sm">This user hasn't made any transactions</p>
			</div>
		{/each}
	</ScrollArea>
</div>
