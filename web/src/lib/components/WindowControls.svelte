<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Button } from './ui/button';
	import { Maximize2, Minus, X } from 'lucide-svelte';

	const appWindow = getCurrentWindow();
	let props: {
		titlebar: HTMLDivElement;
	} = $props();

	let titlebar = props.titlebar;

	$effect(() => {
		if (titlebar) {
			titlebar.addEventListener('mousedown', (e) => {
				if (e.buttons === 1) {
					e.detail == 2 ? appWindow.toggleMaximize() : appWindow.startDragging();
				}
			});
		}
	});
</script>

<Button onclick={() => appWindow.minimize()} variant="ghost" class="rounded-none">
	<Minus />
</Button>
<Button onclick={() => appWindow.toggleMaximize()} variant="ghost" class="rounded-none">
	<Maximize2 />
</Button>
<Button onclick={() => appWindow.close()} variant="ghost" class="rounded-none hover:bg-red-500">
	<X />
</Button>
