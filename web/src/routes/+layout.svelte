<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { ArrowRight, ArrowLeft, History } from 'lucide-svelte';
	import '../app.css';
	import { Button } from '$lib/components/ui/button/index.js';
	import { ModeWatcher } from 'mode-watcher';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import { toggleMode } from 'mode-watcher';
	import { title } from '$lib/stores/title';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import StkPushDialog from '$lib/components/StkPushDialog.svelte';
	import { listen, type UnlistenFn } from '$lib/api';
	import { onMount } from 'svelte';
	import { resolveStkPrompt } from '$lib/api';
	import { Toaster } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Pane, PaneGroup as PaneForge } from 'paneforge';
	import SmartSidebar from '$lib/components/SmartSidebar.svelte';
	import { sidebarStore } from '$lib/stores/sidebarStore';
	import ApiLogWidget from '$lib/components/widgets/ApiLogWidget.svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { footerWidgetStore } from '$lib/stores/footerWidgetStore';
	import SplashScreen from '$lib/components/SplashScreen.svelte';
	import { closeSplashscreen, isApiReady } from '$lib/api';
	import { createKeymapManager, type KeymapManager } from '$lib/keymap';
	import {
		generateProjectSandboxKeymaps,
		globalKeymapActions,
		back,
		forward
	} from '$lib/actions/keymapActions';
	import TransactionsNotification from '$lib/components/TransactionsNotification.svelte';

	const keymapManager: KeymapManager = createKeymapManager();

	let WindowControls: any = $state(null);
	if (import.meta.env.MODE === 'tauri') {
		import('$lib/components/WindowControls.svelte').then((mod) => {
			WindowControls = mod.default;
		});
	}

	let innerWidth = $state(0);
	// 42 px
	let minSmartbarWidth = $derived(Math.ceil((42 / innerWidth) * 100));
	let width = $derived(
		new Tween(minSmartbarWidth, {
			duration: 200,
			easing: cubicOut
		})
	);

	let { children } = $props();

	let stkPush: any = $state(null);
	let stkPushOpened: boolean = $state(false);
	let titlebar: HTMLDivElement | null = $state(null);
	let rightPane: Pane | undefined = $state();

	const unlistenFunctions: UnlistenFn[] = [];
	let showSplash = $state(true); // State for splash screen visibility

	onMount(async () => {
		const projectSandboxKeymaps = await generateProjectSandboxKeymaps();
		const allAppKeymapActions = [...globalKeymapActions, ...projectSandboxKeymaps];
		keymapManager.initialize(allAppKeymapActions);
		// Hide splash screen after a delay
		setTimeout(() => {
			showSplash = false;
			if (import.meta.env.MODE === 'tauri') {
				closeSplashscreen();
			}
		}, 1000); // Adjust delay as needed

		sidebarStore.register({
			id: 'api-logs',
			title: 'API Logs',
			icon: History,
			component: ApiLogWidget
		});

		// Pass URL params to store for initialization
		sidebarStore.initFromUrl(page.url.searchParams);

		// Set a timeout to resolve any pending widget that was not registered
		setTimeout(() => {
			sidebarStore.resolvePending();
		}, 500);

		listen('stk_push', (e) => {
			stkPush = e.payload;
			stkPushOpened = true;
		})
			.then((un) => {
				unlistenFunctions.push(un);
			})
			.catch((err) => {
				console.log(err);
			});
	});

	const { isCollapsed, activeWidget } = sidebarStore;

	// Effect to sync store state to URL
	$effect(() => {
		if (!page.url) return; // Ensure page store is hydrated

		const widgetId = $activeWidget?.id;
		const collapsed = $isCollapsed;
		const params = new URLSearchParams(page.url.searchParams);

		if (widgetId && !collapsed) {
			params.set('widget', widgetId);
		} else {
			params.delete('widget');
		}

		if (collapsed) {
			params.set('collapsed', 'true');
		} else {
			params.delete('collapsed');
		}

		const newSearch = params.toString().replace(/=$/, '');
		const currentSearch = page.url.search.replace(/^\?/, '');

		if (newSearch !== currentSearch) {
			const url = `${page.url.pathname}${newSearch ? `?${newSearch}` : ''}`;
			goto(url, {
				replaceState: true,
				noScroll: true,
				keepFocus: true
			});
		}
	});

	// Reactive effect to control pane size
	$effect(() => {
		if (!rightPane) return;

		if ($isCollapsed) {
			width.target = minSmartbarWidth;
		} else {
			width.target = Math.ceil((350 / innerWidth) * 100);
		}
	});

	async function stkPushAction(e: CustomEvent) {
		let detail: { action: string; checkout_id: string } = e.detail;
		let action: string = detail.action;
		let checkout_id: string = detail.checkout_id;

		let pin = stkPush.user.pin;
		if (action === 'correct_pin') {
			resolveStkPrompt(checkout_id, { accepted: { pin } });
		} else if (action === 'wrong_pin') {
			resolveStkPrompt(checkout_id, { accepted: { pin: pin + 'gg' } });
		} else if (action === 'user_online') {
			resolveStkPrompt(checkout_id, 'offline');
		} else if (action === 'timeout') {
			resolveStkPrompt(checkout_id, 'timeout');
		} else {
			resolveStkPrompt(checkout_id, 'cancelled');
		}
	}
