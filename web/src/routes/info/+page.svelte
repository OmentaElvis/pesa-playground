<script lang="ts">
  import Separator from '$lib/components/ui/separator/separator.svelte';

  export let data;
</script>

<div class="p-4">
  <div class="my-8">
    <img src="/pesaplayground_logo.png" alt="Pesa Playground Logo" class="max-w-xs mx-auto" />
    <span>
      v{data.appInfo.version}
    </span>
  </div>
  <h2 class="font-bold text-xl">Pesa Playground Documentation</h2>
  <Separator class="my-4" />
</div>
