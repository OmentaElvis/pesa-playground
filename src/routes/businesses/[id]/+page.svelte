<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { Separator } from "$lib/components/ui/separator";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import {
    getBusiness,
    updateBusiness,
    deleteBusiness,
    getPaybillAccountsByBusinessId,
    getTillAccountsByBusinessId,
    getProjectsByBusinessId,
    type FullTransactionLog,
    listAccountsFullTransactionLogs,
    type PaybillAccountDetails,
    type TillAccountDetails,
    type ProjectSummary,
    type BusinessDetails,
    formatTransactionAmount,
  } from "$lib/api";
  import { goto } from "$app/navigation";
  import { ArrowRightLeft, ChevronsLeftRightEllipsis, DollarSign, MoveDownLeft, MoveUpRight, Pencil, Save, Trash, WalletMinimal } from "lucide-svelte";
  import PaybillAccounts from "$lib/components/businesses/PaybillAccounts.svelte";
  import TillAccounts from "$lib/components/businesses/TillAccounts.svelte";
  import Projects from "$lib/components/businesses/Projects.svelte";
  import { Label } from "$lib/components/ui/label";
  import { Input } from "$lib/components/ui/input";
  import * as Table from "$lib/components/ui/table/index.js";
  import { formatDate } from "$lib/utils";

  let business: BusinessDetails | null = null;
  let paybillAccounts: PaybillAccountDetails[] = [];
  let tillAccounts: TillAccountDetails[] = [];
  let projects: ProjectSummary[] = [];

  interface Transaction extends FullTransactionLog {
    account_type: "Till" | "Paybill",
  }
  let transactions: Transaction[] = [];

  let businessId: number;

  $: if (page.params.id) {
    businessId = parseInt(page.params.id);
  }

  async function loadBusinessDetails() {
    if (businessId) {
      business = await getBusiness(businessId);
      paybillAccounts = await getPaybillAccountsByBusinessId(businessId);
      tillAccounts = await getTillAccountsByBusinessId(businessId);
      projects = await getProjectsByBusinessId(businessId);
    }
  }

  async function loadTransactions() {
    if (businessId) {
      let paybillTransactions: Transaction[] = (await listAccountsFullTransactionLogs(paybillAccounts.map((acc) => acc.account_id))).map((txn) => {
        return {
          account_type: "Paybill",
          ...txn
        }
      });
      
      let tillTransactions: Transaction[] = (await listAccountsFullTransactionLogs(tillAccounts.map(acc => acc.account_id))).map((txn) => {
        return {
          account_type: "Till",
          ...txn
        }
      });
      transactions = paybillTransactions.concat(tillTransactions);
    }
  }

  async function handleUpdateBusiness() {
    if (business) {
      await updateBusiness(business.id, {
        name: business.name,
      });
      await loadBusinessDetails();
    }
  }

  async function handleDeleteBusiness() {
    if (business && confirm("Are you sure you want to delete this business?")) {
      await deleteBusiness(business.id);
      goto("/businesses");
    }
  }

  onMount(async () => {
    await loadBusinessDetails();
    await loadTransactions();
  });
</script>

<div class="space-y-6 p-6">
  {#if business}
    <div>
      <div>
        <h3 class="text-lg font-medium">{business.name}</h3>
        <p class="text-sm text-muted-foreground">
          Manage business information.
        </p>
      </div>
      <Dialog.Root>
        <Dialog.Trigger>
          <Button><Pencil /> edit</Button>
        </Dialog.Trigger>
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>Edit Business</Dialog.Title>
            <Dialog.Description>
              Update details of this business
            </Dialog.Description>
          </Dialog.Header>
          <div class="grid gap-2">
            <Label for="name">Business Name</Label>
            <Input id="name" type="text" bind:value={business.name} />
          </div>
          <div class="grid gap-2 mt-2">
            <Label for="shortCode">Short Code</Label>
            <Input
              id="shortCode"
              type="text"
              bind:value={business.short_code}
            />
          </div>
          <div>
            <Button onclick={handleUpdateBusiness} class="mt-4"
              ><Save /> Update Business</Button
            >
          </div>
        </Dialog.Content>
      </Dialog.Root>
      <Button
        onclick={handleDeleteBusiness}
        class="mt-4 ml-2"
        variant="destructive"><Trash /> Delete Business</Button
      >
    </div>
    <Separator />
    <Tabs.Root value="account" class="">
      <Tabs.List>
        <Tabs.Trigger value="account"><WalletMinimal /> Accounts</Tabs.Trigger>
        <Tabs.Trigger value="projects"><ChevronsLeftRightEllipsis /> Projects</Tabs.Trigger>
        <Tabs.Trigger value="transactions"><DollarSign /> Transactions</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="account">
        <h3 class="text-lg font-medium mt-6">Associated Accounts</h3>
        <div class="grid gap-4 md:grid-cols-2">
          <PaybillAccounts {paybillAccounts} {businessId} on:refresh={loadBusinessDetails} />
          <TillAccounts {tillAccounts} {businessId} on:refresh={loadBusinessDetails} />
        </div>
      </Tabs.Content>
      <Tabs.Content value="projects">
        <h3 class="text-lg font-medium mt-6">Associated Projects</h3>
        <Projects {projects} {businessId} on:refresh={loadBusinessDetails} />
      </Tabs.Content>
      <Tabs.Content value="transactions">
        <h3 class="text-lg font-medium mt-6">Transactions</h3>
        <Table.Root>
          <Table.Header>
            <Table.Row>
              <Table.Head><ArrowRightLeft class="text-foreground/50"/></Table.Head>
              <Table.Head>Amount</Table.Head>
              <Table.Head class="font-bold">Txn Id</Table.Head>
              <Table.Head class="font-bold">From</Table.Head>
              <Table.Head class="font-bold">To</Table.Head>
              <Table.Head class="font-bold">Date</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each transactions as transaction}
              <Table.Row>
                <Table.Cell>
                  {#if transaction.direction == "Credit"}
                    <MoveDownLeft class="text-green-700" />
                  {:else}
                    <MoveUpRight class="text-red-500" />
                  {/if}
                </Table.Cell>
                <Table.Cell><b>{formatTransactionAmount(transaction.transaction_amount)}</b></Table.Cell>
                <Table.Cell><pre>{transaction.transaction_id}</pre></Table.Cell>
                <Table.Cell>{transaction.from_name}</Table.Cell>
                <Table.Cell>{transaction.to_name}</Table.Cell>
                <Table.Cell>{formatDate(transaction.transaction_date)}</Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      </Tabs.Content>
    </Tabs.Root>
  {:else}
    <p>Loading business details...</p>
  {/if}
</div>
