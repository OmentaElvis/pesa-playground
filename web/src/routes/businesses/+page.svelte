<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { getBusinesses } from '$lib/api';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let businesses: any[] = [];

	async function loadBusinesses() {
		businesses = await getBusinesses();
	}

	onMount(() => {
		loadBusinesses();
	});
</script>

<div class="space-y-6 p-6 md:block">
	<div class="space-y-0.5">
		<div class="space-between mb-10 flex w-full items-center justify-between">
			<div>
				<h2 class="text-3xl font-bold tracking-tight">Businesses</h2>
				<p class="text-muted-foreground">Manage your businesses, accounts, and projects.</p>
			</div>
		</div>
		<div class="space-y-6">
			<div class="grid gap-4 md:grid-cols-3 lg:grid-cols-4">
				{#each businesses as business}
					<Card>
						<CardHeader>
							<CardTitle>{business.name}</CardTitle>
							<CardDescription>{business.short_code}</CardDescription>
						</CardHeader>
						<CardContent>
							<Button onclick={() => goto(`/businesses/${business.id}`)} class="mt-4">
								View Details
							</Button>
						</CardContent>
					</Card>
				{/each}
			</div>
		</div>
	</div>
</div>
