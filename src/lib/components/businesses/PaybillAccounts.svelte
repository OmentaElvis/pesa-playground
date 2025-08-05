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
    createPaybillAccount,
    updatePaybillAccount,
    type CreatePaybillAccountData,
    type UpdatePaybillAccountData,
  } from "$lib/api";
  import { PlusCircle, Save } from "lucide-svelte";

  export let paybillAccounts: any[] = [];
  export let businessId: number;

  let showCreatePaybillDialog = false;
  let showEditPaybillDialog = false;

  let selectedPaybillAccount: any = null;

  let newPaybill: CreatePaybillAccountData = {
    business_id: 0,
    initial_balance: 0,
    paybill_number: 0,
    account_validation_regex: "",
    validation_url: "",
    confirmation_url: "",
  };

  const dispatch = createEventDispatcher();

  async function handleCreatePaybillAccount() {
    if (businessId) {
      newPaybill.business_id = businessId;
      await createPaybillAccount(newPaybill);
      newPaybill = {
        business_id: 0,
        initial_balance: 0,
        paybill_number: 0,
        account_validation_regex: "",
        validation_url: "",
        confirmation_url: "",
      };
      dispatch("refresh");
      showCreatePaybillDialog = false;
    }
  }

  async function handleUpdatePaybillAccount() {
    if (selectedPaybillAccount) {
      const data: UpdatePaybillAccountData = {
        paybill_number: selectedPaybillAccount.paybill_number,
        account_validation_regex: selectedPaybillAccount.account_validation_regex,
        validation_url: selectedPaybillAccount.validation_url,
        confirmation_url: selectedPaybillAccount.confirmation_url,
      };
      await updatePaybillAccount(selectedPaybillAccount.id, data);
      dispatch("refresh");
      showEditPaybillDialog = false;
    }
  }
</script>

<Card>
  <CardHeader class="flex flex-row items-center justify-between">
    <div class="space-y-1">
      <CardTitle>Paybill Accounts</CardTitle>
      <CardDescription>Accounts associated with this business.</CardDescription>
    </div>
    <Dialog.Root bind:open={showCreatePaybillDialog}>
      <Dialog.Trigger>
        <Button><PlusCircle class="mr-2 h-4 w-4" /> Add</Button>
      </Dialog.Trigger>
      <Dialog.Content>
        <Dialog.Header>
          <Dialog.Title>Add New Paybill Account</Dialog.Title>
        </Dialog.Header>
        <div class="grid gap-4 py-4">
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newPaybillNumber" class="text-right">Paybill Number</Label>
            <Input
              id="newPaybillNumber"
              type="number"
              class="col-span-3"
              bind:value={newPaybill.paybill_number}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newPaybillInitialBalance" class="text-right">Initial Balance</Label>
            <Input
              id="newPaybillInitialBalance"
              type="number"
              class="col-span-3"
              bind:value={newPaybill.initial_balance}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newPaybillAccountValidationRegex" class="text-right"
              >Account Validation Regex</Label
            >
            <Input
              id="newPaybillAccountValidationRegex"
              type="text"
              class="col-span-3"
              bind:value={newPaybill.account_validation_regex}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newPaybillValidationUrl" class="text-right">Validation URL</Label>
            <Input
              id="newPaybillValidationUrl"
              type="text"
              class="col-span-3"
              bind:value={newPaybill.validation_url}
            />
          </div>
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="newPaybillConfirmationUrl" class="text-right">Confirmation URL</Label>
            <Input
              id="newPaybillConfirmationUrl"
              type="text"
              class="col-span-3"
              bind:value={newPaybill.confirmation_url}
            />
          </div>
        </div>
        <Dialog.Footer>
          <Button onclick={handleCreatePaybillAccount}>
            <PlusCircle class="mr-2 h-4 w-4" /> Add Paybill Account
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>
  </CardHeader>
  <CardContent>
    {#if paybillAccounts.filter((acc) => acc.business_id === businessId).length > 0}
      <div class="space-y-4">
        {#each paybillAccounts.filter((acc) => acc.business_id === businessId) as account}
          <Dialog.Root bind:open={showEditPaybillDialog}>
            <Dialog.Trigger
              onclick={() => (selectedPaybillAccount = account)}
              class="w-full cursor-pointer hover:bg-muted p-2"
            >
              <div class="flex justify-between items-center w-full">
                <div class="text-left">
                  <p class="text-sm font-medium">
                    {account.paybill_number}
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
                <Dialog.Title>Edit Paybill Account</Dialog.Title>
              </Dialog.Header>
              {#if selectedPaybillAccount}
                <div class="grid gap-4 py-4">
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editPaybillNumber" class="text-right">Paybill Number</Label>
                    <Input
                      id="editPaybillNumber"
                      type="number"
                      class="col-span-3"
                      bind:value={selectedPaybillAccount.paybill_number}
                    />
                  </div>
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editPaybillAccountValidationRegex" class="text-right"
                      >Account Validation Regex</Label
                    >
                    <Input
                      id="editPaybillAccountValidationRegex"
                      type="text"
                      class="col-span-3"
                      bind:value={selectedPaybillAccount.account_validation_regex}
                    />
                  </div>
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editPaybillValidationUrl" class="text-right">Validation URL</Label>
                    <Input
                      id="editPaybillValidationUrl"
                      type="text"
                      class="col-span-3"
                      bind:value={selectedPaybillAccount.validation_url}
                    />
                  </div>
                  <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="editPaybillConfirmationUrl" class="text-right"
                      >Confirmation URL</Label
                    >
                    <Input
                      id="editPaybillConfirmationUrl"
                      type="text"
                      class="col-span-3"
                      bind:value={selectedPaybillAccount.confirmation_url}
                    />
                  </div>
                </div>
                <Dialog.Footer>
                  <Button onclick={handleUpdatePaybillAccount}>
                    <Save class="mr-2 h-4 w-4" /> Update Paybill Account
                  </Button>
                </Dialog.Footer>
              {/if}
            </Dialog.Content>
          </Dialog.Root>
        {/each}
      </div>
    {:else}
      <p>No paybill accounts found for this business.</p>
    {/if}
  </CardContent>
</Card>
