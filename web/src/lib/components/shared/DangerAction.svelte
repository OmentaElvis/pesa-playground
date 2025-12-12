<script lang="ts">
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { Button } from '$lib/components/ui/button';
	import { Loader2, AlertTriangle } from 'lucide-svelte';

	type Props = {
		title: string;
		description: string;
		buttonLabel: string;
		dialogTitle?: string;
		dialogDescription: string;
		onConfirm: () => Promise<void>;
	};

	let {
		title,
		description,
		buttonLabel,
		dialogTitle = 'Are you absolutely sure?',
		dialogDescription,
		onConfirm
	}: Props = $props();

	let open = $state(false);
	let loading = $state(false);

	async function handleConfirm() {
		loading = true;
		try {
			await onConfirm();
		} finally {
			loading = false;
			open = false;
		}
	}
</script>

<div class="flex items-start justify-between rounded-lg border border-destructive/50 bg-destructive/5 p-4">
	<div class="flex items-start gap-4">
		<AlertTriangle class="h-6 w-6 text-destructive" />
		<div>
			<h3 class="font-semibold text-destructive">{title}</h3>
			<p class="text-sm text-muted-foreground">
				{description}
			</p>
		</div>
	</div>
	<Button variant="destructive" onclick={() => (open = true)}>
		<AlertTriangle class="mr-2 h-4 w-4" />
		{buttonLabel}
	</Button>
</div>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{dialogTitle}</AlertDialog.Title>
			<AlertDialog.Description>
				{dialogDescription}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={loading}>Cancel</AlertDialog.Cancel>
			<Button variant="destructive" onclick={handleConfirm} disabled={loading}>
				{#if loading}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				{/if}
				Continue
			</Button>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
