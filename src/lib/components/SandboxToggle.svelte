<script lang="ts">
  import { Play, Pause, Cog, Loader2, CircleX } from "lucide-svelte";
  import { sandboxStatus } from "$lib/stores/sandboxStatus";
  import {
    sandboxStatus as apiSandboxStatus,
    startSandbox,
    stopSandbox,
  } from "$lib/api";
  import { onDestroy, onMount } from "svelte";

  export let id: number;
  let status: "off" | "starting" | "on" | "error" = "off";
  let port: number | null = null;
  let error: string | null = null;
  let pollInterval: ReturnType<typeof setInterval>;

  function setStatus(s: "off" | "starting" | "on" | "error") {
    status = s;
    $sandboxStatus = s;
  }

  async function refresh() {
    try {
      const res = await apiSandboxStatus(id);
      if (res.status === "on") {
        setStatus("on");
        port = res.port;
        error = null;
      } else if (res.status == "error") {
        error = res.error || "The sandbox encountered unknown error";
        setStatus("error");
        port = null;
      } else {
        setStatus("off");
        port = null;
      }
    } catch (e) {
      setStatus("error");
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggle() {
    if (status === "starting") return;

    if (status === "on") {
      try {
        await stopSandbox(id);
        setStatus("off");
        port = null;
        error = null;
      } catch (e) {
        setStatus("error");
        error = e instanceof Error ? e.message : String(e);
      }
    } else {
      setStatus("starting");
      error = null;
      try {
        const addr = await startSandbox(id);
        port = parseInt(addr.split(":").pop() || "0");
        setTimeout(() => refresh(), 2000);
        // status = "on";
      } catch (e) {
        setStatus("error");
        error = e instanceof Error ? e.message : String(e);
      }
    }
  }

  onMount(() => {
    refresh();
    pollInterval = setInterval(() => {
      refresh();
    }, 10000);
  });

  onDestroy(() => {
    clearInterval(pollInterval);
  });
</script>

<div class="flex gap-4 items-center">
  <button
    class="w-12 h-6 rounded-full cursor-pointer transition-colors duration-300 relative"
    class:bg-green-700={status === "on"}
    class:bg-gray-400={status === "off"}
    class:bg-yellow-400={status === "starting"}
    class:bg-red-400={status === "error"}
    on:click={toggle}
  >
    <div
      class="w-6 h-6 bg-white rounded-full shadow-md absolute top-0 transition-all duration-300 flex items-center justify-center"
      class:translate-x-8={status === "on"}
    >
      {#if status === "starting"}
        <Loader2 class="w-4 h-4 animate-spin text-yellow-600" />
      {:else if status === "on"}
        <Cog class="animate-spin w-4 h-4 text-gray-700" />
      {:else if status === "error"}
        <CircleX class="w-4 w-4 text-red-500" />
      {:else}
        <Play class="w-4 h-4 text-gray-500" />
      {/if}
    </div>
  </button>
  {#if status === "on"}
    <div class="text-xs text-green-700 font-bold">Running on port {port}</div>
  {:else if status === "starting"}
    <div class="text-xs text-yellow-700">Starting...</div>
  {:else if status === "error" || error}
    <div class="text-xs text-red-700">Error: {error}</div>
  {/if}
</div>
