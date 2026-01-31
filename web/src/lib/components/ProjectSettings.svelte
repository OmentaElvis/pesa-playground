<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select/index';
	import * as Item from "$lib/components/ui/item/index.js";
	import * as InputGroup from "$lib/components/ui/input-group/index.js";

	import {
		Globe,
		Code,
		Timer,
		Tag,
		LoaderCircle,
		SaveIcon,
		MonitorCog,
		AlarmClock,

		DollarSign

	} from 'lucide-svelte';
	import { getProject, updateProject, deleteProject } from '$lib/api';
	import type { ProjectDetails, UpdateProjectData } from '$lib/api';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import DangerAction from './shared/DangerAction.svelte';
	import CardFooter from './ui/card/card-footer.svelte';

	let { project = $bindable() }: {project: ProjectDetails} = $props();
	let id = $derived(Number(page.params.id));

	// svelte-ignore state_referenced_locally
	let originalData: ProjectDetails = project;
	let saving = $state(false);
	let error = $state('');

	async function handleSave() {
		if (!project.name?.trim()) {
			error = 'Project name and shortcode are required';
			return;
		}

		try {
			saving = true;
			error = '';

			// Create update payload with only changed fields
			const updatePayload: UpdateProjectData = {};

			if (project.name !== originalData.name) {
				updatePayload.name = project.name;
			}
			if (project.callback_url !== originalData.callback_url) {
				updatePayload.callback_url = project.callback_url;
			}
			if (project.simulation_mode !== originalData.simulation_mode) {
				updatePayload.simulation_mode = project.simulation_mode;
			}
			if (project.stk_delay !== originalData.stk_delay) {
				updatePayload.stk_delay = project.stk_delay;
			}
			if (project.prefix !== originalData.prefix) {
				updatePayload.prefix = project.prefix;
			}

			if (Object.keys(updatePayload).length > 0) {
				await updateProject(id, updatePayload);
				originalData = { ...project };
			}

			toast.success('Project updated.');
			goto(`/projects/${id}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to update project';
		} finally {
			saving = false;
		}
	}

	async function handleDelete() {
		if (confirm('Are you sure you want to delete this project? This action cannot be undone.')) {
			try {
				await deleteProject(id);
				toast.success('Project deleted successfully.');
				goto('/projects');
			} catch (err) {
				toast.error(err instanceof Error ? err.message : 'Failed to delete project');
			}
		}
	}

	// Check if form has unsaved changes
	let hasChanges = $derived(
		project.name !== originalData?.name ||
			project.callback_url !== originalData?.callback_url ||
			project.simulation_mode !== originalData?.simulation_mode ||
			project.stk_delay !== originalData?.stk_delay ||
			project.txn_delay !== originalData?.txn_delay ||
			project.prefix !== originalData?.prefix
	);

	// Form validation
	let isValid = $derived.by(() => {
	  return (project.name?.trim() && project.stk_delay >= 0 && project.txn_delay >= 0)
	});
</script>

<div class="min-h-screen bg-background">
	<div class="mx-auto space-y-8">
			<!-- Main Form Card -->
			<Card class="shadow-lg">
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<Code class="h-5 w-5" />
						"{project.name}" Configuration
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-6">
					<!-- Error Message -->
					{#if error}
						<div class="rounded-md border border-destructive/20 bg-destructive/10 p-3">
							<p class="text-sm text-destructive">{error}</p>
						</div>
					{/if}

					<!-- Project Name -->
					<div class="space-y-2">
						<Label for="project-name" class="text-sm font-medium">Project Name</Label>
						<Input
							id="project-name"
							bind:value={project.name}
							placeholder="My Test App"
							class="w-full"
						/>
						<p class="text-xs text-muted-foreground">A friendly name for your project</p>
					</div>

					<!-- Callback URL -->
					<Item.Root variant="outline">
						<Item.Media>
							<Globe />
						</Item.Media>
						<Item.Content>
							<Item.Title>Callback URL (deprecated)</Item.Title>
							<Item.Description>Where we'll send payment notifications for stkpush.</Item.Description>
						</Item.Content>
						<Item.Footer>
							<Input
								id="callback-url"
								bind:value={project.callback_url}
								placeholder="http://localhost:5001/callback"
								class="w-full"
							/>
						</Item.Footer>
					</Item.Root>

					<!-- Custom Prefix -->
					<Item.Root variant="outline">
						<Item.Media>
							<Tag />
						</Item.Media>
						<Item.Content>
							<Item.Title>Custom Prefix</Item.Title>
							<Item.Description>Prefix for generated transaction IDs</Item.Description>
						</Item.Content>
						<Item.Footer>
							<Input id="custom-prefix" bind:value={project.prefix} placeholder="test_" class="w-full" />
						</Item.Footer>
					</Item.Root>

					<h2>Core </h2>

					<!-- Simulation Mode -->
					<Item.Root variant="outline">
						<Item.Media>
							<MonitorCog />
						</Item.Media>
						<Item.Content>
							<Item.Title>Simulation Mode</Item.Title>
							<Item.Description>How payment simulations should behave</Item.Description>
						</Item.Content>
						<Item.Actions>
							<Select.Root type="single" bind:value={project.simulation_mode} name="simulationMode">
								<Select.SelectTrigger>
									{project.simulation_mode || 'Select simulation mode'}
								</Select.SelectTrigger>
								<Select.Content>
									<Select.SelectItem value="always-success">Always Success</Select.SelectItem>
									<Select.SelectItem value="always-fail">Always Fail</Select.SelectItem>
									<Select.SelectItem value="random">Random Success/Fail</Select.SelectItem>
									<Select.SelectItem value="realistic">Realistic Simulation</Select.SelectItem>
								</Select.Content>
							</Select.Root>
						</Item.Actions>
					</Item.Root>

					<!-- STK Delay -->
					<Item.Root variant="outline">
						<Item.Media>
							<Timer />
						</Item.Media>
						<Item.Content>
							<Item.Title>STK Push Delay</Item.Title>
							<Item.Description>STK push delay time. This is the number of seconds the core will wait before pushing an STK prompt.</Item.Description>
						</Item.Content>
						<Item.Actions>
							<InputGroup.Root>
						    <InputGroup.Input placeholder="" type="number" min="0" max="30" bind:value={project.stk_delay} />
						    <InputGroup.Addon align="inline-end">seconds</InputGroup.Addon>
						  </InputGroup.Root>
						</Item.Actions>
					</Item.Root>

					<Item.Root variant="outline">
						<Item.Media>
							<DollarSign />
						</Item.Media>
						<Item.Content>
							<Item.Title>Transaction schedule delay</Item.Title>
							<Item.Description >
								Introduces a delay in transaction processing, allowing you to test the Transaction Status API in scenarios where transactions are not instantaneous.
								A non-zero value simulates a slower transaction.
							</Item.Description>
						</Item.Content>
						<Item.Actions>
							<InputGroup.Root>
						    <InputGroup.Input placeholder="" type="number" min="0" max="30" bind:value={project.txn_delay} />
						    <InputGroup.Addon align="inline-end">seconds</InputGroup.Addon>
						  </InputGroup.Root>
						</Item.Actions>
					</Item.Root>

				</CardContent>
				<CardFooter>
					<!-- Action Buttons -->
					<div class="flex justify-end gap-3">
						{#if !saving}
							<Button onclick={handleSave} disabled={!isValid || !hasChanges} class="min-w-32">
							 <SaveIcon />	Update Project
							</Button>
						{:else}
							<Button disabled={true} class="min-w-32">
								<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
								Saving...
							</Button>
						{/if}
					</div>
				</CardFooter>
			</Card>


			<div>
				<h3 class="mb-2 text-lg font-semibold">Danger Zone</h3>
				<DangerAction
					title="Delete this project"
					description="This will delete the project, api keys, project settings and api logs. Business, users and transaction will be unaffected."
					buttonLabel="Delete this project"
					dialogTitle="Are you absolutely sure?"
					dialogDescription="This action is irreversible. This project will be permanently deleted."
					onConfirm={handleDelete}
				/>
			</div>
	</div>
</div>
