<script lang="ts">
  import { Separator } from '$lib/components/ui/separator';

  export let data;
</script>

<div class="prose dark:prose-invert max-w-none">
  <h1>{data.article.title}</h1>
  <Separator class="my-4" />
  <svelte:component this={data.article.component} />
</div>
