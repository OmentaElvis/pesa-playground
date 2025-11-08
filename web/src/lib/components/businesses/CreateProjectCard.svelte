<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Plus } from "lucide-svelte";

  export let businessId: number;
</script>

<Card
  class="border-dashed border-2 hover:border-primary/50 transition-colors h-full"
>
  <CardContent class="flex flex-col items-center justify-center py-12 h-full">
    <div class="text-center space-y-4">
      <div
        class="mx-auto w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center"
      >
        <Plus class="h-8 w-8 text-primary/60" />
      </div>
      <div>
        <h3 class="font-semibold text-foreground">Create a New Project</h3>
        <p class="text-sm text-muted-foreground mt-1">
          Set up a new testing environment.
        </p>
      </div>
      <Button href="/projects/new/{businessId}" variant="outline" class="gap-2">
        <Plus class="h-4 w-4" />
        New Project
      </Button>
    </div>
  </CardContent>
</Card>
