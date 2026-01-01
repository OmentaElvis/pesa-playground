<script lang="ts">
	import { PaneGroup, Pane } from 'paneforge';

	import {
		Circle,
		CheckCircle,
		XCircle,
		Loader2,
		AlertCircle,
		LayoutGrid,
		FilesIcon,
		TestTubeDiagonal,
		LoaderCircle,
		ClipboardCopy,
		Check,
		CircleSlash
	} from 'lucide-svelte';
	import { cn, copyToClipboard } from '$lib/utils';
	import { onMount } from 'svelte';
	import {
		listen,
		startSelfTest,
		type TestMode,
		type TestStatus,
		TestEvents,
		type TestStepInfo,
		type TestPlanEvent,
		type TestStepUpdateEvent,
		type TestProgressLogEvent,
		type TestFinishEvent
	} from '$lib/api';
	import * as Item from '$lib/components/ui/item/index.js';
	import { urlStateManager } from '$lib/utils/urlState';
	import { writable, type Writable } from 'svelte/store';
	import { Button } from '$lib/components/ui/button';
	import PesaPlaygroundLogo from '$lib/components/logo/PesaPlaygroundLogo.svelte';

	// Augment TestStepInfo for local UI state
	interface AugmentedTestStepInfo extends TestStepInfo {
		inferredStatus: TestStatus | 'cancelled';
	}

	let uiState: 'idle' | 'running' | 'passed' | 'failed' = $state('idle');
	let mode: Writable<TestMode | null> = writable(null);
	let tests: AugmentedTestStepInfo[] = $state([]);
	let selectedTestIndex: number = $state(0);
	let runner_logs: String[] = $state([]);
	let disableAutoSelect = $state(false);
	let copied = $state(false);

	// Reactive declaration to find the selected test object
	let selectedTest = $derived(tests[selectedTestIndex] ?? null);
	let failedTest = $derived(
		tests.find((t) => t.status === 'failed' || t.status === 'panicked' || t.status === 'timed_out')
	);

	onMount(() => {
		const unlistenMap = new Map<string, () => void>();
		const { destroy } = urlStateManager.sync({
			mode: mode
		});

		async function listenToEvents() {
			const listeners = {
				self_test_plan: await listen<TestPlanEvent>(TestEvents.Plan, ({ payload }) => {
					tests = payload.steps.map((step) => {
						return {
							name: step.name,
							description: step.description,
							status: 'pending',
							message: '',
							logs: [],
							inferredStatus: 'pending' // Initialize inferred status
						};
					});

					uiState = 'running';
				}),

				self_test_step_update: await listen<TestStepUpdateEvent>(
					TestEvents.StepUpdate,
					({ payload }) => {
						const { index, name, status, message } = payload;
						let hasFailed = false;

						// Update the current test
						if (tests[index] && tests[index].name === name) {
							tests[index].status = status;
							tests[index].message = message || '';
							tests[index].inferredStatus = status; // Update inferred status

							if (status === 'failed' || status === 'panicked' || status === 'timed_out') {
								hasFailed = true;
							}
						}

						// If the current test failed, mark all subsequent tests as cancelled
						if (hasFailed) {
							for (let i = index + 1; i < tests.length; i++) {
								if (tests[i].inferredStatus === 'pending') {
									// Only cancel if still pending
									tests[i].inferredStatus = 'cancelled';
								}
							}
						}

						// Auto-select the running or failed test
						if (
							(tests[index].status === 'running' ||
								tests[index].status === 'failed' ||
								tests[index].status === 'panicked' ||
								tests[index].status === 'timed_out') &&
							!disableAutoSelect
						) {
							selectedTestIndex = index;
						}
						// Trigger reactivity for the entire array
						tests = [...tests];
					}
				),

				self_test_progress_log: await listen<TestProgressLogEvent>(
					TestEvents.ProgressLog,
					(event) => {
						const { name, message, runner, index } = event.payload;
						if (name === 'main' && runner == true) {
							runner_logs.push(message);
							return;
						}

						const test = tests[index];
						if (test) {
							test.logs.push(`${message}\n`);
							// This is needed to trigger reactivity for the log view
							tests = [...tests];
						}
					}
				),

				self_test_finish: await listen<TestFinishEvent>(TestEvents.Finish, ({ payload }) => {
					// This will be 'passed' or 'failed'
					switch (payload.status) {
						case "failed":
						case "panicked":
						case "timed_out":
							 uiState = "failed";
							 break;
						case "running":
						case "pending":
							uiState = "running";
							break;
						case "passed":
						   uiState = "passed";
					}
					// If the suite finished, but some tests are still pending, mark them as cancelled
					for (let i = 0; i < tests.length; i++) {
						if (tests[i].inferredStatus === 'pending') {
							tests[i].inferredStatus = 'cancelled';
						}
					}
					tests = [...tests]; // Trigger reactivity
				})
			};

			for (const [key, unlisten] of Object.entries(listeners)) {
				unlistenMap.set(key, unlisten);
			}
		}

		listenToEvents();

		return () => {
			for (const unlisten of unlistenMap.values()) {
				unlisten();
			}

			destroy();
		};
	});

	function getStatusIcon(test: AugmentedTestStepInfo) {
		switch (test.inferredStatus) {
			case 'passed':
				return CheckCircle;
			case 'failed':
			case 'panicked':
				return XCircle;
			case 'running':
				return Loader2;
			case 'timed_out':
				return AlertCircle;
			case 'pending':
				return Circle;
			case 'cancelled':
				return CircleSlash;
			default:
				return Circle;
		}
	}

	function getStatusColorClass(test: AugmentedTestStepInfo) {
		switch (test.inferredStatus) {
			case 'passed':
				return 'text-green-900 fill-green-500';
			case 'failed':
			case 'panicked':
				return 'text-red-500';
			case 'running':
				return 'text-orange-500';
			case 'pending':
				return 'text-muted-foreground';
			case 'timed_out':
				return 'text-yellow-500';
			case 'cancelled':
				return 'text-muted-foreground'; // Muted color for cancelled
			default:
				return 'text-muted-foreground';
		}
	}

	async function startTests(newMode: TestMode) {
		startSelfTest(newMode);
		mode.set(newMode);
	}

	function selectTest(index: number) {
		selectedTestIndex = index;
		disableAutoSelect = true;
	}

	function handleCopy() {
		if (!selectedTest || selectedTest.logs.length === 0) return;

		// If there's an error message, prepend it to the logs
		let textToCopy = '';
		if (
			(selectedTest.status === 'failed' ||
				selectedTest.status === 'panicked' ||
				selectedTest.status === 'timed_out') &&
			selectedTest.message
		) {
			textToCopy += `Error: ${selectedTest.message}\n\n------\n\n`;
		}
		textToCopy += selectedTest.logs.join('');

		copyToClipboard(textToCopy);

		copied = true;
		setTimeout(() => {
			copied = false;
		}, 2000);
	}
