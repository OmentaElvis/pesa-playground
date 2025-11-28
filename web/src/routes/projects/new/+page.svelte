<script lang="ts">
	import {
		createBusiness,
		createProject,
		getBusinesses,
		SimulationMode,
		type BusinessSummary
	} from '$lib/api';
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as RadioGroup from '$lib/components/ui/radio-group';
	import * as Select from '$lib/components/ui/select';
	import { ArrowRight, Loader2 } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { writable, get } from 'svelte/store';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	// Zod Schema for client-side validation
	const newProjectSchema = z
		.object({
			projectName: z.string().min(3, 'Project name is required.'),
			businessChoice: z.enum(['existing', 'new']),
			existingBusinessId: z.string().optional(),
			newBusinessName: z.string().optional(),
			newBusinessShortCode: z.string().optional(),
			simulationMode: z.enum(SimulationMode),
			stkDelay: z.number().min(0)
		})
		.refine(
			(data) => {
				if (data.businessChoice === 'new') {
					return data.newBusinessName && data.newBusinessName.trim().length > 0;
				}
				return true;
			},
			{ message: 'New business name is required.', path: ['newBusinessName'] }
		)
		.refine(
			(data) => {
				if (data.businessChoice === 'new') {
					return data.newBusinessShortCode && data.newBusinessShortCode.trim().length > 0;
				}
				return true;
			},
			{ message: 'Shortcode is required.', path: ['newBusinessShortCode'] }
		)
		.refine(
			(data) => {
				if (data.businessChoice === 'existing') {
					return !!data.existingBusinessId;
				}
				return true;
			},
			{ message: 'Please select a business.', path: ['existingBusinessId'] }
		);

	type NewProjectSchema = z.infer<typeof newProjectSchema>;

	let isLoading = $state(false);
	const formData = writable<NewProjectSchema>({
		projectName: '',
		businessChoice: 'existing',
		existingBusinessId: undefined,
		newBusinessName: undefined,
		newBusinessShortCode: undefined,
		simulationMode: SimulationMode.Realistic,
		stkDelay: 0
	});
	const { update } = formData;
	const errors = writable<z.ZodFormattedError<NewProjectSchema> | null>(null);

	let businesses: BusinessSummary[] = $state([]);

	onMount(async () => {
		businesses = await getBusinesses();
		const urlBusinessId = page.url.searchParams.get('business_id');

		if (businesses.length === 0) {
			update((data) => ({ ...data, businessChoice: 'new' }));
		} else if (urlBusinessId && businesses.some((b) => b.id.toString() === urlBusinessId)) {
			update((data) => ({ ...data, existingBusinessId: urlBusinessId }));
		} else {
			update((data) => ({ ...data, existingBusinessId: businesses[0]?.id.toString() }));
		}
	});

	async function handleSubmit() {
		isLoading = true;
		errors.set(null);
		const currentData = get(formData);
		const validationResult = newProjectSchema.safeParse(currentData);

		if (!validationResult.success) {
			errors.set(validationResult.error.format());
			isLoading = false;
			return;
		}

		try {
			let businessId: number;
			const { data } = validationResult;

			if (data.businessChoice === 'new') {
				// These checks are for type safety, though Zod's refine should prevent this state.
				if (!data.newBusinessName || !data.newBusinessShortCode) {
					toast.error('Business name and shortcode are required.');
					isLoading = false;
					return;
				}
				const newBusiness = await createBusiness({
					name: data.newBusinessName,
					short_code: data.newBusinessShortCode
				});
				businessId = newBusiness.id;
			} else {
				if (!data.existingBusinessId) {
					toast.error('Please select an existing business.');
					isLoading = false;
					return;
				}
				businessId = parseInt(data.existingBusinessId, 10);
			}

			const newProject = await createProject({
				name: data.projectName,
				business_id: businessId,
				simulation_mode: data.simulationMode,
				stk_delay: Number(data.stkDelay) || 0
			});

			toast.success('Project created successfully!');
			goto(`/projects/${newProject.id}`);
		} catch (error: any) {
			console.error('Failed to create project:', error);
			toast.error(`Failed to create project: ${error.message}`);
		} finally {
			isLoading = false;
		}
	}
