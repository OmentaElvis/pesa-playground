<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { users } from "$lib/stores/users";

  const dispatch = createEventDispatcher();

  function generateUsers() {
    const names = [
      "Alice Cooper",
      "Bob Wilson",
      "Carol Davis",
      "Daniel Miller",
      "Emma Garcia",
    ];
    const newUsers = names.map((name, index) => ({
      id: $users.length + index + 1,
      name,
      phone: `+25470${Math.floor(Math.random() * 9000000) + 1000000}`,
      balance: Math.floor(Math.random() * 50000) + 1000,
      avatar: name
        .split(" ")
        .map((n) => n[0])
        .join(""),
      lastActive: `${Math.floor(Math.random() * 24)} hours ago`,
    }));
    users.update((currentUsers) => [...currentUsers, ...newUsers]);
  }

  function clearAllUsers() {
    users.set([]);
    dispatch("close");
  }

  function randomizeBalances() {
    users.update((currentUsers) =>
      currentUsers.map((u) => ({
        ...u,
        balance: Math.floor(Math.random() * 50000) + 1000,
      }))
    );
    dispatch("close");
  }
</script>

<div class="mt-4 p-3 rounded-lg border">
  <h3 class="font-medium text-gray-900 mb-2">Settings</h3>
  <div class="space-y-2">
    <button
      class="w-full text-left px-3 py-2 text-sm rounded-md transition-colors duration-200"
      on:click={clearAllUsers}
    >
      Clear All Users
    </button>
    <button
      class="w-full text-left px-3 py-2 text-sm rounded-md transition-colors duration-200"
      on:click={generateUsers}
    >
      Generate 5 More Users
    </button>
    <button
      class="w-full text-left px-3 py-2 text-sm rounded-md transition-colors duration-200"
      on:click={randomizeBalances}
    >
      Randomize Balances
    </button>
  </div>
</div>
