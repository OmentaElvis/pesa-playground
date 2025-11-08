<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Plus } from 'lucide-svelte';

	export let businessId: number;
</script>

<Card class="h-full border-2 border-dashed transition-colors hover:border-primary/50">
	<CardContent class="flex h-full flex-col items-center justify-center py-12">
		<div class="space-y-4 text-center">
			<div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
				<Plus class="h-8 w-8 text-primary/60" />
			</div>
			<div>
				<h3 class="font-semibold text-foreground">Create a New Project</h3>
				<p class="mt-1 text-sm text-muted-foreground">Set up a new testing environment.</p>
			</div>
			<Button href="/projects/new/{businessId}" variant="outline" class="gap-2">
				<Plus class="h-4 w-4" />
				New Project
			</Button>
		</div>
	</CardContent>
</Card>
