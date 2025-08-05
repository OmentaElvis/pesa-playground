<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { deleteProject, SimulationMode, type ProjectDetails } from "$lib/api";
  import { Clock, Plus, Settings, Trash2, Users } from "lucide-svelte";
  import { Badge } from "$lib/components/ui/badge/index.js";


  export let projects: ProjectDetails[] = [];
  export let businessId: number;

  function getSimulationModeColor(mode: SimulationMode) {
    switch (mode) {
      case SimulationMode.AlwaysSuccess: return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
      case SimulationMode.AlwaysFail: return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
      case SimulationMode.Random: return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
      case SimulationMode.Realistic: return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
    }
  }

  async function removeProject(id: number) {
    await deleteProject(id);
  }
</script>

<!-- Projects Grid -->
{#if projects.length > 0}
  <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
    {#each projects as project}
      <Card class="hover:shadow-lg transition-shadow duration-200">
        <CardHeader class="pb-3">
          <div class="flex justify-between items-start">
            <div class="space-y-1">
              <CardTitle class="text-lg font-semibold">{project.name}</CardTitle
              >
              <div class="flex items-center gap-2">
                <Badge
                  class={getSimulationModeColor(project.simulation_mode)}
                  variant="outline"
                >
                  {project.simulation_mode.replace("-", " ")}
                </Badge>
              </div>
            </div>
            <div class="flex gap-1">
              <Button
                size="sm"
                variant="ghost"
                href={`/projects/${project.id}/settings`}
              >
                <Settings class="h-4 w-4" />
              </Button>
              <AlertDialog.Root>
                <AlertDialog.Trigger>
                  <Trash2 class="h-4 w-4 text-destructive hover:text-destructive" />
                </AlertDialog.Trigger>
                <AlertDialog.Content>
                  <AlertDialog.Header>
                    <AlertDialog.Title>Delete Project</AlertDialog.Title>
                    <AlertDialog.Description>
                      This action cannot be undone. This will permanently delete your project.
                    </AlertDialog.Description>
                  </AlertDialog.Header>
                  <AlertDialog.Footer>
                    <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                    <AlertDialog.Action  onclick={()=> removeProject(project.id)}>Continue</AlertDialog.Action>
                  </AlertDialog.Footer>
                </AlertDialog.Content>
              </AlertDialog.Root>
            </div>
          </div>
        </CardHeader>

        <CardContent class="space-y-4">
          <!-- Actions -->
          <div class="flex gap-2 pt-2">
            <Button size="sm" href={`/projects/${project.id}`} class="flex-1">
              View project
            </Button>
          </div>
        </CardContent>
      </Card>
    {/each}
  </div>

  <!-- Empty State for New Project -->
  <Card
    class="border-dashed border-2 hover:border-primary/50 transition-colors"
  >
    <CardContent class="flex flex-col items-center justify-center py-12">
      <div class="text-center space-y-4">
        <div
          class="mx-auto w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center"
        >
          <Plus class="h-8 w-8 text-primary/60" />
        </div>
        <div>
          <h3 class="font-semibold text-foreground">Create Another Project</h3>
          <p class="text-sm text-muted-foreground mt-1">
            Set up a new testing environment in seconds
          </p>
        </div>
        <Button href="/projects/new/{businessId}" variant="outline" class="gap-2">
          <Plus class="h-4 w-4" />
          New Project
        </Button>
      </div>
    </CardContent>
  </Card>
{:else}
  <!-- Empty State -->
  <Card class="border-dashed border-2">
    <CardContent class="flex flex-col items-center justify-center py-20">
      <div class="text-center space-y-6">
        <div
          class="mx-auto w-20 h-20 bg-primary/10 rounded-full flex items-center justify-center"
        >
          <Users class="h-10 w-10 text-primary/60" />
        </div>
        <div>
          <h2 class="text-2xl font-semibold text-foreground">
            No Projects Yet
          </h2>
          <p class="text-muted-foreground mt-2 max-w-md">
            Create your first project to start testing M-Pesa integrations. It
            takes less than a minute to set up.
          </p>
        </div>
        <Button href="/projects/new/{businessId}" size="lg" class="gap-2">
          <Plus class="h-5 w-5" />
          Create Your First Project
        </Button>
      </div>
    </CardContent>
  </Card>
{/if}
