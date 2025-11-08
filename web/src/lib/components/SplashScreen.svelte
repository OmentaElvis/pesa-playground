<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	export let show: boolean = true;
</script>

{#if show}
	<div
		class="fixed inset-0 z-[9999] flex items-center justify-center bg-secondary transition-opacity duration-500 ease-out"
	>
		<div class="flex flex-col items-center gap-4">
			<img
				src="/pesaplayground_logo.svg"
				alt="Pesa Playground Logo"
				class="max-h-[80vh] max-w-[80%]"
			/>

			<LoaderCircle class="size-12 animate-spin dark:text-green-600" />
		</div>
	</div>
{/if}
