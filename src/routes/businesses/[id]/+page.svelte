<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import {
    getBusiness,
    updateBusiness,
    deleteBusiness,
    createPaybillAccount,
    createTillAccount,
    createProject,
    getPaybillAccountsByBusinessId,
    getTillAccountsByBusinessId,
    getProjectsByBusinessId,
    type CreatePaybillAccountData,
    type CreateTillAccountData,
  } from "$lib/api";
  import { goto } from "$app/navigation";
  import { PlusCircle } from "lucide-svelte";

  let business: any = null;
  let paybillAccounts: any[] = [];
  let tillAccounts: any[] = [];
  let projects: any[] = [];

  let newPaybill: CreatePaybillAccountData = {
    business_id: business,
    initial_balance: 0,
    paybill_number: 0,
  };
  
  let newTill: CreateTillAccountData = {
    business_id: 0,
    initial_balance: 0,
    store_number: 0,
    till_number: 0,
  };

  let newProjectName: string = "";
  let newProjectCallbackUrl: string = "";
  let newProjectPrefix: string = "";
  let newProjectSimulationMode: boolean = false;
  let newProjectStkDelay: number = 0;

  let businessId: number;

  $: if ($page.params.id) {
    businessId = parseInt($page.params.id);
  }

  async function loadBusinessDetails() {
    if (businessId) {
      business = await getBusiness(businessId);
      paybillAccounts = await getPaybillAccountsByBusinessId(businessId);
      tillAccounts = await getTillAccountsByBusinessId(businessId);
      projects = await getProjectsByBusinessId(businessId);
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

  async function handleCreatePaybillAccount() {
    if (businessId) {
      newPaybill.business_id = businessId;
      await createPaybillAccount(newPaybill);
      newPaybill = {
        business_id: 0,
        initial_balance: 0,
        paybill_number: 0,
      };
      await loadBusinessDetails();
    }
  }

  async function handleCreateTillAccount() {
    if (businessId) {
      newTill.business_id = businessId;
      await createTillAccount(newTill);
      newTill = {
        business_id: 0,
        initial_balance: 0,
        store_number: 0,
        till_number: 0,
      };
      await loadBusinessDetails();
    }
  }

  async function handleCreateProject() {
    if (businessId) {
      await createProject({
        business_id: businessId,
        name: newProjectName,
        callback_url: newProjectCallbackUrl,
        prefix: newProjectPrefix,
        simulation_mode: newProjectSimulationMode,
        stk_delay: newProjectStkDelay,
      });
      newProjectName = "";
      newProjectCallbackUrl = "";
      newProjectPrefix = "";
      newProjectSimulationMode = false;
      newProjectStkDelay = 0;
      await loadBusinessDetails();
    }
  }

  onMount(() => {
    loadBusinessDetails();
  });
</script>

<div class="space-y-6">
  {#if business}
    <div>
      <h3 class="text-lg font-medium">Business Details</h3>
      <p class="text-sm text-muted-foreground">Manage business information.</p>
    </div>
    <Separator />

    <Card>
      <CardHeader>
        <CardTitle>Edit Business</CardTitle>
        <CardDescription>Update the details of this business.</CardDescription>
      </CardHeader>
      <CardContent>
        <div class="grid gap-2">
          <Label for="name">Business Name</Label>
          <Input id="name" type="text" bind:value={business.name} />
        </div>
        <div class="grid gap-2 mt-2">
          <Label for="shortCode">Short Code</Label>
          <Input id="shortCode" type="text" bind:value={business.short_code} />
        </div>
        <Button onclick={handleUpdateBusiness} class="mt-4"
          >Update Business</Button
        >
        <Button
          onclick={handleDeleteBusiness}
          class="mt-4 ml-2"
          variant="destructive">Delete Business</Button
        >
      </CardContent>
    </Card>

    <h3 class="text-lg font-medium mt-6">Associated Accounts</h3>
    <Separator />

    <div class="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Paybill Accounts</CardTitle>
          <CardDescription
            >Accounts associated with this business.</CardDescription
          >
        </CardHeader>
        <CardContent>
          {#if paybillAccounts.filter((acc) => acc.business_id === businessId).length > 0}
            <ul>
              {#each paybillAccounts.filter((acc) => acc.business_id === businessId) as account}
                <li>
                  {account.paybill_number} - Balance: {account.balance} - Created:
                  {account.created_at}
                </li>
              {/each}
            </ul>
          {:else}
            <p>No paybill accounts found for this business.</p>
          {/if}
          <div class="mt-4 space-y-2">
            <h4 class="font-medium">Add New Paybill Account</h4>
            <Label for="newPaybillNumber">Paybill Number</Label>
            <Input
              id="newPaybillNumber"
              type="number"
              bind:value={newPaybill.paybill_number}
            />
            <Label for="newPaybillInitialBalance">Initial Balance</Label>
            <Input
              id="newPaybillInitialBalance"
              type="number"
              bind:value={newPaybill.initial_balance}
            />
            <Label for="newPaybillAccountValidationRegex"
              >Account Validation Regex</Label
            >
            <Input
              id="newPaybillAccountValidationRegex"
              type="text"
              bind:value={newPaybill.account_validation_regex}
            />
            <Label for="newPaybillValidationUrl">Validation URL</Label>
            <Input
              id="newPaybillValidationUrl"
              type="text"
              bind:value={newPaybill.confirmation_url}
            />
            <Label for="newPaybillConfirmationUrl">Confirmation URL</Label>
            <Input
              id="newPaybillConfirmationUrl"
              type="text"
              bind:value={newPaybill.confirmation_url}
            />
            <Button onclick={handleCreatePaybillAccount} class="mt-2">
              <PlusCircle class="mr-2 h-4 w-4" /> Add Paybill Account
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Till Accounts</CardTitle>
          <CardDescription
            >Till accounts associated with this business.</CardDescription
          >
        </CardHeader>
        <CardContent>
          {#if tillAccounts.filter((acc) => acc.business_id === businessId).length > 0}
            <ul>
              {#each tillAccounts.filter((acc) => acc.business_id === businessId) as account}
                <li>
                  {account.till_number} - Balance: {account.balance} - Created: {account.created_at}
                </li>
              {/each}
            </ul>
          {:else}
            <p>No till accounts found for this business.</p>
          {/if}
          <div class="mt-4 space-y-2">
            <h4 class="font-medium">Add New Till Account</h4>
            <Label for="newTillNumber">Till Number</Label>
            <Input
              id="newTillNumber"
              type="number"
              bind:value={newTill.till_number}
            />
            <Label for="newTillInitialBalance">Initial Balance</Label>
            <Input
              id="newTillInitialBalance"
              type="number"
              bind:value={newTill.initial_balance}
            />
            <Label for="newTillStoreNumber">Store Number</Label>
            <Input
              id="newTillStoreNumber"
              type="number"
              bind:value={newTill.store_number}
            />
            <Label for="newTillLocationDescription">Location Description</Label>
            <Input
              id="newTillLocationDescription"
              type="text"
              bind:value={newTill.location_description}
            />
            <Button onclick={handleCreateTillAccount} class="mt-2">
              <PlusCircle class="mr-2 h-4 w-4" /> Add Till Account
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>

    <h3 class="text-lg font-medium mt-6">Associated Projects</h3>
    <Separator />

    <Card>
      <CardHeader>
        <CardTitle>Projects</CardTitle>
        <CardDescription
          >Projects associated with this business.</CardDescription
        >
      </CardHeader>
      <CardContent>
        {#if projects.length > 0}
          <ul>
            {#each projects as project}
              <li>{project.name}</li>
            {/each}
          </ul>
        {:else}
          <p>No projects found for this business.</p>
        {/if}
        <div class="mt-4 space-y-2">
          <h4 class="font-medium">Add New Project</h4>
          <Label for="newProjectName">Project Name</Label>
          <Input id="newProjectName" type="text" bind:value={newProjectName} />
          <Label for="newProjectCallbackUrl">Callback URL</Label>
          <Input
            id="newProjectCallbackUrl"
            type="text"
            bind:value={newProjectCallbackUrl}
          />
          <Label for="newProjectPrefix">Prefix</Label>
          <Input
            id="newProjectPrefix"
            type="text"
            bind:value={newProjectPrefix}
          />
          <Label for="newProjectSimulationMode">Simulation Mode</Label>
          <input
            id="newProjectSimulationMode"
            type="checkbox"
            bind:checked={newProjectSimulationMode}
            class="ml-2"
          />
          <Label for="newProjectStkDelay">STK Delay</Label>
          <Input
            id="newProjectStkDelay"
            type="number"
            bind:value={newProjectStkDelay}
          />
          <Button onclick={handleCreateProject} class="mt-2">
            <PlusCircle class="mr-2 h-4 w-4" /> Add Project
          </Button>
        </div>
      </CardContent>
    </Card>
  {:else}
    <p>Loading business details...</p>
  {/if}
</div>
