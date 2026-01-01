<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getAccount, getUser, AccountType, getMmfAccount, getUtilityAccount } from '$lib/api';
	import { LoaderCircle } from 'lucide-svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	let isLoading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		const id = Number(page.params.id);
		if (isNaN(id)) {
			error = 'Invalid account ID provided.';
			isLoading = false;
			return;
		}

		if (id == 0) {
			goto(`/accounts/system`, { replaceState: true });
			return;
		}

		try {
			const account = await getAccount(id);

			if (!account) {
				error = `Account with ID ${id} not found.`;
				isLoading = false;
				return;
			}

			switch (account.account_type) {
				case AccountType.User:
					const user = await getUser(account.id);
					if (user) goto(`/users/${user.account_id}`, { replaceState: true });
					else error = 'Associated user profile not found.';
					break;
				case AccountType.Mmf:
					const mmf = await getMmfAccount(account.id);
					if (mmf) goto(`/businesses/${mmf.business_id}?tab=accounts`, { replaceState: true });
					else error = 'Associated mmf account not found.';
					break;
				case AccountType.Utility:
					const utility = await getUtilityAccount(account.id);
					if (utility)
						goto(`/businesses/${utility.business_id}?tab=accounts`, { replaceState: true });
					else error = 'Associated utility account not found.';
					break;
				case AccountType.System:
					goto(`/accounts/system`, { replaceState: true });
					break;
				default:
					error = 'Unknown account type encountered.';
					break;
			}
		} catch (e: any) {
			console.error('Failed to resolve account:', e);
			error = `An unexpected error occurred: ${e.message}`;
		} finally {
			// This part might not be reached if a goto happens, which is fine.
			isLoading = false;
		}
	});
</script>

<main class="container mx-auto space-y-6 p-6">
	{#if isLoading}
		<div class="flex h-64 flex-col items-center justify-center gap-4">
			<LoaderCircle class="h-8 w-8 animate-spin text-primary" />
			<p class="text-muted-foreground">Resolving account...</p>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardHeader>
				<CardTitle>Error</CardTitle>
			</CardHeader>
			<CardContent>
				<p>{error}</p>
			</CardContent>
		</Card>
	{/if}
	<!-- This page will almost always redirect, so no other UI is needed. -->
</main>
