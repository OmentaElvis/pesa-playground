<script lang="ts">
	import { cn } from '$lib/utils';

	let { images, class: className = '' } = $props<{
		images: string[];
		class?: string;
	}>();
</script>

<div class={cn('grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3', className)}>
	{#each images as src}
		<div class="flex h-full items-center justify-center rounded-lg border p-2">
			<img
				{src}
				alt="Documentation screenshot"
				class="max-h-full max-w-full rounded-lg object-contain shadow-lg"
			/>
		</div>
	{/each}
</div>
