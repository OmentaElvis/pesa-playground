<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import {
		Settings,
		Building,
		FileDigit,
		SearchIcon,
		PlusCircle,
		ArrowRight,
		HashIcon,
		LandmarkIcon
	} from 'lucide-svelte';
	import {
		getProjects,
		type ProjectSummary,
		getBusinesses,
		type BusinessSummary,
		SimulationMode
	} from '$lib/api';
	import { onMount } from 'svelte';
	import { getSimulationModeColor } from '$lib/utils';
	import * as Select from '$lib/components/ui/select';
	import * as InputGroup from '$lib/components/ui/input-group/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import PesaPlaygroundLogo from '$lib/components/logo/PesaPlaygroundLogo.svelte';
	import * as Kbd from '$lib/components/ui/kbd/index.js';
	import { globalKeymapActions } from '$lib/actions/keymapActions';

	const displayedKeymapActions = globalKeymapActions.filter((action) =>
		['project.create', 'users.manage', 'businesses.manage', 'settings.open'].includes(action.id)
	);

	let projects: ProjectSummary[] = $state([]);
	let businesses: BusinessSummary[] = $state([]);
	let searchText = $state('');
	let selectedBusiness: string = $state('');
	let selectedSimMode: string = $state('');
	let activeTab = $state('all-projects');
	let loading = $state(true);
	let isHoveringCreateProjectButton = $state(false);

	let projectsByBusiness: Map<number, ProjectSummary[]> = $derived(
		projects.reduce((acc, project) => {
			if (!acc.has(project.business_id)) {
				acc.set(project.business_id, []);
			}
			acc.get(project.business_id)!.push(project);
			return acc;
		}, new Map<number, ProjectSummary[]>())
	);

	let filteredProjects: ProjectSummary[] = $derived.by(() => {
		let filtered = projects;

		if (searchText) {
			filtered = filtered.filter(
				(p) =>
					p.name.toLowerCase().includes(searchText.toLowerCase()) ||
					p.business_name.toLowerCase().includes(searchText.toLowerCase()) ||
					p.short_code.toLowerCase().includes(searchText.toLowerCase())
			);
		}

		if (selectedBusiness) {
			filtered = filtered.filter((p) => p.business_id === parseInt(selectedBusiness!));
		}

		if (selectedSimMode) {
			filtered = filtered.filter((p) => p.simulation_mode === selectedSimMode);
		}

		return filtered;
	});

	onMount(() => {
		const loadData = async () => {
			try {
				projects = (await getProjects()) as ProjectSummary[];
				businesses = await getBusinesses();
			} finally {
				loading = false;
			}
		};

		loadData();
	});
</script>