</script>

<main class="container mx-auto flex max-w-2xl flex-col items-center p-6">
	<Card class="w-full">
		<CardHeader>
			<CardTitle>Create a New Project</CardTitle>
			<CardDescription>
				A project is a workspace for your integration. Fill in the details below to get started.
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-6">
			<div class="space-y-2">
				<Label for="projectName">Project Name</Label>
				<Input
					id="projectName"
					placeholder="e.g., 'Website Checkout Test'"
					bind:value={$formData.projectName}
					class="text-lg"
				/>
				{#if $errors?.projectName?._errors}
					<p class="text-sm text-red-500">{$errors.projectName._errors[0]}</p>
				{/if}
			</div>

			<div class="space-y-4">
				<Label>Business</Label>
				<RadioGroup.Root bind:value={$formData.businessChoice} class="grid grid-cols-2 gap-4">
					<Label
						class="flex cursor-pointer flex-col items-center justify-center rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground [&:has([data-state=checked])]:border-primary"
					>
						<RadioGroup.Item value="existing" class="sr-only" />
						<span>Use Existing</span>
					</Label>
					<Label
						class="flex cursor-pointer flex-col items-center justify-center rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground [&:has([data-state=checked])]:border-primary"
					>
						<RadioGroup.Item value="new" class="sr-only" />
						<span>Create New</span>
					</Label>
				</RadioGroup.Root>
			</div>

			{#if $formData.businessChoice === 'existing'}
				<div class="space-y-2">
					<Label for="existingBusiness">Select a Business</Label>
					<Select.Root bind:value={$formData.existingBusinessId} type="single">
						<Select.Trigger class="w-full" aria-invalid={!!$errors?.existingBusinessId}>
							{#if $formData.existingBusinessId}
								{businesses.find((b) => b.id.toString() === $formData.existingBusinessId)?.name}
							{:else}
								Select a business...
							{/if}
						</Select.Trigger>
						<Select.Content>
							{#each businesses as business}
								<Select.Item value={business.id.toString()}>{business.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
					{#if $errors?.existingBusinessId?._errors}
						<p class="text-sm text-red-500">{$errors.existingBusinessId._errors[0]}</p>
					{/if}
				</div>
			{:else}
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="newBusinessName">New Business Name</Label>
						<Input
							id="newBusinessName"
							bind:value={$formData.newBusinessName}
							placeholder="e.g., 'My Online Store'"
						/>
						{#if $errors?.newBusinessName?._errors}
							<p class="text-sm text-red-500">{$errors.newBusinessName._errors[0]}</p>
						{/if}
					</div>
					<div class="space-y-2">
						<Label for="newBusinessShortCode">Shortcode</Label>
						<Input
							id="newBusinessShortCode"
							bind:value={$formData.newBusinessShortCode}
							placeholder="e.g., 600988"
						/>
						{#if $errors?.newBusinessShortCode?._errors}
							<p class="text-sm text-red-500">{$errors.newBusinessShortCode._errors[0]}</p>
						{/if}
					</div>
				</div>
			{/if}

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="simulationMode">Simulation Mode</Label>
					<Select.Root bind:value={$formData.simulationMode} type="single">
						<Select.Trigger class="w-full">
							{$formData.simulationMode}
						</Select.Trigger>
						<Select.Content>
							{#each Object.values(SimulationMode) as mode}
								<Select.Item value={mode}>{mode}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
				<div class="space-y-2">
					<Label for="stkDelay">STK Push Delay (ms)</Label>
					<Input id="stkDelay" type="number" bind:value={$formData.stkDelay} />
					{#if $errors?.stkDelay?._errors}
						<p class="text-sm text-red-500">{$errors.stkDelay._errors[0]}</p>
					{/if}
				</div>
			</div>
		</CardContent>
		<CardFooter class="flex justify-end">
			<Button onclick={handleSubmit} disabled={isLoading}>
				{#if isLoading}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Creating...
				{:else}
					Create Project <ArrowRight class="ml-2 h-4 w-4" />
				{/if}
			</Button>
		</CardFooter>
	</Card>
</main>
