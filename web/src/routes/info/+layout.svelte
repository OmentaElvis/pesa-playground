<script lang="ts">
	import { page } from '$app/state';
	import { buttonVariants } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { cn } from '$lib/utils';

	export let data;

	let searchTerm = '';
	$: filteredArticles = data.articles.filter((article) =>
		article.title.toLowerCase().includes(searchTerm.toLowerCase())
	);
</script>

<div class="grid h-full grid-cols-12">
	<div class="col-span-3 border-r bg-muted/40">
		<div class="flex h-full flex-col">
			<div class="p-4">
				<h2 class="text-lg font-bold">Docs</h2>
				<p class="text-sm text-muted-foreground">PPG in-app guide</p>
			</div>
			<div class="p-4 pt-0">
				<Input placeholder="Search articles..." bind:value={searchTerm} />
			</div>
			<div class="flex flex-col space-y-1 overflow-y-auto p-4 pt-0">
				<a
					href="/info"
					class={cn(
						buttonVariants({ variant: 'ghost' }),
						'w-full justify-start',
						page.url.pathname === '/info' ? 'bg-muted hover:bg-muted' : ''
					)}
				>
					Introduction
				</a>
				{#each filteredArticles as article}
					<a
						href={`/info/${article.slug}`}
						class={cn(
							buttonVariants({ variant: 'ghost' }),
							'w-full justify-start',
							page.url.pathname === `/info/${article.slug}` ? 'bg-muted hover:bg-muted' : ''
						)}
					>
						{article.title}
					</a>
				{/each}
			</div>
		</div>
	</div>
	<div class="col-span-9 overflow-y-auto p-6">
		<slot />
	</div>
</div>
