<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Settings, Building, FileDigit } from 'lucide-svelte';
	import { getProjects, type ProjectSummary } from '$lib/api';
	import { onMount } from 'svelte';
	import { getSimulationModeColor } from '$lib/utils';

	// Mock data - replace with your actual data source
	let projects: ProjectSummary[] = $state([]);

	onMount(async () => {
		projects = (await getProjects()) as any;
	});
</script>

<main class="container mx-auto space-y-6 p-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight text-foreground">Projects</h1>
			<p class="mt-1 text-muted-foreground">Manage your M-Pesa testing environments</p>
		</div>
	</div>
	<div class="grid grid-cols-3 gap-4">
		{#each projects as project (project.id)}
			<Card class="transition-shadow duration-200 hover:shadow-lg">
				<CardHeader class="pb-3">
					<div class="flex items-start justify-between">
						<div class="space-y-1">
							<CardTitle class="text-lg font-semibold">{project.name}</CardTitle>
							<div class="flex items-center gap-2">
								<Badge class={getSimulationModeColor(project.simulation_mode)} variant="outline">
									{project.simulation_mode.replace('-', ' ')}
								</Badge>
							</div>
						</div>
						<div class="flex gap-1">
							<Button size="sm" variant="ghost" href={`/projects/${project.id}/settings`}>
								<Settings class="h-4 w-4" />
							</Button>
						</div>
					</div>
				</CardHeader>
				<CardContent class="space-y-4">
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
						<Button size="sm" href={`/projects/${project.id}`} class="flex-1">View project</Button>
					</div>
				</CardContent>
			</Card>
		{/each}
	</div>
</main>
