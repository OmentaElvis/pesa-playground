<script lang="ts">
  import * as HoverCard from "$lib/components/ui/hover-card";
  import { HelpCircle } from "lucide-svelte";
  import { getArticle } from "$lib/documentation";
  import { buttonVariants } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";

  export let slug: string;

  const article = getArticle(slug);
</script>

{#if article}
  <HoverCard.Root openDelay={200}>
    <HoverCard.Trigger
      href={`/info/${slug}`}
      class={cn(buttonVariants({ variant: "ghost", size: "icon" }), "h-6 w-6")}
    >
      <HelpCircle class="h-4 w-4 text-muted-foreground" />
    </HoverCard.Trigger>
    <HoverCard.Content class="text-sm w-80">
      <h4 class="font-bold mb-2">{article.title}</h4>
      <p>{article.summary || "No summary available."}</p>
      <p class="mt-2 text-xs text-muted-foreground">
        Click icon for full article.
      </p>
    </HoverCard.Content>
  </HoverCard.Root>
{/if}
