<script lang="ts">
  import { sidebarStore } from '$lib/stores/sidebarStore';
  import { Button } from '$lib/components/ui/button';
  import { PanelLeftClose } from 'lucide-svelte';

  const { widgets, activeWidget, isCollapsed, lastAddedWidgetId, acknowledgeNewWidget } = sidebarStore;
</script>

<div class="flex h-full w-full bg-background">
  <!-- Icon Strip -->
  <div class="flex flex-col items-center w-[50px] p-2 border-l border-r bg-muted/50">
    {#each $widgets as widget (widget.id)}
      <div
        class:new-widget-dance={$lastAddedWidgetId === widget.id}
        on:animationend={acknowledgeNewWidget}
      >
        <Button
          variant={$activeWidget?.id === widget.id && !$isCollapsed ? 'secondary' : 'ghost'}
          size="icon"
          title={widget.title}
          onclick={() => sidebarStore.setActiveWidget(widget.id)}
          class="mb-2 cursor-pointer"
        >
          <widget.icon class="w-5 h-5" />
        </Button>
      </div>
    {/each}
  </div>

  <!-- Content Panel -->
  {#if !$isCollapsed && $activeWidget}
    <div class="flex-1 flex flex-col w-[calc(100%-50px)]">
      <div class="flex items-center justify-between p-2 border-b h-[45px]">
        <h3 class="font-semibold text-sm">{$activeWidget.title}</h3>
        <Button
          variant="ghost"
          size="icon"
          onclick={sidebarStore.collapse}
          title="Collapse"
        >
          <PanelLeftClose class="w-5 h-5" />
        </Button>
      </div>
      <div class="flex-1 py-2 overflow-y-auto">
        <svelte:component
          this={$activeWidget.component}
          {...$activeWidget.props || {}}
        />
      </div>
    </div>
  {/if}
</div>

<style>
    @keyframes icon-dance {
      0% { transform: scale(1) rotate(0); }
      20% { transform: scale(1.2) rotate(-10deg); }
      40% { transform: scale(1.2) rotate(10deg); }
      60% { transform: scale(1.2) rotate(-10deg); }
      80% { transform: scale(1.2) rotate(10deg); }
      100% { transform: scale(1) rotate(0); }
    }

    .new-widget-dance {
      animation: icon-dance 0.5s ease-in-out;
    }
</style>
