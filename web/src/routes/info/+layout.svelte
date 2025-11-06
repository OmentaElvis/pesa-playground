<script lang="ts">
  import { page } from '$app/state';
  import { buttonVariants } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { cn } from '$lib/utils';

  export let data;

  let searchTerm = '';
  $: filteredArticles = data.articles.filter(article => 
    article.title.toLowerCase().includes(searchTerm.toLowerCase())
  );
</script>

<div class="grid grid-cols-12 h-full">
  <div class="col-span-3 border-r bg-muted/40">
    <div class="flex flex-col h-full">
      <div class="p-4">
        <h2 class="font-bold text-lg">Docs</h2>
        <p class="text-sm text-muted-foreground">PPG in-app guide</p>
      </div>
      <div class="p-4 pt-0">
        <Input placeholder="Search articles..." bind:value={searchTerm} />
      </div>
      <div class="flex flex-col space-y-1 p-4 pt-0 overflow-y-auto">
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
  <div class="col-span-9 p-6 overflow-y-auto">
    <slot />
  </div>
</div>
