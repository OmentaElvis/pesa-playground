<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { deleteProject, getProject, type ProjectSummary } from '$lib/api';
	import { Settings, Trash2, KeyRound, FileText } from 'lucide-svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { getSimulationModeColor, formatRelativeTime } from '$lib/utils';
	import CreateProjectCard from './CreateProjectCard.svelte';
	import { toast } from 'svelte-sonner';

	export let projects: ProjectSummary[] = [];
	export let businessId: number;

	async function removeProject(id: number) {
		await deleteProject(id);
	}

	async function copyPasskey(projectId: number) {
		try {
			const projectDetails = await getProject(projectId);
			await navigator.clipboard.writeText(projectDetails.passkey);
			toast.success('Passkey copied to clipboard!');
		} catch (error) {
			toast.error('Failed to copy passkey.');
		}
	}
</script>

<div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
	<CreateProjectCard {businessId} />
	{#each projects as project}
		<Card class="flex flex-col transition-shadow duration-200 hover:shadow-lg">
			<CardHeader class="pb-3">
				<div class="flex items-start justify-between">
					<div class="space-y-1">
						<CardTitle class="text-lg font-semibold">{project.name}</CardTitle>
						<CardDescription>
							for {project.business_name}
						</CardDescription>
					</div>
					<div class="flex gap-1">
						<Button size="sm" variant="ghost" href={`/projects/${project.id}/settings`}>
							<Settings class="h-4 w-4" />
						</Button>
						<AlertDialog.Root>
							<AlertDialog.Trigger>
								<Button variant="ghost" size="sm">
									<Trash2 class="h-4 w-4 text-destructive" />
								</Button>
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
									<AlertDialog.Action onclick={() => removeProject(project.id)}>
										Continue
									</AlertDialog.Action>
								</AlertDialog.Footer>
							</AlertDialog.Content>
						</AlertDialog.Root>
					</div>
				</div>
			</CardHeader>

			<CardContent class="flex-grow space-y-4">
				<div class="flex items-center gap-2">
					<Badge class={getSimulationModeColor(project.simulation_mode)} variant="outline">
						{project.simulation_mode.replace('-', ' ')}
					</Badge>
					<span class="text-xs text-muted-foreground">
						{formatRelativeTime(project.created_at)}
					</span>
				</div>
				<div class="flex gap-2 pt-2">
					<Button size="sm" variant="outline" onclick={() => copyPasskey(project.id)}>
						<KeyRound class="mr-2 h-4 w-4" />
						Copy Passkey
					</Button>
					<Button size="sm" variant="outline" href={`/projects/${project.id}?tab=logs`}>
						<FileText class="mr-2 h-4 w-4" />
						API Logs
					</Button>
				</div>
			</CardContent>
			<CardFooter class="pt-4">
				<Button size="sm" href={`/projects/${project.id}`} class="flex-1">View Project</Button>
			</CardFooter>
		</Card>
	{/each}
</div>
