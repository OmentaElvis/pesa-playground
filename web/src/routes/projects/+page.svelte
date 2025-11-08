<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
	} from "$lib/components/ui/card";
	import { Badge } from "$lib/components/ui/badge";
	import { Settings, Building, FileDigit, SearchIcon } from "lucide-svelte";
	import {
		getProjects,
		type ProjectSummary,
		getBusinesses,
		type BusinessSummary,
		SimulationMode,
	} from "$lib/api";
	import { onMount } from "svelte";
	import { getSimulationModeColor } from "$lib/utils";
	import * as Select from "$lib/components/ui/select";
	import * as InputGroup from "$lib/components/ui/input-group/index.js";

	let projects: ProjectSummary[] = $state([]);
	let businesses: BusinessSummary[] = $state([]);
	let searchText = $state("");
	let selectedBusiness: string = $state("");
	let selectedSimMode: string = $state("");

	let filteredProjects: ProjectSummary[] = $derived.by(() => {
		let filtered = projects;

		if (searchText) {
			filtered = filtered.filter(
				(p) =>
					p.name.toLowerCase().includes(searchText.toLowerCase()) ||
					p.business_name.toLowerCase().includes(searchText.toLowerCase()) ||
					p.short_code.toLowerCase().includes(searchText.toLowerCase()),
			);
		}

		if (selectedBusiness) {
			filtered = filtered.filter(
				(p) => p.business_id === parseInt(selectedBusiness!),
			);
		}

		if (selectedSimMode) {
			filtered = filtered.filter((p) => p.simulation_mode === selectedSimMode);
		}

		return filtered;
	});

	onMount(async () => {
		projects = (await getProjects()) as ProjectSummary[];
		businesses = await getBusinesses();
	});
</script>

<main class="container mx-auto p-6 space-y-6">
	<!-- Header -->
	<div class="flex justify-between items-center">
		<div>
			<h1 class="text-3xl font-bold tracking-tight text-foreground">
				All Projects
			</h1>
			<p class="text-muted-foreground mt-1">
				Manage all your M-Pesa testing environments
			</p>
		</div>
	</div>

	<!-- Search and Filters -->
	<div class="flex gap-4 max-md:flex-col">
		<div class="relative w-full max-w-sm">
			<InputGroup.Root>
				<InputGroup.Input
					type="search"
					bind:value={searchText}
					class="pl-10"
					placeholder="Search by name, business, or shortcode..."
				/>
				<InputGroup.Addon>
					<SearchIcon />
				</InputGroup.Addon>
			</InputGroup.Root>
		</div>
		<Select.Root type="single" bind:value={selectedBusiness}>
			<Select.Trigger class="w-[180px]">Filter by Business</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All Businesses</Select.Item>
				{#each businesses as business}
					<Select.Item value={business.id.toString()}
						>{business.name}</Select.Item
					>
				{/each}
			</Select.Content>
		</Select.Root>
		<Select.Root type="single" bind:value={selectedSimMode}>
			<Select.Trigger class="w-[180px]">Filter by Sim Mode</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All Modes</Select.Item>
				{#each Object.values(SimulationMode) as mode}
					<Select.Item value={mode}>{mode}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<div
		class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
	>
		{#each filteredProjects as project (project.id)}
			<Card class="hover:shadow-lg transition-shadow duration-200">
				<CardHeader class="pb-3">
					<div class="flex justify-between items-start">
						<div class="space-y-1">
							<CardTitle class="text-lg font-semibold">{project.name}</CardTitle
							>
						</div>
						<div class="flex gap-1">
							<Button
								size="sm"
								variant="ghost"
								href={`/projects/${project.id}/settings`}
							>
								<Settings class="h-4 w-4" />
							</Button>
						</div>
					</div>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="flex items-center gap-2">
						<Badge
							class={getSimulationModeColor(project.simulation_mode)}
							variant="outline"
						>
							{project.simulation_mode.replace("-", " ")}
						</Badge>
					</div>
					<a
						class="flex gap-2 items-center"
						href="/businesses/{project.business_id}"
					>
						<Building size={20} />
						{project.business_name}
					</a>
					<spawn class="text-muted-foreground flex items-center gap-2"
						><FileDigit size={20} /> {project.short_code}</spawn
					>
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
</main>
