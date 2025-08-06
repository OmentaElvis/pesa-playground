<script lang="ts">
  import { Phone, Wallet, EllipsisVertical, CardSim } from "lucide-svelte";
  import { formatAmount, getInitials } from "$lib/utils";
  import TransactionList from "$lib/components/users/TransactionList.svelte";
  import { page } from "$app/state";
  import { getUser, listFullTransactionLogs, type FullTransactionLog, type UserDetails } from "$lib/api";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Button } from "$lib/components/ui/button";
  import SimToolkit from "$lib/components/users/SimToolkit.svelte";
  import DiceBearAvatar from "$lib/components/ui/avatar/DiceBearAvatar.svelte";

  
  let id = $derived(page.params.id);
  let stkOpen = $state(false);
  
  let user: Promise<UserDetails | null> = $derived(getUser(Number(id)));
  let transactions: Promise<FullTransactionLog[]> = $derived(listFullTransactionLogs(Number(id)));


</script>

<div class="h-full flex flex-col">
  <!-- Header -->
  {#await user}
    Loading user
  {:then user}
   {#if !user}
      User not found
   {:else}
      <div class="border-b border-gray-200 p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3 flex-1">
            <div class="w-12 h-12">
              <DiceBearAvatar seed={`${user.id}-${user.name}`} fallback={getInitials(user.name)} />
            </div>
            <div>
              <h1 class="text-xl font-semibold">{user.name}</h1>
              <div class="flex items-center gap-4 text-sm">
                <span class="flex items-center gap-1">
                  <Phone size={14} />
                  {user.phone}
                </span>
                <span class="flex items-center gap-1">
                  <Wallet size={14} />
                  Balance: {formatAmount(user.balance)}
                </span>
              </div>
            </div>
          </div>
          <Button variant="ghost" onclick={()=> stkOpen=true}>
            <CardSim />
          </Button>
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <EllipsisVertical />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Group>
                <DropdownMenu.Label>{user.name}</DropdownMenu.Label>
                <DropdownMenu.Separator />
              </DropdownMenu.Group>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </div>

      <!-- Transactions -->
      {#await transactions then transactions}
        <TransactionList {transactions} {user} />
      {/await}
      <SimToolkit bind:open={stkOpen} {user} />
    {/if}

  {:catch err}
    <div class="p-4">
      Failed to get user({id}): {err}
    </div>
  {/await}
</div>
