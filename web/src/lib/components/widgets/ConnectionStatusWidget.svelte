<script lang="ts">
	import { connectionStore } from '$lib/stores/connectionStore';
	import { Wifi, WifiOff, LoaderCircle } from 'lucide-svelte';

	const statusInfo = {
		connecting: {
			icon: LoaderCircle,
			text: 'Connecting...',
			color: 'text-orange-600 font-bold',
			animate: true
		},
		connected: {
			icon: Wifi,
			text: 'Connected',
			color: 'text-green-600 font-bold',
			animate: false
		},
		disconnected: {
			icon: WifiOff,
			text: 'Connection lost',
			color: 'text-red-500',
			animate: false
		}
	};

	let currentStatusInfo = $derived(statusInfo[$connectionStore]);
	let showConnected = $state(false);
	let timeoutId: any = null;

	$effect(() => {
		if ($connectionStore === 'connected') {
			showConnected = true;
			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = setTimeout(() => {
				showConnected = false;
			}, 2000); // Show for 2 seconds then hide
		}

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
		};
	});
</script>

{#if $connectionStore !== 'connected' || showConnected}
	<div class="flex items-center gap-2 px-2 text-xs {currentStatusInfo.color}">
		<currentStatusInfo.icon class="h-4 w-4 {currentStatusInfo.animate ? 'animate-spin' : ''}" />
		{currentStatusInfo.text}
	</div>
{/if}
