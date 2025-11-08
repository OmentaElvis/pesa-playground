<script lang="ts">
	import { Separator } from '$lib/components/ui/separator';

	export let data;
</script>

<div
	class="prose max-w-none px-8 dark:prose-invert prose-a:text-blue-600 prose-img:mx-auto prose-img:rounded-lg prose-img:shadow-lg"
>
	<h1>{data.article.title}</h1>
	<Separator class="my-4" />
	<svelte:component this={data.article.component} />
</div>
