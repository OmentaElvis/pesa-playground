<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { 
    Plus, 
    Users, 
    Activity, 
    Settings, 
    Trash2,
    Clock
  } from 'lucide-svelte';
    import { getProjects, SimulationMode, type ProjectSummary } from "$lib/api";
    import { onMount } from "svelte";

  // Mock data - replace with your actual data source
  let projects: ProjectSummary[]  = [];

  onMount(async ()=> {
    projects = await getProjects();    
  })



</script>

<main class="container mx-auto p-6 space-y-6">
  <!-- Header -->
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-3xl font-bold tracking-tight text-foreground">Projects</h1>
      <p class="text-muted-foreground mt-1">Manage your M-Pesa testing environments</p>
    </div>
    <Button href="/projects/new" class="gap-2">
      <Plus class="h-4 w-4" />
      New Project
    </Button>
  </div>
</main>
