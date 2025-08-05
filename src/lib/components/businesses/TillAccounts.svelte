<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { createEventDispatcher } from "svelte";
  import {
    createTillAccount,
    updateTillAccount,
    type CreateTillAccountData,
    type UpdateTillAccountData,
  } from "$lib/api";
  import { PlusCircle, Save } from "lucide-svelte";

  export let tillAccounts: any[] = [];
  export let businessId: number;

  let showCreateTillDialog = false;
  let showEditTillDialog = false;

  let selectedTillAccount: any = null;

  let newTill: CreateTillAccountData = {
    business_id: 0,
    initial_balance: 0,
    store_number: 0,
    till_number: 0,
    location_description: "",
  };

  const dispatch = createEventDispatcher();

  async function handleCreateTillAccount() {
    if (businessId) {
      newTill.business_id = businessId;
      await createTillAccount(newTill);
      newTill = {
        business_id: 0,
        initial_balance: 0,
        store_number: 0,
        till_number: 0,
        location_description: "",
      };
      dispatch("refresh");
      showCreateTillDialog = false;
    }
  }

  async function handleUpdateTillAccount() {
    if (selectedTillAccount) {
      const data: UpdateTillAccountData = {
        till_number: selectedTillAccount.till_number,
        store_number: selectedTillAccount.store_number,
        location_description: selectedTillAccount.location_description,
      };
      await updateTillAccount(selectedTillAccount.id, data);
      dispatch("refresh");
      showEditTillDialog = false;
    }
  }
</script>

<Card>
  <CardHeader class="flex flex-row items-center justify-between">
    <div class="space-y-1">
      <CardTitle>Till Accounts</CardTitle>
      <CardDescription>Till accounts associated with this business.</CardDescription>
    </div>
    <Dialog.Root bind:open={showCreateTillDialog}>
      <Dialog.Trigger>
        <Button><PlusCircle class="mr-2 h-4 w-4" /> Add</Button>
      </Dialog.Trigger>
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Add New Till Account</Dialog.Title>
        </Dialog.Header>
        <div class="grid gap-4 py-4">
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newTillNumber" class="text-right">Till Number</Label>
            <Input
              id="newTillNumber"
              type="number"
              class="col-span-3"
              bind:value={newTill.till_number}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newTillInitialBalance" class="text-right">Initial Balance</Label>
            <Input
              id="newTillInitialBalance"
              type="number"
              class="col-span-3"
              bind:value={newTill.initial_balance}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newTillStoreNumber" class="text-right">Store Number</Label>
            <Input
              id="newTillStoreNumber"
              type="number"
              class="col-span-3"
              bind:value={newTill.store_number}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newTillLocationDescription" class="text-right">Location Description</Label>
            <Input
              id="newTillLocationDescription"
              type="text"
              class="col-span-3"
              bind:value={newTill.location_description}
            />
          </div>
        </div>
        <Dialog.Footer>
          <Button onclick={handleCreateTillAccount}>
            <PlusCircle class="mr-2 h-4 w-4" /> Add Till Account
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>
  </CardHeader>
  <CardContent>
    {#if tillAccounts.filter((acc) => acc.business_id === businessId).length > 0}
      <div class="space-y-4">
        {#each tillAccounts.filter((acc) => acc.business_id === businessId) as account}
          <Dialog.Root bind:open={showEditTillDialog}>
            <Dialog.Trigger
              onclick={() => (selectedTillAccount = account)} class="w-full p-2 hover:bg-muted cursor-pointer">
              <div class="flex justify-between items-center w-full">
                <div class="text-left">
                  <p class="text-sm font-medium">
                    {account.till_number}
                  </p>
                  <p class="text-xs text-muted-foreground">
                    Created: {new Date(account.created_at).toLocaleDateString()}
                  </p>
                </div>
                <div>
                  <p class="text-lg font-bold">
                    {new Intl.NumberFormat("en-US", {
                      style: "currency",
                      currency: "KES",
                    }).format(account.balance / 100)}
                  </p>
                </div>
              </div>
            </Dialog.Trigger>
            <Dialog.Content>
              <Dialog.Header>
                <Dialog.Title>Edit Till Account</Dialog.Title>
              </Dialog.Header>
              {#if selectedTillAccount}
                <div class="grid gap-4 py-4">
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editTillNumber" class="text-right">Till Number</Label>
                    <Input
                      id="editTillNumber"
                      type="number"
                      class="col-span-3"
                      bind:value={selectedTillAccount.till_number}
                    />
                  </div>
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editTillStoreNumber" class="text-right">Store Number</Label>
                    <Input
                      id="editTillStoreNumber"
                      type="number"
                      class="col-span-3"
                      bind:value={selectedTillAccount.store_number}
                    />
                  </div>
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editTillLocationDescription" class="text-right"
                      >Location Description</Label
                    >
                    <Input
                      id="editTillLocationDescription"
                      type="text"
                      class="col-span-3"
                      bind:value={selectedTillAccount.location_description}
                    />
                  </div>
                </div>
                <Dialog.Footer>
                  <Button onclick={handleUpdateTillAccount}>
                    <Save class="mr-2 h-4 w-4" /> Update Till Account
                  </Button>
                </Dialog.Footer>
              {/if}
            </Dialog.Content>
          </Dialog.Root>
        {/each}
      </div>
    {:else}
      <p>No till accounts found for this business.</p>
    {/if}
  </CardContent>
</Card>
