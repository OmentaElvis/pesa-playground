<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import * as Select from "$lib/components/ui/select/index";
  import { Slider } from "$lib/components/ui/slider";
  import {
    Globe,
    Code,
    Timer,
    Tag,
    CheckCircle,
    LoaderCircle,
    ArrowLeft,
  } from "lucide-svelte";
  import { getProject, SimulationMode, updateProject } from "$lib/api";
  import type { ProjectDetails, UpdateProjectData } from "$lib/api";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { toast } from "svelte-sonner";


  let id = $derived(Number(page.params.id));
  let data: ProjectDetails = $derived({
    id: id,
    consumer_key: "",
    consumer_secret: "",
    name: "",
    passkey: "",
    simulation_mode: SimulationMode.Realistic,
    stk_delay: 0,
    callback_url: "",
    created_at: "",
    prefix: "",
    business_id: 0,
  });

  // svelte-ignore state_referenced_locally
  let originalData: ProjectDetails = data;
  let loading = $state(true);
  let saving = $state(false);
  let error = $state("");

  onMount(async () => {
    try {
      loading = true;
      data = await getProject(id);
      originalData = { ...data };
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load project";
    } finally {
      loading = false;
    }
  });

  async function handleSave() {
    if (!data.name?.trim()) {
      error = "Project name and shortcode are required";
      return;
    }

    try {
      saving = true;
      error = "";
      
      // Create update payload with only changed fields
      const updatePayload: UpdateProjectData = {};
      
      if (data.name !== originalData.name) {
        updatePayload.name = data.name;
      }
      if (data.callback_url !== originalData.callback_url) {
        updatePayload.callback_url = data.callback_url;
      }
      if (data.simulation_mode !== originalData.simulation_mode) {
        updatePayload.simulation_mode = data.simulation_mode;
      }
      if (data.stk_delay !== originalData.stk_delay) {
        updatePayload.stk_delay = data.stk_delay;
      }
      if (data.prefix !== originalData.prefix) {
        updatePayload.prefix = data.prefix;
      }

      if (Object.keys(updatePayload).length > 0) {
        await updateProject(id, updatePayload);
        originalData = { ...data };
      }

      toast.success("Project updated.");
      window.history.back();
      
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to update project";
    } finally {
      saving = false;
    }
  }

  function back() {
    window.history.back();
  }

  // Check if form has unsaved changes
  let hasChanges = $derived(
    data.name !== originalData?.name ||
    data.callback_url !== originalData?.callback_url ||
    data.simulation_mode !== originalData?.simulation_mode ||
    data.stk_delay !== originalData?.stk_delay ||
    data.prefix !== originalData?.prefix
  );

  // Form validation
  let isValid = $derived(
    data.name?.trim()
  );
</script>

<div class="min-h-screen bg-background p-6">
  <div class="max-w-2xl mx-auto space-y-8">
    <!-- Header -->
    <div class="flex">
      <Button variant="ghost" onclick={back}><ArrowLeft /></Button>
      <div class="text-center space-y-2 basis-full">
        <h1 class="text-3xl font-bold tracking-tight text-foreground">
          Update Project
        </h1>
        <p class="text-muted-foreground">
          Update sandbox testing environment
        </p>
      </div>
    </div>

    {#if loading}
      <Card class="shadow-lg">
        <CardContent class="flex items-center justify-center py-12">
          <LoaderCircle class="animate-spin h-8 w-8" />
          <span class="ml-2">Loading project...</span>
        </CardContent>
      </Card>
    {:else}
      <!-- Main Form Card -->
      <Card class="shadow-lg">
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Code class="h-5 w-5" />
            "{data.name}" Configuration
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-6">
          <!-- Error Message -->
          {#if error}
            <div class="bg-destructive/10 border border-destructive/20 rounded-md p-3">
              <p class="text-sm text-destructive">{error}</p>
            </div>
          {/if}

          <!-- Project Name -->
          <div class="space-y-2">
            <Label for="project-name" class="text-sm font-medium">
              Project Name
            </Label>
            <Input
              id="project-name"
              bind:value={data.name}
              placeholder="My Test App"
              class="w-full"
            />
            <p class="text-xs text-muted-foreground">
              A friendly name for your project
            </p>
          </div>

          <!-- Callback URL -->
          <div class="space-y-2">
            <Label
              for="callback-url"
              class="text-sm font-medium flex items-center gap-1"
            >
              <Globe class="h-4 w-4" />
              Callback URL
            </Label>
            <Input
              id="callback-url"
              bind:value={data.callback_url}
              placeholder="http://localhost:5001/callback"
              class="w-full"
            />
            <p class="text-xs text-muted-foreground">
              Where we'll send payment notifications
            </p>
          </div>

          <!-- Simulation Mode -->
          <div class="space-y-2">
            <Label class="text-sm font-medium flex items-center gap-1">
              <CheckCircle class="h-4 w-4" />
              Simulation Mode
            </Label>
            <Select.Root
              type="single"
              bind:value={data.simulation_mode}
              name="simulationMode"
            >
              <Select.SelectTrigger>
                {data.simulation_mode || "Select simulation mode"}
              </Select.SelectTrigger>
              <Select.Content>
                <Select.SelectItem value="always-success">
                  Always Success
                </Select.SelectItem>
                <Select.SelectItem value="always-fail">
                  Always Fail
                </Select.SelectItem>
                <Select.SelectItem value="random">
                  Random Success/Fail
                </Select.SelectItem>
                <Select.SelectItem value="realistic">
                  Realistic Simulation
                </Select.SelectItem>
              </Select.Content>
            </Select.Root>
            <p class="text-xs text-muted-foreground">
              How payment simulations should behave
            </p>
          </div>

          <!-- STK Delay -->
          <div class="space-y-4">
            <Label class="text-sm font-medium flex items-center gap-1">
              <Timer class="h-4 w-4" />
              STK Push Delay
            </Label>
            <div class="px-2">
              <Slider
                type="single"
                bind:value={data.stk_delay}
                max={30}
                min={1}
                step={1}
                class="w-full"
              />
            </div>
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>1s</span>
              <span class="font-medium">{data.stk_delay}s delay</span>
              <span>30s</span>
            </div>
            <p class="text-xs text-muted-foreground">
              Simulate real-world STK push response time
            </p>
          </div>

          <!-- Custom Prefix -->
          <div class="space-y-2">
            <Label
              for="custom-prefix"
              class="text-sm font-medium flex items-center gap-1"
            >
              <Tag class="h-4 w-4" />
              Custom Prefix
            </Label>
            <Input
              id="custom-prefix"
              bind:value={data.prefix}
              placeholder="test_"
              class="w-full"
            />
            <p class="text-xs text-muted-foreground">
              Prefix for generated transaction IDs
            </p>
          </div>
        </CardContent>
      </Card>

      <!-- Action Buttons -->
      <div class="flex gap-3 justify-end">
        <Button variant="outline" onclick={back}>Cancel</Button>
        {#if !saving}
          <Button
            onclick={handleSave}
            disabled={!isValid || !hasChanges}
            class="min-w-32"
          >
            Save Project
          </Button>
        {:else}
          <Button disabled={true} class="min-w-32">
            <LoaderCircle class="animate-spin h-4 w-4 mr-2" />
            Saving...
          </Button>
        {/if}
      </div>
    {/if}
  </div>
</div>
