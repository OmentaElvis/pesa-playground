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
  import { createBusiness, getBusinesses } from "$lib/api";
  import { onMount } from "svelte";
  import { Plus } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import * as Dialog from "$lib/components/ui/dialog";

  let businesses: any[] = [];
  let newBusinessName: string = "";
  let newBusinessShortCode: string = "";
  let showCreateBusinessDialog: boolean = false;

  async function loadBusinesses() {
    businesses = await getBusinesses();
  }

  async function handleCreateBusiness() {
    const business = await createBusiness({
      name: newBusinessName,
      short_code: newBusinessShortCode,
    });
    if (business) {
      await loadBusinesses();
      newBusinessName = "";
      newBusinessShortCode = "";
      showCreateBusinessDialog = false;
    }
  }

  onMount(() => {
    loadBusinesses();
  });
</script>

<div class="space-y-6 p-6 md:block">
  <div class="space-y-0.5">
    <div class="flex mb-10 justify-between space-between items-center w-full">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">Businesses</h2>
        <p class="text-muted-foreground">
          Manage your businesses, accounts, and projects.
        </p>
      </div>
      <Dialog.Root bind:open={showCreateBusinessDialog}>
        <Dialog.Trigger>
          <Button>
            <Plus />
            <span class="">Create New Business</span>
          </Button>
        </Dialog.Trigger>
        <Dialog.Content class="sm:max-w-[425px]">
          <Dialog.Header>
            <Dialog.Title>Create New Business</Dialog.Title>
            <Dialog.Description>
              Add a new business to your system.
            </Dialog.Description>
          </Dialog.Header>
          <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
              <Label for="name" class="text-right">Business Name</Label>
              <Input
                id="name"
                type="text"
                placeholder="Business Name"
                bind:value={newBusinessName}
                class="col-span-3"
              />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
              <Label for="shortCode" class="text-right">Short Code</Label>
              <Input
                id="shortCode"
                type="text"
                placeholder="Short Code"
                bind:value={newBusinessShortCode}
                class="col-span-3"
              />
            </div>
          </div>
          <Dialog.Footer>
            <Button onclick={handleCreateBusiness}>Create Business</Button>
          </Dialog.Footer>
        </Dialog.Content>
      </Dialog.Root>
    </div>
    <div class="space-y-6">
      <div class="grid gap-4 md:grid-cols-3 lg:grid-cols-4">
        {#each businesses as business}
          <Card>
            <CardHeader>
              <CardTitle>{business.name}</CardTitle>
              <CardDescription>{business.short_code}</CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                onclick={() => goto(`/businesses/${business.id}`)}
                class="mt-4">View Details</Button
              >
            </CardContent>
          </Card>
        {/each}
      </div>
    </div>
  </div>
</div>
