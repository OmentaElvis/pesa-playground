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
	import * as Select from '$lib/components/ui/select';
	import { ArrowRight, FolderKanban, Landmark, Loader2, Plus } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { createForm } from 'felte';
	import { validator } from '@felte/validator-zod';
	import { toast } from 'svelte-sonner';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import {
		Field,
		FieldDescription,
		FieldGroup,
		FieldLabel,
		FieldSet
	} from '$lib/components/ui/field';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import Building_2 from 'lucide-svelte/icons/building-2';
	import { newProjectSchema, type NewProjectSchema } from '$lib/schema';
	import { Hash } from '@lucide/svelte';

	const { form, data, errors, isSubmitting } = createForm<NewProjectSchema>({
		extend: validator({ schema: newProjectSchema }),
		initialValues: {
			projectName: '',
			businessChoice: 'existing',
			existingBusinessId: undefined,
			newBusinessName: undefined,
			newBusinessShortCode: undefined,
			simulationMode: SimulationMode.Realistic,
			initialWorkingBalance: 10000,
			initialUtilityBalance: 500,
			stkDelay: 0
		},
		onSubmit: handleSubmit
	});

	let submitError: string | null = $state(null);
	let businesses: BusinessSummary[] = $state([]);

	async function handleSubmit(values: NewProjectSchema) {
		try {
			let businessId: number;
			submitError = null;

			if (values.businessChoice === 'new') {
				if (!values.newBusinessName || !values.newBusinessShortCode) {
					submitError = 'New business name and shortcode are required.';
					return;
				}

				const newBusiness = await createBusiness({
					name: values.newBusinessName,
					short_code: values.newBusinessShortCode,
					initial_utility_balance: values.initialUtilityBalance || 0,
					initial_working_balance: values.initialWorkingBalance || 0
				});
				businessId = newBusiness.id;
			} else {
				if (!values.existingBusinessId) {
					submitError = 'Please select an existing business.';
					return;
				}
				businessId = parseInt(values.existingBusinessId, 10);
			}

			const newProject = await createProject({
				name: values.projectName,
				business_id: businessId,
				simulation_mode: values.simulationMode,
				stk_delay: Number(values.stkDelay) || 0
			});

			toast.success('Project created successfully!');
			goto(`/projects/${newProject.id}`);
		} catch (error: any) {
			console.error('Failed to create project:', error);
			submitError = `Error creating project: ${error}`;
		}
	}

	onMount(async () => {
		businesses = await getBusinesses();
		if (businesses.length == 0) {
			data.update((d) => {
				d.businessChoice = 'existing';
				return d;
			});
		}
		const urlBusinessId = page.url.searchParams.get('business_id');

		if (businesses.length === 0) {
			data.set({ ...$data, businessChoice: 'new' });
		} else if (urlBusinessId && businesses.some((b) => b.id.toString() === urlBusinessId)) {
			data.set({ ...$data, existingBusinessId: urlBusinessId });
		} else {
			data.set({ ...$data, existingBusinessId: businesses[0]?.id.toString() });
		}
	});
</script>

