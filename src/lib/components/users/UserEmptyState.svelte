<script>
  import { User, Plus, LoaderCircle } from "lucide-svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { generateUsers, createUser } from '$lib/api'
  import { toast } from "svelte-sonner";
	let generating = $state(false);

  async function genUsers() {
    generating = true;
    try {
      let users = await generateUsers(10);
      for (let user of users) {
        let id = await createUser(user.name, user.phone, user.balance, user.pin);
      }
    } catch (err) {
      toast(`Failed to create user: ${err}`)      
    } finally {
      generating = false;
    }
  }
</script>

<div class="flex-1 flex flex-col items-center justify-center p-8">
  <div class="text-center max-w-md">
    <div class="w-16 h-16 bg-gray-600 rounded-full flex items-center justify-center mx-auto mb-6">
      <User size={32} class="text-white" />
    </div>
    <h1 class="text-2xl font-bold mb-4">Sandbox Users</h1>
    <p class="mb-8 text-sm">
      Select a user from the sidebar to view their transaction history, or generate new users.
    </p>
    <Button
      variant="outline"
      class="font-medium py-3 px-6 rounded-lg flex items-center gap-2 mx-auto cursor-pointer"
      onclick={genUsers}
      disabled={generating}
    >
      {#if generating}
        <LoaderCircle class="animate-spin" />
      {:else}
        <Plus size={20} />
      {/if}
      Generate New Users
    </Button>
  </div>
</div>
