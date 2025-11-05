<script lang="ts">
  import { LoaderCircle } from '@lucide/svelte';
  export let show: boolean = true;
</script>

{#if show}
  <div class="fixed inset-0 flex items-center justify-center bg-secondary z-[9999] transition-opacity duration-500 ease-out">
    <div class="flex flex-col items-center gap-4">
      <img src="/pesaplayground_logo.svg" alt="Pesa Playground Logo" class="max-w-[80%] max-h-[80vh]" />

      <LoaderCircle class="animate-spin size-12 dark:text-green-600" />
    </div>
  </div>
{/if}
