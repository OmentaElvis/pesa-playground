<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { X, Maximize2, Minus, ArrowRight, ArrowLeft } from 'lucide-svelte';
	import '../app.css';
	import { Button } from "$lib/components/ui/button/index.js";
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ModeWatcher } from "mode-watcher";
  import SunIcon from "@lucide/svelte/icons/sun";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import { toggleMode } from "mode-watcher";
  import { title } from '$lib/stores/title';
	import AppSidebar from '$lib/components/AppSidebar.svelte';
	import StkPushDialog from '$lib/components/StkPushDialog.svelte';
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";
  import { resolveStkPrompt } from "$lib/api";
  import { Toaster } from "svelte-sonner";
    import { page } from "$app/state";
    import { onNavigate } from "$app/navigation";
  
	const appWindow = getCurrentWindow();
	let { children } = $props();

	let titlebar: HTMLDivElement | null = $state(null);

	$effect(()=> {
	 if (titlebar) {
		titlebar.addEventListener('mousedown', (e)=> {
			if (e.buttons === 1) {
				e.detail == 2 ? appWindow.toggleMaximize() : appWindow.startDragging();
			}
		});
	 };	
	});

	let stkPush: any = $state(null);
	let stkPushOpened : boolean = $state(false);

  let unlisten: UnlistenFn;
  listen('stk_push', (e)=> {
  	stkPush = e.payload;
  	stkPushOpened = true;
  }).then((un)=> {
  	unlisten = un;
  }).catch((err)=> {
  	console.log(err)
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
  	if (unlisten)
  		unlisten();
  })

  function forward() {
  	window.history.forward();
  }

  function back() {
  	window.history.back();
  }
	
</script>

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
	  <Button onclick={()=> appWindow.minimize()} variant="ghost" class="rounded-none" >
	  	<Minus />
	  </Button>
	  <Button onclick={()=> appWindow.toggleMaximize()} variant="ghost" class="rounded-none">
	  	<Maximize2 />
	  </Button>
	  <Button onclick={()=> appWindow.close()} variant="ghost" class="hover:bg-red-500 rounded-none">
	  	<X />
	  </Button>
	</div>
	<AppSidebar variant="sidebar" />
	<div class="mt-[36px] mb-[32px] h-[calc(100vh-72px)] w-full overflow-y-auto">
		{@render children()}
	</div>
	<div class="bg-muted h-[36px] fixed w-full bottom-0 border z-1000">
		<Button variant="ghost" onclick={back} class="cursor-pointer" aria-label="back"> <ArrowLeft /></Button>
		<Button variant="ghost" onclick={forward} class="cursor-pointer" aria-label="foward"> <ArrowRight /></Button>
	</div>
</Sidebar.Provider>
<StkPushDialog bind:open={stkPushOpened} dialogData={stkPush} on:action={stkPushAction} />
<Toaster position="top-right" richColors offset="40px" />
