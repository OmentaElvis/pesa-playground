<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { ArrowRight, ArrowLeft, Bell, History } from 'lucide-svelte';
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { formatAmount } from "$lib/utils";
	import '../app.css';
	import { Button } from "$lib/components/ui/button/index.js";
  import { ModeWatcher } from "mode-watcher";
  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import { toggleMode } from "mode-watcher";
  import { title } from '$lib/stores/title';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import StkPushDialog from '$lib/components/StkPushDialog.svelte';
  import { listen, type UnlistenFn } from "$lib/api";
  import { onDestroy, onMount } from "svelte";
  import { resolveStkPrompt, type FullTransactionLog, resolveAccountAndNavigate } from "$lib/api";
  import { transactionLogStore } from '$lib/stores/transactionLogStore';
  import { Toaster } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import { Pane, PaneGroup as PaneForge} from "paneforge";
  import SmartSidebar from "$lib/components/SmartSidebar.svelte";
  import { sidebarStore } from "$lib/stores/sidebarStore";
  import ApiLogWidget from "$lib/components/widgets/ApiLogWidget.svelte";
  import { Tween } from 'svelte/motion';
  import { cubicOut } from "svelte/easing";

  let WindowControls: any = $state(null);
  if (import.meta.env.MODE === 'tauri') {
  	import("$lib/components/WindowControls.svelte").then((mod)=> {
  		WindowControls = mod.default;
  	});
  }

  let innerWidth = $state(0);
  // 42 px
  let minSmartbarWidth = $derived(Math.ceil((42 / innerWidth) * 100));
  let width = $derived(new Tween(minSmartbarWidth, {
  	duration: 200,
  	easing: cubicOut
  }));

	let { children } = $props();

	let stkPush: any = $state(null);
	let stkPushOpened : boolean = $state(false);
	let titlebar: HTMLDivElement | null = $state(null);
  let rightPane: Pane | undefined = $state();

  const unlistenFunctions: UnlistenFn[] = [];

  onMount(() => {
    sidebarStore.register({
      id: 'api-logs',
      title: 'API Logs',
      icon: History,
      component: ApiLogWidget,
    });

    if (innerWidth > 1000) {
    	sidebarStore.setActiveWidget('api-logs');
    }

    listen('stk_push', (e)=> {
      stkPush = e.payload;
      stkPushOpened = true;
    }).then((un)=> {
      unlistenFunctions.push(un);
    }).catch((err)=> {
      console.log(err)
    });

    listen<FullTransactionLog>('new_transaction', (e) => {
      console.log('New transaction log received', e.payload);
      transactionLogStore.add(e.payload);
    }).then((un) => {
      unlistenFunctions.push(un);
    }).catch((err) => {
      console.error('Failed to set up new_transaction listener:', err);
    });
  });

  const { isCollapsed } = sidebarStore;

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
  	let detail: {action: string, checkout_id: string} = e.detail;
  	let action: string = detail.action;
  	let checkout_id: string =  detail.checkout_id;

  	let pin = stkPush.user.pin;
  	if (action === "correct_pin") {
  		resolveStkPrompt(checkout_id, {accepted: {pin}})
  	} else if (action === "wrong_pin") {
  		resolveStkPrompt(checkout_id, {accepted: {pin: pin+"gg"}})
  	} else if (action === "user_online") {
  		resolveStkPrompt(checkout_id, "offline")
  	} else if (action === "timeout") {
  		resolveStkPrompt(checkout_id, "timeout")
  	} else {
  		resolveStkPrompt(checkout_id, "cancelled")
  	}
  }

  onDestroy(()=> {
  	unlistenFunctions.forEach((unlisten) => unlisten());
  })

  function forward() {
  	window.history.forward();
  }

  function back() {
  	window.history.back();
  }

  function getTransactionLogDescription(log: FullTransactionLog): string {
    if (log.transaction_type === 'Deposit') {
      return 'Deposit';
    } else if (log.direction === 'Credit') {
      return 'Received';
    } else if (log.direction === 'Debit') {
      if (log.transaction_type === 'send_money') {
        return 'Sent';
      } else if (log.transaction_type === 'paybill' || log.transaction_type === 'buy_goods') {
        return 'Paid';
      } else if (log.transaction_type === 'withdraw') {
        return 'Withdrawn';
      }
    }
    return 'Transacted'; // Fallback
  }

  function getTransactionLogSummarySentence(log: FullTransactionLog): string {
    const formattedAmount = formatAmount(log.transaction_amount / 100);

    if (log.transaction_type === 'Deposit') {
        return `Deposit of ${formattedAmount} to ${log.to_name}`;
    } else if (log.direction === 'Credit') {
        return `${log.to_name} received ${formattedAmount} from ${log.from_name}`;
    } else if (log.direction === 'Debit') {
        if (log.transaction_type === 'send_money') {
            return `${log.from_name} sent ${formattedAmount} to ${log.to_name}`;
        } else if (log.transaction_type === 'paybill') {
            return `${log.from_name} paid ${formattedAmount} to Pay Bill ${log.to_name}`;
        } else if (log.transaction_type === 'buy_goods') {
            return `${log.from_name} paid ${formattedAmount} to Buy Goods ${log.to_name}`;
        } else if (log.transaction_type === 'withdraw') {
            return `${log.from_name} withdrew ${formattedAmount}`;
        }
    }
    return `Transaction of ${formattedAmount} between ${log.from_name} and ${log.to_name}`; // Fallback
  }
	
