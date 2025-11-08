<script lang="ts">
  import { cn } from '$lib/utils';

  let { images, class: className = '' } = $props<{
    images: string[];
    class?: string;
  }>();
</script>

<div class={cn('grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4', className)}>
  {#each images as src}
    <div class="flex items-center justify-center h-full border rounded-lg p-2">
      <img {src} alt="Documentation screenshot" class="max-h-full max-w-full object-contain shadow-lg rounded-lg" />
    </div>
  {/each}
</div>