</script>

<svelte:window bind:innerWidth on:keydown={keymapManager.handleKeyDown} />

{#if showSplash}
	<SplashScreen show={showSplash} />
{/if}

{#if $isApiReady}
	<ModeWatcher />
	<Sidebar.Provider>
		<div
			class="fixed top-0 left-0 z-1000 flex h-[36px] w-full items-center bg-green-800 pr-4 text-white shadow-lg transition-colors duration-500 select-none"
		>
			<Sidebar.Trigger class="cursor-pointer" />
			<div bind:this={titlebar} class="basis-full font-bold">{$title}</div>
			<Button onclick={toggleMode} variant="ghost" size="icon">
				<SunIcon
					class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90"
				/>
				<MoonIcon
					class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0"
				/>
				<span class="sr-only">Toggle theme</span>
			</Button>
			{#if WindowControls}
				<WindowControls {titlebar} />
			{/if}
		</div>
		<AppSidebar variant="sidebar" />

		<div class="mt-[36px] mb-[32px] h-[calc(100vh-72px)] w-full">
			<div class="h-full">
				<PaneForge direction="horizontal" class="h-full w-full">
					<Pane id="main-content" class="h-full">
						<div class="h-full overflow-y-auto">
							{@render children()}
						</div>
					</Pane>
					<Pane
						id="right-sidebar"
						bind:this={rightPane}
						defaultSize={width.current}
						minSize={width.current}
						collapsible={true}
						collapsedSize={minSmartbarWidth}
						onCollapse={() => isCollapsed.set(true)}
						onExpand={() => isCollapsed.set(false)}
						class="h-full overflow-hidden"
					>
						<SmartSidebar />
					</Pane>
				</PaneForge>
			</div>
		</div>

		<div
			class="fixed bottom-0 z-1000 flex h-[36px] w-full items-center justify-between border bg-muted px-2"
		>
			<div>
				<Button variant="ghost" onclick={back} class="cursor-pointer" aria-label="back">
					<ArrowLeft />
				</Button>
				<Button variant="ghost" onclick={forward} class="cursor-pointer" aria-label="foward">
					<ArrowRight />
				</Button>
			</div>

			<div class="flex flex-1 gap-4">
				{#each $footerWidgetStore as widget (widget.id)}
					<widget.component {...widget.props || {}} />
				{/each}
			</div>

			<div>
				<TransactionsNotification />
			</div>
		</div>
	</Sidebar.Provider>
	<StkPushDialog bind:open={stkPushOpened} dialogData={stkPush} on:action={stkPushAction} />
	<Toaster position="top-right" richColors offset="40px" />
{/if}

<style>
	:global(body) {
		overflow: hidden;
	}
</style>