</script>

<svelte:window bind:innerWidth />

<ModeWatcher />
<Sidebar.Provider>
	<div class="fixed top-0 transition-colors duration-500 left-0 pr-4 select-none items-center flex w-full h-[36px] shadow-lg z-1000 bg-green-800 text-white" >
		<Sidebar.Trigger class="cursor-pointer" />
		<div bind:this={titlebar} class="basis-full font-bold">{$title}</div>
		<Button onclick={toggleMode} variant="ghost" size="icon">
		  <SunIcon
		    class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
		  />
		  <MoonIcon
		    class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
		  />
		  <span class="sr-only">Toggle theme</span>
		</Button>
		{#if WindowControls}
			<WindowControls {titlebar} />
		{/if}
	</div>
	<AppSidebar variant="sidebar" />

  <div class="mt-[36px] mb-[32px] h-[calc(100vh-72px)] w-full">
    <div class="h-full" >
      <PaneForge direction="horizontal" class="w-full h-full">
        <Pane id="main-content" class="h-full overflow-y-auto">
          {@render children()}
        </Pane>
        <Pane id="right-sidebar"
        	    bind:this={rightPane}
        	    defaultSize={width.current}
        	    minSize={width.current}
        	    collapsible={true}
        	    collapsedSize={minSmartbarWidth}
        	    onCollapse={() => isCollapsed.set(true)}
        	    onExpand={() => isCollapsed.set(false)}
        	    class="h-full overflow-hidden">
          <SmartSidebar />
        </Pane>
      </PaneForge>
    </div>
  </div>

	<div class="bg-muted h-[36px] fixed w-full bottom-0 border z-1000 flex items-center justify-between px-2">
		<div>
			<Button variant="ghost" onclick={back} class="cursor-pointer" aria-label="back"> <ArrowLeft /></Button>
			<Button variant="ghost" onclick={forward} class="cursor-pointer" aria-label="foward"> <ArrowRight /></Button>
		</div>
		<div>
			<Popover.Root>
				<Popover.Trigger>
					<Button variant="ghost" size="icon" class="relative">
						<Bell />
						{#if $transactionLogStore.length > 0}
							<div class="absolute top-1 right-1 h-3 w-3 rounded-full bg-red-500 border-2 border-muted" ></div>
						{/if}
					</Button>
				</Popover.Trigger>
				<Popover.Content class="w-96">
					<div class="flex justify-between items-center mb-2">
						<h3 class="font-medium">Unread Transactions</h3>
						<Button variant="link" size="sm" onclick={() => transactionLogStore.reset()}>Clear All</Button>
					</div>
					<ScrollArea class="h-72">
						<div class="flex flex-col gap-2">
							{#each $transactionLogStore as log (log.transaction_id + log.direction)}
								{@const account_id_to_visit = log.direction === 'Credit' ? log.to_id : log.from_id}
								{#if account_id_to_visit}
									<button class="block p-2 text-left rounded-md hover:bg-secondary text-sm" onclick={() => { resolveAccountAndNavigate(account_id_to_visit, goto); transactionLogStore.remove(log.transaction_id); }}>
										<div class="font-semibold">{getTransactionLogSummarySentence(log)}</div>
										<div class="flex justify-between items-center">
											<div class="flex items-center gap-1">
												<span class="font-semibold">{getTransactionLogDescription(log)}</span>
												<span class:text-green-500={log.direction === 'Credit'} class:text-red-500={log.direction === 'Debit'}>
													{log.direction === 'Credit' ? '+' : '-'}{formatAmount(log.transaction_amount / 100)}
												</span>
											</div>
											<span class="text-xs text-muted-foreground">{new Date(log.transaction_date).toLocaleTimeString()}</span>
										</div>
									</button>
								{/if}
							{:else}
								<div class="text-center text-muted-foreground p-4">No new transactions.</div>
							{/each}
						</div>
					</ScrollArea>
				</Popover.Content>
			</Popover.Root>
		</div>
	</div>
</Sidebar.Provider>
<StkPushDialog bind:open={stkPushOpened} dialogData={stkPush} on:action={stkPushAction} />
<Toaster position="top-right" richColors offset="40px" />

<style>
	:global(body) {
		overflow: hidden;
	}
</style>
