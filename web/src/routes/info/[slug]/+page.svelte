<script lang="ts">
  import { Separator } from '$lib/components/ui/separator';

  export let data;
</script>

<div class="prose dark:prose-invert max-w-none prose-img:mx-auto prose-img:shadow-lg prose-img:rounded-lg max-w-none prose-a:text-blue-600 px-8">
  <h1>{data.article.title}</h1>
  <Separator class="my-4" />
  <svelte:component this={data.article.component} />
</div>