{#snippet ErrorMessage(errors: any, field: keyof NewProjectSchema)}
	{#if errors && errors[field]}
		<p class="text-sm text-red-500">{errors[field][0]}</p>
	{/if}
{/snippet}

<main class="container mx-auto flex max-w-2xl flex-col items-center p-6">
	<Card class="w-full">
		<CardHeader>
			<CardTitle class="flex items-center gap-4"><FolderKanban /> Create a New Project</CardTitle>
			<CardDescription>
				A project is a workspace for your integration. Fill in the details below to get started.
			</CardDescription>
		</CardHeader>
		<form use:form>
			<CardContent class="space-y-6">
				<FieldSet>
					<FieldGroup>
						<Field>
							<FieldLabel for="projectName">Project Name</FieldLabel>
							<Input
								id="projectName"
								name="projectName"
								placeholder="e.g., 'Website Checkout Test'"
								value={$data.projectName}
								class="text-lg"
							/>
							{@render ErrorMessage($errors, 'projectName')}
						</Field>
					</FieldGroup>
				</FieldSet>
				<Tabs.Root bind:value={$data.businessChoice}>
					<Tabs.List>
						<Tabs.Trigger value="existing" disabled={businesses.length == 0}>
							<Building_2 /> Use existing
						</Tabs.Trigger>
						<Tabs.Trigger value="new"><Plus /> New business</Tabs.Trigger>
					</Tabs.List>

					<Tabs.Content value="existing">
						<FieldSet>
							<FieldGroup>
								<Field>
									<FieldLabel for="existingBusiness">Select a Business</FieldLabel>
									<Select.Root
										name="existingBusinessId"
										bind:value={$data.existingBusinessId}
										type="single"
									>
										<Select.Trigger class="w-full" aria-invalid={!!$errors.existingBusinessId}>
											{#if $data.existingBusinessId}
												{businesses.find((b) => b.id.toString() === $data.existingBusinessId)?.name}
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
									{@render ErrorMessage($errors, 'existingBusinessId')}
								</Field>
							</FieldGroup>
						</FieldSet>
					</Tabs.Content>
					<Tabs.Content value="new">
						<small class="mb-4 block text-muted-foreground">
							A business is the wallet that holds all funds for paybills and till accounts. A
							business can have multiple projects, you can always reuse this business on future
							projects.
						</small>
						<FieldSet>
							<FieldGroup class="grid grid-cols-2 gap-4">
								<Field>
									<FieldLabel for="newBusinessName">New Business Name</FieldLabel>
									<Input
										id="newBusinessName"
										name="newBusinessName"
										bind:value={$data.newBusinessName}
										placeholder="e.g., 'My Online Store'"
									/>
									{@render ErrorMessage($errors, 'newBusinessName')}
								</Field>
								<Field>
									<FieldLabel for="newBusinessShortCode">Shortcode</FieldLabel>
									<InputGroup>
										<InputGroupAddon align="inline-start">
											<Hash />
										</InputGroupAddon>
										<InputGroupInput
											id="newBusinessShortCode"
											name="newBusinessShortCode"
											bind:value={$data.newBusinessShortCode}
											placeholder="e.g., 600988"
										/>
									</InputGroup>
									{@render ErrorMessage($errors, 'newBusinessShortCode')}
								</Field>
							</FieldGroup>
							<FieldGroup class="grid grid-cols-2 gap-4">
								<Field>
									<FieldLabel for="initialWorkingBalance">Initial Working Balance</FieldLabel>
									<InputGroup>
										<InputGroupAddon align="inline-start">
											<Landmark />
										</InputGroupAddon>
										<InputGroupAddon>Ksh</InputGroupAddon>
										<InputGroupInput
											id="initialWorkingBalance"
											name="initialWorkingBalance"
											value={$data.initialWorkingBalance}
											type="number"
										/>
									</InputGroup>
									<FieldDescription>
										The starting balance for the business's main account.
									</FieldDescription>
									{@render ErrorMessage($errors, 'initialWorkingBalance')}
								</Field>
								<Field>
									<FieldLabel for="initialUtilityBalance">Initial Utility Balance</FieldLabel>
									<InputGroup>
										<InputGroupAddon align="inline-start">
											<Landmark />
										</InputGroupAddon>
										<InputGroupAddon>Ksh</InputGroupAddon>
										<InputGroupInput
											id="initialUtilityBalance"
											name="initialUtilityBalance"
											value={$data.initialUtilityBalance}
											type="number"
										/>
									</InputGroup>
									<FieldDescription>
										The starting balance for the business's utility account.
									</FieldDescription>
									{@render ErrorMessage($errors, 'initialUtilityBalance')}
								</Field>
							</FieldGroup>
						</FieldSet>
					</Tabs.Content>
				</Tabs.Root>

				<FieldSet>
					<FieldGroup class="grid grid-cols-2 gap-4">
						<Field>
							<FieldLabel for="simulationMode">Simulation Mode</FieldLabel>
							<Select.Root name="simulationMode" bind:value={$data.simulationMode} type="single">
								<Select.Trigger class="w-full">
									{$data.simulationMode}
								</Select.Trigger>
								<Select.Content>
									{#each Object.values(SimulationMode) as mode}
										<Select.Item value={mode}>{mode}</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</Field>
						<Field>
							<FieldLabel for="stkDelay">STK Push Delay (ms)</FieldLabel>
							<Input id="stkDelay" name="stkDelay" type="number" bind:value={$data.stkDelay} />
							<FieldDescription>
								The delay in milliseconds before the STK push is shown.
							</FieldDescription>
							{@render ErrorMessage($errors, 'stkDelay')}
						</Field>
					</FieldGroup>
				</FieldSet>
			</CardContent>
			<CardFooter class="flex flex-col">
				<div>
					{#if submitError}
						<div
							class="mb-4 max-h-48 w-full overflow-auto rounded-md border border-red-500/30 bg-red-500/5 p-3"
						>
							<pre
								class="font-mono text-sm break-words whitespace-pre-wrap text-red-700 dark:text-red-400">{submitError}</pre>
						</div>
					{/if}
				</div>
				<div class="flex w-full justify-end">
					<Button type="submit" disabled={$isSubmitting}>
						{#if $isSubmitting}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Creating...
						{:else}
							Create Project <ArrowRight class="ml-2 h-4 w-4" />
						{/if}
					</Button>
				</div>
			</CardFooter>
		</form>
	</Card>
</main>
