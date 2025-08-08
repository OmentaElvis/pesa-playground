<script lang="ts">
  import type { FullTransactionLog, UserDetails } from "$lib/api";
  import { MessageSquare, ArrowUpRight, ArrowDownLeft } from "lucide-svelte";
  import { formatTransactionAmount, formatTransactionDate, TransactionStatus, TransactionType } from "$lib/api";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  
  export let transactions: FullTransactionLog[];
  export let user: UserDetails | null = null;

  function getTransactionIcon(transaction: FullTransactionLog) {
    if (transaction.status === TransactionStatus.Completed) {
      if (transaction.direction == "Debit") {
        return ArrowUpRight;
      } else if (transaction.direction = "Credit") {
        return ArrowDownLeft;
      }
    }
    return MessageSquare;
  }

  function getTransactionColor(transaction: FullTransactionLog) {
    switch (transaction.direction) {
      case "Credit":
        return "text-green-600";
      case "Debit":
        return "text-red-600";
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
        return `<b>${transaction.transaction_id}</b> Confirmed. <b>${amount}</b> sent to <button onclick={viewDetails}><b>${transaction.to_name}</b></button> on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.Deposit:
        return `<b>${transaction.transaction_id}</b> Confirmed. Deposit <b>${amount}</b> on ${date} at ${time}. New M-PESA balance is ${new_balance}.`;
      case TransactionType.Paybill:
        return `<b>${transaction.transaction_id}</b> Confirmed. <b>${amount}</b> paid to Pay Bill <button onclick={viewDetails}><b>${transaction.to_name}</b></button> for account ${transaction.from_name || 'N/A'} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.BuyGoods:
        return `<b>${transaction.transaction_id}</b> Confirmed. <b>${amount}</b> paid to Buy Goods <button onclick={viewDetails}><b>${transaction.to_name}</b></button> on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      case TransactionType.Withdraw:
        return `<b>${transaction.transaction_id}</b> Confirmed. <b>${amount}</b> withdrawn at ${transaction.from_name || 'Agent'} on ${date} at ${time}. New M-PESA balance is ${new_balance}. Transaction cost, ${cost} `;
      default:
        return `${transaction.transaction_id} Confirmed. <b>${amount}</b> transaction on ${date} at ${time}. Transaction ID: ${transactionId}. Transaction cost, ${cost}`;
    }
  }

  function formatValues(transaction: FullTransactionLog) {
    const amount = formatTransactionAmount(transaction.transaction_amount);
    const new_balance = formatTransactionAmount(transaction.new_balance);
    const date = formatTransactionDate(transaction.transaction_date);
    const time = new Date(transaction.transaction_date).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
    const transactionId = transaction.transaction_id;
    const cost = formatTransactionAmount(transaction.fee);

    return {amount, new_balance, date, time, transactionId, cost}
  }

  function viewDetails(id: number) {
    alert(id)
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
              {#if transaction.transaction_type == TransactionType.SendMoney}
                {@const {date, time, amount, cost, new_balance, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. <b>{amount}</b> sent to <button onclick={() =>viewDetails(transaction.to_id)}><b>{transaction.to_name}</b></button> on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
              {:else if transaction.transaction_type ==  TransactionType.Deposit}
                {@const {date, time, amount, new_balance, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. Deposit <b>{amount}</b> on {date} at {time}. New M-PESA balance is  {new_balance}.
              {:else if transaction.transaction_type == TransactionType.Paybill }
                {@const {date, time, amount, cost, new_balance, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. <b>{amount}</b> paid to Pay Bill <button onclick={() => viewDetails(transaction.to_id)}><b>{transaction.to_name}</b></button> for account {transaction.from_name || 'N/A'} on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
              {:else if transaction.transaction_type == TransactionType.BuyGoods}
                {@const {date, time, amount, cost, new_balance, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. <b>{amount}</b> paid to Buy Goods <button onclick={() => viewDetails(transaction.to_id)}><b>{transaction.to_name}</b></button> on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
              {:else if transaction.transaction_type == TransactionType.Withdraw}
                {@const {date, time, amount, cost, new_balance, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. <b>{amount}</b> withdrawn at {transaction.from_name || 'Agent'} on {date} at {time}. New M-PESA balance is {new_balance}. Transaction cost, {cost}
              {:else}
                {@const {date, time, amount, cost, transactionId} = formatValues(transaction)}
                <b>{transactionId}</b> Confirmed. <b>{amount}</b> transaction on {date} at {time}. Transaction ID: {transaction.transaction_id}. Transaction cost, {cost}
              {/if}
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
