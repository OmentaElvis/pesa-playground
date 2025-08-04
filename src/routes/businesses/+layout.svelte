<script lang="ts">
  import { Separator } from '$lib/components/ui/separator';
  import { page } from '$app/stores';
  import { cn } from '$lib/utils';

  let sidebarNavItems = [
    {
      title: 'Overview',
      href: '/businesses'
    }
  ];

  $: currentPath = $page.url.pathname;
</script>

<div class="space-y-6 p-10 pb-16 md:block">
  <div class="space-y-0.5">
    <h2 class="text-2xl font-bold tracking-tight">Businesses</h2>
    <p class="text-muted-foreground">
      Manage your businesses, accounts, and projects.
    </p>
  </div>
  <Separator class="my-6" />
  <div class="flex flex-col space-y-8 lg:flex-row lg:space-x-12 lg:space-y-0">
    <aside class="-mx-4 lg:w-1/5">
      <nav class="flex space-x-2 lg:flex-col lg:space-x-0 lg:space-y-1">
        {#each sidebarNavItems as item}
          <a
            href={item.href}
            class={cn(
              'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium hover:bg-muted hover:text-foreground',
              currentPath === item.href ? 'bg-muted text-foreground' : 'text-muted-foreground'
            )}
          >
            {item.title}
          </a>
        {/each}
      </nav>
    </aside>
    <div class="flex-1 lg:max-w-2xl">
      <slot />
    </div>
  </div>
</div>