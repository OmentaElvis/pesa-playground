<script>
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
    Shuffle,
    Globe,
    Code,
    Timer,
    Users,
    Tag,
    CheckCircle,
    LoaderCircle,
  } from "lucide-svelte";
  import { goto } from "$app/navigation";
  import { createProject } from "$lib/api";

  let projectName = "";
  let shortcode = "";
  let callbackUrl = "http://localhost:5001/callback";
  let simulationMode = "always-success";
  let stkDelay = 3;
  let initialUsers = 5;
  let customPrefix = "test_";
  let creating = false;

  function generateShortcode() {
    const code = Math.floor(100000 + Math.random() * 900000);
    shortcode = code.toString();
  }

  async function handleCreate() {
    try {
      creating = true;
      let res = await createProject({
        callback_url: callbackUrl,
        name: projectName,
        shortcode: shortcode,
        simulation_mode: simulationMode,
        stk_delay: stkDelay,
        initial_users: initialUsers,
        prefix: customPrefix,
      });
      await goto(`/projects/${res.project_id}`, {replaceState: true});
    } catch (err) {
      console.log("Creating project:", err);
    } finally {
      creating = false;
    }
  }

  async function handleCancel() {
    // Reset form or navigate away
    await goto("/", { replaceState: true });
  }
</script>

<div class="min-h-screen bg-background p-6">
  <div class="max-w-2xl mx-auto space-y-8">
    <!-- Header -->
    <div class="text-center space-y-2">
      <h1 class="text-3xl font-bold tracking-tight text-foreground">
        Create New Project
      </h1>
      <p class="text-muted-foreground">
        Set up your M-Pesa testing environment
      </p>
    </div>

    <!-- Main Form Card -->
    <Card class="shadow-lg">
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <Code class="h-5 w-5" />
          Project Configuration
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-6">
        <!-- Project Name -->
        <div class="space-y-2">
          <Label for="project-name" class="text-sm font-medium"
            >Project Name</Label
          >
          <Input
            id="project-name"
            bind:value={projectName}
            placeholder="My Test App"
            class="w-full"
          />
          <p class="text-xs text-muted-foreground">
            A friendly name for your project
          </p>
        </div>

        <!-- Shortcode -->
        <div class="space-y-2">
          <Label for="shortcode" class="text-sm font-medium">Shortcode</Label>
          <div class="flex gap-2">
            <Input
              id="shortcode"
              bind:value={shortcode}
              placeholder="174379"
              class="flex-1"
            />
            <Button
              variant="outline"
              size="icon"
              onclick={generateShortcode}
              title="Generate random shortcode"
            >
              <Shuffle class="h-4 w-4" />
            </Button>
          </div>
          <p class="text-xs text-muted-foreground">
            6-digit business shortcode for payments
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
            bind:value={callbackUrl}
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
            bind:value={simulationMode}
            name="simulationMode"
          >
            <Select.SelectTrigger>
              {simulationMode}
            </Select.SelectTrigger>
            <Select.Content>
              <Select.SelectItem value="always-success"
                >Always Success</Select.SelectItem
              >
              <Select.SelectItem value="always-fail"
                >Always Fail</Select.SelectItem
              >
              <Select.SelectItem value="random"
                >Random Success/Fail</Select.SelectItem
              >
              <Select.SelectItem value="realistic"
                >Realistic Simulation</Select.SelectItem
              >
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
              bind:value={stkDelay}
              max={30}
              min={1}
              step={1}
              class="w-full"
            />
          </div>
          <div class="flex justify-between text-xs text-muted-foreground">
            <span>1s</span>
            <span class="font-medium">{stkDelay}s delay</span>
            <span>30s</span>
          </div>
          <p class="text-xs text-muted-foreground">
            Simulate real-world STK push response time
          </p>
        </div>

        <!-- Initial Users -->
        <div class="space-y-2">
          <Label
            for="initial-users"
            class="text-sm font-medium flex items-center gap-1"
          >
            <Users class="h-4 w-4" />
            Initial Test Users
          </Label>
          <Input
            id="initial-users"
            type="number"
            bind:value={initialUsers}
            min="1"
            max="50"
            class="w-full"
          />
          <p class="text-xs text-muted-foreground">
            Number of fake users to generate for testing
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
            bind:value={customPrefix}
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
      <Button variant="outline" onclick={handleCancel}>Cancel</Button>
      {#if !creating}
        <Button
          onclick={handleCreate}
          disabled={!projectName.trim() || !shortcode.trim()}
          class="min-w-32"
        >
          Create Project
        </Button>
      {:else}
        <Button
          onclick={handleCreate}
          disabled={true}
          class="min-w-32"
        >
          <LoaderCircle class="animate-spin" /> Create Project
        </Button>
      {/if}
    </div>
  </div>
</div>