{#snippet keymapRow(name: string, shortcut: string)}
	{@const keys = shortcut.split('+')}
	<p class="text-right text-sm">{name}</p>
	<div class="text-left">
		<Kbd.Root>
			{#each keys as key, i}
				<span class="uppercase">{key}</span>
				{#if i < keys.length - 1}
					+
				{/if}
			{/each}
		</Kbd.Root>
	</div>
{/snippet}

<main class="container mx-auto space-y-6 p-6">
	{#if loading}
		<p>Loading projects...</p>
	{:else if projects.length === 0}
		<div class="flex h-[calc(100vh-200px)] flex-col items-center justify-center gap-8 text-center">
			<PesaPlaygroundLogo variant={isHoveringCreateProjectButton ? 'color' : 'mono'} width="400" />
			<div class="flex flex-col items-center gap-4">
				<h2 class="text-xl font-semibold">No Projects Yet</h2>
				<p class="max-w-md text-muted-foreground">
					Get started by creating your first project. A project is a workspace for your integration,
					containing API credentials and settings.
				</p>
				<Button
					href="/projects/new"
					size="lg"
					class="mt-4"
					onmouseenter={() => (isHoveringCreateProjectButton = true)}
					onmouseleave={() => (isHoveringCreateProjectButton = false)}
				>
					<PlusCircle class="mr-2 h-4 w-4" />
					Create New Project
				</Button>
			</div>

			<div class="mt-8 grid w-full max-w-md grid-cols-2 gap-x-4 gap-y-2 text-muted-foreground">
				{#each displayedKeymapActions as action}
					{@render keymapRow(action.name, action.defaultShortcut)}
				{/each}
			</div>
		</div>
	{:else}
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground">Projects</h1>
				<p class="mt-1 text-muted-foreground">
					A project is a workspace for your integration. It contains API credentials and settings.
				</p>
			</div>
			<Button href="/projects/new">
				<PlusCircle class="mr-2 h-4 w-4" />
				New Project
			</Button>
		</div>

		<Tabs.Root bind:value={activeTab}>
			<Tabs.List>
				<Tabs.Trigger value="all-projects">All Projects</Tabs.Trigger>
				<Tabs.Trigger value="by-business"><LandmarkIcon /> By Business</Tabs.Trigger>
			</Tabs.List>

			<Tabs.Content value="all-projects">
				<!-- Search and Filters -->
				<div class="mt-4 flex gap-4 max-md:flex-col">
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
					<Select.Root bind:value={selectedBusiness} type="single">
						<Select.Trigger class="w-[180px]">Filter by Business</Select.Trigger>
						<Select.Content>
							<Select.Item value="">All Businesses</Select.Item>
							{#each businesses as business}
								<Select.Item value={business.id.toString()}>{business.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					<Select.Root bind:value={selectedSimMode} type="single">
						<Select.Trigger class="w-[180px]">Filter by Sim Mode</Select.Trigger>
						<Select.Content>
							<Select.Item value="">All Modes</Select.Item>
							{#each Object.values(SimulationMode) as mode}
								<Select.Item value={mode}>{mode}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="mt-6 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
					{#each filteredProjects as project (project.id)}
						<Card class="transition-shadow duration-200 hover:shadow-lg">
							<CardHeader class="pb-3">
								<div class="flex items-start justify-between">
									<div class="space-y-1">
										<CardTitle class="text-lg font-semibold">{project.name}</CardTitle>
									</div>
									<div class="flex gap-1">
										<Button size="sm" variant="ghost" href={`/projects/${project.id}/settings`}>
											<Settings class="h-4 w-4" />
										</Button>
									</div>
								</div>
							</CardHeader>
							<CardContent class="space-y-4">
								<div class="flex items-center gap-2">
									<Badge class={getSimulationModeColor(project.simulation_mode)} variant="outline">
										{project.simulation_mode.replace('-', ' ')}
									</Badge>
								</div>
								<a class="flex items-center gap-2" href="/businesses/{project.business_id}">
									<Building size={20} />
									{project.business_name}
								</a>
								<spawn class="flex items-center gap-2 text-muted-foreground">
									<FileDigit size={20} />
									{project.short_code}
								</spawn>
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
			</Tabs.Content>

			<Tabs.Content value="by-business">
				<div class="mt-6 space-y-8">
					{#each businesses as business (business.id)}
						{@const businessProjects = projectsByBusiness.get(business.id) || []}
						<section>
							<div class="flex items-center">
								<h2 class="flex-1 text-2xl font-semibold tracking-tight text-foreground">
									{business.name}
									<Badge variant="outline" class="ml-4"><HashIcon /> {business.short_code}</Badge>
								</h2>
								<Button href="/businesses/{business.id}" variant="outline">
									View {business.name}<ArrowRight />
								</Button>
							</div>
							<Separator class="my-4" />
							{#if businessProjects.length > 0}
								<div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
									{#each businessProjects as project (project.id)}
										<Card class="transition-shadow duration-200 hover:shadow-lg">
											<CardHeader class="pb-3">
												<div class="flex items-start justify-between">
													<div class="space-y-1">
														<CardTitle class="text-lg font-semibold">
															{project.name}
														</CardTitle>
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
														{project.simulation_mode.replace('-', ' ')}
													</Badge>
												</div>
												<spawn class="flex items-center gap-2 text-muted-foreground">
													<FileDigit size={20} />
													{project.short_code}
												</spawn>
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
							{:else}
								<div
									class="flex flex-col items-center justify-center rounded-lg border-2 border-dashed border-muted bg-muted/50 p-8 text-center"
								>
									<p class="text-muted-foreground">No projects for this business yet.</p>
									<Button
										size="sm"
										variant="outline"
										class="mt-4"
										href="/projects/new?business_id={business.id}"
									>
										Create a project
									</Button>
								</div>
							{/if}
						</section>
					{/each}
				</div>
			</Tabs.Content>
		</Tabs.Root>
	{/if}
</main>
