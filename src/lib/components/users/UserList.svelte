<script lang="ts">
  import { Phone, Wallet, Settings, User as UserIcon} from "lucide-svelte";
  import { formatAmount, getInitials } from "$lib/utils";
  import { getUsers, type UserDetails } from '$lib/api';
  import { onDestroy, onMount } from "svelte";
  import DiceBearAvatar from '$lib/components/ui/avatar/DiceBearAvatar.svelte';
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { toast } from "svelte-sonner";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  let users: UserDetails[] = $state([]);
  let listen_handler: UnlistenFn;
   
  async function fetchUsers() {
    try {
      users = await getUsers();
    } catch (err) {
      toast(`Failed to fetch users: ${err}`);
    }
  }

  onMount(async () => {
    await fetchUsers();
    listen_handler = await listen("user_created", (_) => {
      fetchUsers();
    });
  })

  onDestroy(()=>{
    listen_handler();
  })
</script>

<div class="w-full flex bg-secondary flex-col h-full">
  <div class="p-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-semibold flex items-center gap-2">
          <UserIcon size={20} />
          Sandbox Users
        </h2>
        <p class="text-sm mt-1">{users.length} users available</p>
      </div>
      <Popover.Root>
        <Popover.Trigger
          >
            <Settings size={20} />
          </Popover.Trigger
        >
        <Popover.Content class="w-80">
          <div class="grid gap-4">
            <div class="">
              <h3 class="font-medium text-gray-900 mb-2">Settings</h3>
              <div class="space-y-2">
                <button
                  class="w-full text-left px-3 py-2 text-sm rounded-md transition-colors duration-200"
                  onclick={()=> {}}
                >
                  Generate 5 More Users
                </button>
              </div>
            </div>
          </div>
        </Popover.Content>
      </Popover.Root>
    </div>
  </div>
  <ScrollArea class="flex-1 min-h-0">
    {#each users as user}
      <a
        class="w-full block p-4 text-left border-b duration-200"
        href="/users/{user.id}"
      >
        <div class="flex gap-3 items-center">
          <div class="w-16 h-16">
            <DiceBearAvatar seed={`${user.id}-${user.name}`} fallback={getInitials(user.name)} />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <h3 class="font-medium truncate">{user.name}</h3>
            </div>
            <div class="flex items-center gap-2 mt-1">
              <Phone size={12} class="" />
              <span class="text-sm">{user.phone}</span>
            </div>
            <div class="flex items-center gap-2 mt-1">
              <Wallet size={12} class="" />
              <span class="text-sm font-medium text-green-600">{formatAmount(user.balance)}</span>
            </div>
          </div>
        </div>
      </a>
    {/each}
  </ScrollArea>
</div>
