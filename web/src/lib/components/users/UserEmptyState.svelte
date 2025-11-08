<script>
	import { User, Plus, LoaderCircle } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { generateUsers, createUser } from '$lib/api';
	import { toast } from 'svelte-sonner';
	let generating = $state(false);

	async function genUsers() {
		generating = true;
		try {
			let users = await generateUsers(10);
			for (let user of users) {
				let id = await createUser(user.name, user.phone, user.balance, user.pin);
			}
		} catch (err) {
			toast(`Failed to create user: ${err}`);
		} finally {
			generating = false;
		}
	}
</script>

<div class="flex flex-1 flex-col items-center justify-center p-8">
	<div class="max-w-md text-center">
		<div class="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-gray-600">
			<User size={32} class="text-white" />
		</div>
		<h1 class="mb-4 text-2xl font-bold">Sandbox Users</h1>
		<p class="mb-8 text-sm">
			Select a user from the sidebar to view their transaction history, or generate new users.
		</p>
		<Button
			variant="outline"
			class="mx-auto flex cursor-pointer items-center gap-2 rounded-lg px-6 py-3 font-medium"
			onclick={genUsers}
			disabled={generating}
		>
			{#if generating}
				<LoaderCircle class="animate-spin" />
			{:else}
				<Plus size={20} />
			{/if}
			Generate New Users
		</Button>
	</div>
</div>
