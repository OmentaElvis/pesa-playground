<script lang="ts">
  import type { FullTransactionLog, UserDetails } from "$lib/api";
  import { MessageSquare, ArrowUpRight, ArrowDownLeft } from "lucide-svelte";
  import { formatTransactionAmount, formatTransactionDate, TransactionStatus, TransactionType } from "$lib/api";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  
  export let transactions: FullTransactionLog[];
  export let user: UserDetails | null = null;

  function getTransactionIcon(transaction: FullTransactionLog) {
    if (transaction.status === TransactionStatus.Completed) {
      if (transaction.transaction_type === TransactionType.SendMoney || transaction.transaction_type === TransactionType.Withdraw) {
        return ArrowUpRight;
      } else if (transaction.transaction_type === TransactionType.Deposit || transaction.transaction_type === TransactionType.Paybill || transaction.transaction_type === TransactionType.BuyGoods) {
        return ArrowDownLeft;
      }
    }
    return MessageSquare;
  }

  function getTransactionColor(transaction: FullTransactionLog) {
    switch (transaction.status) {
      case TransactionStatus.Completed:
        return "text-green-600";
      case TransactionStatus.Failed:
        return "text-red-600";
      case TransactionStatus.Pending:
      default:
        return "text-blue-600";
    }
  }

  function getMpesaMessage(transaction: FullTransactionLog): string {
    if (transaction.status !== TransactionStatus.Completed) {
      return `Transaction ${transaction.status}.`;
    }

    const amount = formatTransactionAmount(transaction.transaction_amount);
    const new_balance = formatTransactionAmount(transaction.new_balance);
    const date = formatTransactionDate(transaction.transaction_date);
    const time = new Date(transaction.transaction_date).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const transactionId = transaction.transaction_id;
    const cost = formatTransactionAmount(transaction.fee);

    switch (transaction.transaction_type) {
      case TransactionType.SendMoney:
        return `${transaction.transaction_id} Confirmed. ${amount} sent to ${transaction.to_name} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.Deposit:
        return `${transaction.transaction_id} Confirmed. ${amount} received from ${transaction.from_name || 'M-PESA'} on ${date} at ${time}. New M-PESA balance is ${new_balance}.`;
      case TransactionType.Paybill:
        return `${transaction.transaction_id} Confirmed. ${amount} paid to Pay Bill ${transaction.to_name} for account ${transaction.from_name || 'N/A'} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.BuyGoods:
        return `${transaction.transaction_id} Confirmed. ${amount} paid to Buy Goods ${transaction.to_name} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.Withdraw:
        return `${transaction.transaction_id} Confirmed. ${amount} withdrawn from ${transaction.from_name || 'Agent'} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      default:
        return `${transaction.transaction_id} Confirmed. ${amount} transaction on ${date} at ${time}. Transaction ID: ${transactionId}. Transaction cost, ${cost}`;
    }
  }

  function isSentTransaction(transaction: FullTransactionLog): boolean {
    if (!user) return false;
    return transaction.from_name === user.name;
  }
</script>

<div class="h-full overflow-auto m-4">
    <ScrollArea class="h-full">
      {#each transactions as transaction}
        <div class="m-4 flex {isSentTransaction(transaction) ? 'justify-end' : 'justify-start'}">
          <div
            class="max-w-[70%] p-3 rounded-lg shadow-md {isSentTransaction(transaction) ? 'bg-blue-100 text-blue-900 rounded-br-none' : 'rounded-bl-none'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <svelte:component
                this={getTransactionIcon(transaction)}
                size={16}
                class={getTransactionColor(transaction)}
              />
              <span class="text-xs text-gray-500">{formatTransactionDate(transaction.transaction_date)}</span>
            </div>
            <p class="text-sm leading-relaxed">
              {getMpesaMessage(transaction)}
            </p>
            <div class="text-right text-xs font-medium mt-1">
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
