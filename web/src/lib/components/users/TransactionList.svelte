<script lang="ts">
	import type {
		CalculatedDirection as CalculatedTransactionDirection,
		TransactionHistoryEntry,
		UserDetails
	} from '$lib/api';
	import { MessageSquare, ArrowUpRight, ArrowDownLeft } from 'lucide-svelte';
	import {
		formatTransactionAmount,
		formatTransactionDate,
		getTransactionDirection,
		resolveAccountAndNavigate,
		TransactionStatus,
		TransactionType
	} from '$lib/api';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { goto } from '$app/navigation';

	let {
		transactions,
		user
	}: {
		transactions: TransactionHistoryEntry[];
		user: UserDetails | null;
	} = $props();

	function getTransactionIcon(
		transaction: TransactionHistoryEntry,
		direction: CalculatedTransactionDirection
	) {
		if (transaction.status === TransactionStatus.Completed) {
			if (direction === 'Outflow') {
				return ArrowUpRight;
			} else if (direction === 'Inflow') {
				return ArrowDownLeft;
			}
		}
		return MessageSquare;
	}

	function getTransactionColor(direction: CalculatedTransactionDirection) {
		switch (direction) {
			case 'Inflow':
				return 'text-green-500';
			case 'Outflow':
				return 'text-red-500';
			default:
				return 'text-blue-500';
		}
	}

	function formatValues(
		transaction: TransactionHistoryEntry,
		direction: CalculatedTransactionDirection
	) {
		const amount = formatTransactionAmount(transaction.amount);
		let newBalance;
		if (direction === 'Inflow') {
			newBalance = formatTransactionAmount(transaction.receiver_balance || 0);
		} else {
			newBalance = formatTransactionAmount(transaction.sender_balance || 0);
		}
		const date = formatTransactionDate(transaction.date);
		const time = new Date(transaction.date).toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		});
		const transactionId = transaction.transaction_id;
		const cost = formatTransactionAmount(transaction.fee);

		return { amount, newBalance, date, time, transactionId, cost };
	}

	function isSentTransaction(direction: CalculatedTransactionDirection): boolean {
		return direction === 'Outflow';
	}
</script>

<div class="mx-4 h-full overflow-auto">
	<ScrollArea class="mx-auto h-full max-w-xl">
		{#if transactions.length > 0}
			{#each transactions as transaction (transaction.transaction_id)}
				{@const direction = getTransactionDirection(transaction, user?.id)}
				{@const { date, time, amount, cost, newBalance, transactionId } = formatValues(
					transaction,
					direction
				)}
				{@const Icon = getTransactionIcon(transaction, direction)}
				<div class="m-4 flex {isSentTransaction(direction) ? 'justify-end' : 'justify-start'}">
					<div
						class="max-w-[85%] rounded-lg p-3 shadow-md {isSentTransaction(direction)
							? 'rounded-br-none bg-green-800 text-white'
							: 'rounded-bl-none bg-background'}"
					>
						<div class="mb-1 flex items-center gap-2">
							<Icon size={16} class={getTransactionColor(direction)} />
							<span
								class="text-xs {!isSentTransaction(direction)
									? 'text-foreground'
									: 'text-gray-300'}"
							>
								{date} at {time}
							</span>
						</div>
						<p class="text-sm leading-relaxed">
							{#if transaction.transaction_type === TransactionType.SendMoney && direction === 'Outflow'}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								sent to
								<button
									class="cursor-pointer hover:underline"
									onclick={() => resolveAccountAndNavigate(transaction.receiver_id, goto)}
								>
									<b>{transaction.receiver_name}</b>
								</button>
								on {date} at {time}. New M-PESA balance is {newBalance}. Transaction cost, {cost}
							{:else if transaction.transaction_type === TransactionType.SendMoney && direction === 'Inflow'}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								received from
								<button
									class="cursor-pointer hover:underline"
									onclick={() => resolveAccountAndNavigate(transaction.sender_id || 0, goto)}
								>
									<b>{transaction.sender_name}</b>
								</button>
								on {date} at {time}. New M-PESA balance is {newBalance}.
							{:else if transaction.transaction_type === TransactionType.Deposit}
								<b>{transactionId}</b>
								Confirmed. Deposit
								<b>{amount}</b>
								on {date} at {time}. New M-PESA balance is {newBalance}.
							{:else if transaction.transaction_type === TransactionType.Paybill}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								paid to Pay Bill
								<button
									class="cursor-pointer hover:underline"
									onclick={() => resolveAccountAndNavigate(transaction.receiver_id, goto)}
								>
									<b>{transaction.receiver_name}</b>
								</button>
								{#if transaction.notes?.type === 'PaybillPayment'}
									for account {transaction.notes.data.bill_ref_number}
								{/if}
								on {date} at {time}. New M-PESA balance is {newBalance}. Transaction cost, {cost}
							{:else if transaction.transaction_type === TransactionType.BuyGoods}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								paid to Buy Goods
								<button
									class="cursor-pointer hover:underline"
									onclick={() => resolveAccountAndNavigate(transaction.receiver_id, goto)}
								>
									<b>{transaction.receiver_name}</b>
								</button>
								{#if transaction.notes?.type === 'TillPayment'}
									using Till {transaction.notes.data.till_number}
								{/if}
								on {date} at {time}. New M-PESA balance is {newBalance}. Transaction cost, {cost}
							{:else if transaction.transaction_type === TransactionType.Withdraw}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								withdrawn at {transaction.sender_name || 'Agent'} on {date} at {time}. New M-PESA
								balance is {newBalance}. Transaction cost, {cost}
							{:else}
								<b>{transactionId}</b>
								Confirmed.
								<b>{amount}</b>
								transaction on {date} at {time}. Transaction ID: {transaction.transaction_id}.
								Transaction cost, {cost}
							{/if}
						</p>
						<div class="mt-1 text-right text-xs font-medium">
							{formatTransactionAmount(transaction.amount)}
						</div>
					</div>
				</div>
			{:else}
				<div class="flex h-full flex-col items-center justify-center text-muted-foreground">
					<MessageSquare size={48} class="mb-4" />
					<p class="text-lg font-medium">No transactions yet</p>
					<p class="text-sm">This user hasn't made any transactions.</p>
				</div>
			{/each}
		{/if}
	</ScrollArea>
</div>
