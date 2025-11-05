<script lang="ts">
  import {
    Phone,
    Wallet,
    EllipsisVertical,
    CardSim,
    Banknote,
    Save,
    LoaderCircle,
  } from "lucide-svelte";
  import { formatAmount, getInitials } from "$lib/utils";
  import TransactionList from "$lib/components/users/TransactionList.svelte";
  import { page } from "$app/state";
  import {
    getUser,
    listFullTransactionLogs,
    TransactionType,
    transfer,
    type FullTransactionLog,
    type UserDetails,
  } from "$lib/api";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Button } from "$lib/components/ui/button";
  import SimToolkit from "$lib/components/users/SimToolkit.svelte";
  import DiceBearAvatar from "$lib/components/ui/avatar/DiceBearAvatar.svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import { toast } from "svelte-sonner";
  import { onMount } from "svelte";
  import { listen } from "$lib/api";
  import { activeUserPageId } from "$lib/stores/activePageStore";
  import { sidebarStore } from "$lib/stores/sidebarStore";

  let id = $derived(page.params.id);

  let user: Promise<UserDetails | null> = $derived(getUser(Number(id)));
  let transactions: Promise<FullTransactionLog[]> = $derived(
    listFullTransactionLogs(Number(id)),
  );

  let fundsToAdd = $state(1000);
  let depositDialogOpen = $state(false);
  let addingDeposit = $state(false);

  async function handleDepositFunds() {
    let user_details = await user;
    if (!user_details) return;

    try {
      addingDeposit = true;
      await transfer(
        null,
        user_details.id,
        fundsToAdd * 100,
        TransactionType.Deposit,
      );

      toast.success(
        `Deposited ${formatAmount(fundsToAdd)} to ${user_details.name} ${user_details.phone}`,
      );
    } catch (err) {
      console.log(err);
      toast.error("" + err);
    } finally {
      addingDeposit = false;
      depositDialogOpen = false;
    }
  }

  $effect(() => {
    user.then((u) => {
      if (!u) return;
      
      sidebarStore.register({
        id: 'user-stk-menu',
        title: `${u.name}'s Sim Toolkit`,
        icon: CardSim,
        component: SimToolkit,
        props: {
          user: u
        }
      });
    })
  });

  onMount(() => {
    // Set active user page ID
    activeUserPageId.set(Number(id));

    // Listen for new transactions to refresh data
    const unlisten = listen<FullTransactionLog>("new_transaction", (event) => {
      const isRelated =
        event.payload.to_id === Number(id) ||
        event.payload.from_id === Number(id);
      if (isRelated) {
        console.log(`Refreshing user ${id} due to new transaction.`);
        user = getUser(Number(id));
        transactions = listFullTransactionLogs(Number(id));
      }
    });

    // Cleanup on component destroy
    return () => {
      activeUserPageId.set(null);
      unlisten.then((f) => f());
      sidebarStore.unregister('user-stk-menu');
    };
  });
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
              <DiceBearAvatar
                seed={`${user.id}-${user.name}`}
                fallback={getInitials(user.name)}
              />
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
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <EllipsisVertical />
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Group>
                <DropdownMenu.Label>{user.name}</DropdownMenu.Label>
                <DropdownMenu.Separator />
              </DropdownMenu.Group>
              <DropdownMenu.Group>
                <Dialog.Root bind:open={depositDialogOpen}>
                  <Dialog.Trigger>
                    <Button variant="ghost"><Banknote /> Deposit funds</Button>
                  </Dialog.Trigger>
                  <Dialog.Content>
                    <Dialog.Header>
                      <Dialog.Title>Add funds to {user.name}</Dialog.Title>
                      <Dialog.Description>
                        Current balance <span class="font-bold text-green-600"
                          >{formatAmount(user.balance)}</span
                        >
                      </Dialog.Description>
                    </Dialog.Header>
                    <div class="grid gap-2 mt-2">
                      <Label for="shortCode">Amount</Label>
                      <Input
                        id="amount"
                        type="number"
                        min={1}
                        max={99999999}
                        bind:value={fundsToAdd}
                      />
                    </div>
                    <div>
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 1)}
                        class="mt-4">+ 1</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 10)}
                        class="mt-4">+ 10</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 50)}
                        class="mt-4">+ 50</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 100)}
                        class="mt-4">+ 100</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 250)}
                        class="mt-4">+ 250</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 500)}
                        class="mt-4">+ 500</Button
                      >
                      <Button
                        variant="outline"
                        onclick={() => (fundsToAdd = fundsToAdd + 1000)}
                        class="mt-4">+ 1000</Button
                      >
                    </div>
                    <div>
                      <Button
                        onclick={handleDepositFunds}
                        disabled={addingDeposit}
                        class="mt-4"
                      >
                        {#if addingDeposit}
                          <LoaderCircle class="animate-spin" />
                        {:else}
                          <Save />
                        {/if}
                        Deposit funds</Button
                      >
                    </div>
                  </Dialog.Content>
                </Dialog.Root>
              </DropdownMenu.Group>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </div>

      <!-- Transactions -->
      {#await transactions then transactions}
        <TransactionList {transactions} {user} />
      {/await}
    {/if}
  {:catch err}
    <div class="p-4">
      Failed to get user({id}): {err}
    </div>
  {/await}
</div>
