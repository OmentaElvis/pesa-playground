<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select/index';
	import { Slider } from '$lib/components/ui/slider';
	import { Globe, Code, Timer, Tag, CheckCircle, LoaderCircle } from 'lucide-svelte';
	import { goto } from '$app/navigation';
	import { createProject, SimulationMode } from '$lib/api';
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';

	let projectName = $state('');
	let callbackUrl = $state('http://localhost:5001/callback');
	let simulationMode: SimulationMode = $state(SimulationMode.Realistic);
	let stkDelay = $state(3);
	let customPrefix = $state('test_');
	let creating = $state(false);
	let businessId = $derived(page.params.business);

	async function handleCreate() {
		try {
			creating = true;
			let res = await createProject({
				callback_url: callbackUrl,
				name: projectName,
				simulation_mode: simulationMode,
				stk_delay: stkDelay,
				prefix: customPrefix,
				business_id: Number(businessId)
			});
			await goto(`/projects/${res.id}`, { replaceState: true });
		} catch (err) {
			toast(`Creating project: ${err}`);
		} finally {
			creating = false;
		}
	}

	async function handleCancel() {
		// Reset form or navigate away
		await goto('/', { replaceState: true });
	}
</script>

<div class="min-h-screen bg-background p-6">
	<div class="mx-auto max-w-2xl space-y-8">
		<!-- Header -->
		<div class="space-y-2 text-center">
			<h1 class="text-3xl font-bold tracking-tight text-foreground">Create New Project</h1>
			<p class="text-muted-foreground">Set up your M-Pesa testing environment</p>
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
					<Label for="project-name" class="text-sm font-medium">Project Name</Label>
					<Input
						id="project-name"
						bind:value={projectName}
						placeholder="My Test App"
						class="w-full"
					/>
					<p class="text-xs text-muted-foreground">A friendly name for your project</p>
				</div>

				<!-- Callback URL -->
				<div class="space-y-2">
					<Label for="callback-url" class="flex items-center gap-1 text-sm font-medium">
						<Globe class="h-4 w-4" />
						Callback URL
					</Label>
					<Input
						id="callback-url"
						bind:value={callbackUrl}
						placeholder="http://localhost:5001/callback"
						class="w-full"
					/>
					<p class="text-xs text-muted-foreground">Where we'll send payment notifications</p>
				</div>

				<!-- Simulation Mode -->
				<div class="space-y-2">
					<Label class="flex items-center gap-1 text-sm font-medium">
						<CheckCircle class="h-4 w-4" />
						Simulation Mode
					</Label>
					<Select.Root type="single" bind:value={simulationMode} name="simulationMode">
						<Select.SelectTrigger>
							{simulationMode}
						</Select.SelectTrigger>
						<Select.Content>
							<Select.SelectItem value="always_success">Always Success</Select.SelectItem>
							<Select.SelectItem value="always_fail">Always Fail</Select.SelectItem>
							<Select.SelectItem value="random">Random Success/Fail</Select.SelectItem>
							<Select.SelectItem value="realistic">Realistic Simulation</Select.SelectItem>
						</Select.Content>
					</Select.Root>
					<p class="text-xs text-muted-foreground">How payment simulations should behave</p>
				</div>

				<!-- STK Delay -->
				<div class="space-y-4">
					<Label class="flex items-center gap-1 text-sm font-medium">
						<Timer class="h-4 w-4" />
						STK Push Delay
					</Label>
					<div class="px-2">
						<Slider type="single" bind:value={stkDelay} max={30} min={1} step={1} class="w-full" />
					</div>
					<div class="flex justify-between text-xs text-muted-foreground">
						<span>1s</span>
						<span class="font-medium">{stkDelay}s delay</span>
						<span>30s</span>
					</div>
					<p class="text-xs text-muted-foreground">Simulate real-world STK push response time</p>
				</div>

				<!-- Custom Prefix -->
				<div class="space-y-2">
					<Label for="custom-prefix" class="flex items-center gap-1 text-sm font-medium">
						<Tag class="h-4 w-4" />
						Custom Prefix
					</Label>
					<Input id="custom-prefix" bind:value={customPrefix} placeholder="test_" class="w-full" />
					<p class="text-xs text-muted-foreground">Prefix for generated transaction IDs</p>
				</div>
			</CardContent>
		</Card>

		<!-- Action Buttons -->
		<div class="flex justify-end gap-3">
			<Button variant="outline" onclick={handleCancel}>Cancel</Button>
			{#if !creating}
				<Button onclick={handleCreate} disabled={!projectName.trim()} class="min-w-32">
					Create Project
				</Button>
			{:else}
				<Button onclick={handleCreate} disabled={true} class="min-w-32">
					<LoaderCircle class="animate-spin" /> Create Project
				</Button>
			{/if}
		</div>
	</div>
</div>