</script>

<div class="p-4 lg:p-8 h-full flex flex-col">
	{#if $mode != 'Fresh' && $mode != 'Clone'}
		<div class="flex-grow flex flex-col items-center justify-center space-y-6 text-center">
			<div class="space-y-2">
				<div class="flex justify-center">
					<PesaPlaygroundLogo width="256" variant="color" height="128" /></div>
				<h2 class="text-3xl font-bold tracking-tight">Self Diagnostic</h2>
				<p class="text-muted-foreground max-w-xl">
					Choose a test mode to run a series of automated checks against the application's core
					components. This tests will emulate transactions, api interactions and other core functionality interactions.
					These tests are not expected to fail.
				</p>
			</div>
			<div class="flex w-full max-w-xl flex-col gap-6">
				<Item.Group class="grid grid-cols-2 gap-4 text-center">
					<Item.Root
						variant="outline"
						class="cursor-pointer items-center hover:border-green-500"
						onclick={() => startTests('Fresh')}
					>
						<Item.Header class="flex justify-center items-center">
							<LayoutGrid size="48" />
						</Item.Header>
						<Item.Content>
							<Item.Title class="text-xl text-center mx-auto">Fresh</Item.Title>
							<Item.Description>Do diagnostics on fresh instance of the app</Item.Description>
						</Item.Content>
					</Item.Root>
					<Item.Root
						variant="outline"
						class="cursor-pointer items-center hover:border-green-500"
						onclick={() => startTests('Clone')}
					>
						<Item.Header class="flex justify-center items-center">
							<FilesIcon size="48" />
						</Item.Header>
						<Item.Content>
							<Item.Title class="text-xl text-center mx-auto">Clone</Item.Title>
							<Item.Description
								>Clones your existing database and configurations and starts tests on that
								copy.</Item.Description
							>
						</Item.Content>
					</Item.Root>
				</Item.Group>
			</div>
		</div>
	{:else}
		<div class="flex items-center justify-between space-y-2">
			<div>
				<h2 class="text-2xl font-bold tracking-tight">Diagnostic Tests: {$mode}</h2>
				<p class="text-muted-foreground">
					Test execution is {uiState}.
				</p>
			</div>
		</div>
		<div
			class={cn('flex flex-row gap-2 items-center p-2 mb-2 text-sm', {
				'text-red-500': uiState === 'failed'
			})}
		>
			<div>
				{#if uiState == 'running'}
					<LoaderCircle class="animate-spin" />
				{:else}
					<TestTubeDiagonal />
				{/if}
			</div>
			<div class="text-sm">
				{#if uiState === 'failed' && failedTest}
					Execution failed at step: <span class="font-bold">{failedTest.name}</span>
				{:else if runner_logs && runner_logs.length > 0}
					{runner_logs[runner_logs.length - 1]}
				{:else}
					Waiting for test events
				{/if}
			</div>
		</div>
		<PaneGroup direction="horizontal" class="flex-grow">
			<Pane defaultSize={30} minSize={20}>
				<div class="flex flex-col h-full space-y-4 pr-2">
					<div class="border rounded-md bg-muted/20 flex-grow overflow-y-auto">
						<ul class="divide-y divide-border">
							{#each tests as test, index (test.name)}
								{@const Icon = getStatusIcon(test)}
								<button
									class={cn(
										'flex gap-4 p-2 items-center text-left text-sm cursor-pointer w-full',
										selectedTestIndex === index && 'bg-primary/10 text-accent-foreground',
										(test.inferredStatus === 'pending' || test.inferredStatus === 'cancelled') && 'opacity-50 cursor-not-allowed',
										(test.inferredStatus === 'passed') && 'bg-green-500/10',
										(test.inferredStatus == "pending") && 'bg-orange-500/10'
									)}
									disabled={test.inferredStatus === 'pending' || test.inferredStatus === 'cancelled'}
									onclick={() => selectTest(index)}
									onkeydown={(e) => e.key === 'Enter' && selectTest(index)}
								>
									<Icon
										class={cn(
											getStatusColorClass(test),
											test.inferredStatus === 'running' && 'animate-spin'
										)}
										size={16}
									/>
									<span
										class={cn(
											'truncate',
											test.inferredStatus === 'passed' && 'text-green-700 dark:text-green-500 font-bold'
										)}>{test.name}</span
									>
								</button>
							{/each}
						</ul>
					</div>
				</div>
			</Pane>
			<Pane>
				<div class="flex flex-col h-full border rounded-md bg-muted/20">
					{#if selectedTest}
						<div class="p-3 border-b bg-muted/50 flex items-center justify-between">
							<div>
								<h3 class="font-semibold">{selectedTest.name}</h3>
								<p class="text-sm text-muted-foreground">{selectedTest.description}</p>
							</div>
							<Button
								variant="ghost"
								size="icon"
								class="text-muted-foreground"
								disabled={!selectedTest.logs || selectedTest.logs.length === 0}
								onclick={handleCopy}
							>
								{#if copied}
									<Check class="h-4 w-4" />
								{:else}
									<ClipboardCopy class="h-4 w-4" />
								{/if}
							</Button>
						</div>
						<div class="flex-grow overflow-y-auto">
							{#if selectedTest.inferredStatus === 'failed' || selectedTest.inferredStatus === 'panicked' || selectedTest.inferredStatus === 'timed_out'}
								{#if selectedTest.message && selectedTest.message.length > 0}
									<div
										class="bg-destructive/10 border-l-4 border-destructive text-destructive p-3 m-2 rounded-md"
									>
										<p class="font-bold mb-1">Error Details:</p>
										<p class="whitespace-pre-wrap text-sm font-mono">
											{selectedTest.message}
										</p>
									</div>
								{/if}
							{/if}
							<div class="text-xs whitespace-pre-wrap p-4 text-left font-mono">
								{#each selectedTest.logs as log}
									{log}
								{:else}
									<code>No logs for this step yet.</code>
								{/each}
							</div>
						</div>
					{:else}
						<div class="flex-grow flex items-center justify-center">
							<p class="text-muted-foreground">Select a test step to see its logs.</p>
						</div>
					{/if}
				</div>
			</Pane>
		</PaneGroup>
	{/if}
</div>
