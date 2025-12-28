<script lang="ts">
	import { Phone, Wallet, Settings, User as UserIcon, Plus, LoaderCircle } from 'lucide-svelte';
	import { formatAmount, getInitials } from '$lib/utils';
	import { getUsers, type User, generateUsers, createUser } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';
	import DiceBearAvatar from '$lib/components/ui/avatar/DiceBearAvatar.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { toast } from 'svelte-sonner';
	import { listen, type UnlistenFn } from '$lib/api';
	import { transactionLogStore } from '$lib/stores/transactionLogStore';
	import { Button } from '../ui/button';

	let users: User[] = $state([]);
	let generating = $state(false);
	const unlistenFunctions: UnlistenFn[] = [];

	async function fetchUsers() {
		try {
			users = await getUsers();
		} catch (err) {
			toast(`Failed to fetch users: ${err}`);
		}
	}

	async function genUsers(count: number) {
		generating = true;
		try {
			let new_users = await generateUsers(count);
			for (let user of new_users) {
				await createUser(user.name, user.phone, user.balance, user.pin);
			}
		} catch (err) {
			toast(`Failed to create user: ${err}`);
		} finally {
			generating = false;
		}
	}

	onMount(async () => {
		await fetchUsers();

		const userCreatedUnlisten = await listen('user_created', (_) => {
			fetchUsers();
		});
		unlistenFunctions.push(userCreatedUnlisten);

		const newTransactionUnlisten = await listen('new_transaction', (_) => {
			fetchUsers();
		});
		unlistenFunctions.push(newTransactionUnlisten);
	});

	onDestroy(() => {
		unlistenFunctions.forEach((f) => f());
	});
</script>

<div class="flex h-full w-full flex-col bg-secondary">
	<div class="p-4">
		<div class="flex items-center justify-between">
			<div>
				<h2 class="flex items-center gap-2 text-lg font-semibold">
					<UserIcon size={20} />
					Sandbox Users
				</h2>
				<p class="mt-1 text-sm">{users.length} users available</p>
			</div>
			<Popover.Root>
				<Popover.Trigger>
					<Settings size={20} />
				</Popover.Trigger>
				<Popover.Content class="w-80">
					<div class="grid gap-4">
						<div class="">
							<h3 class="mb-2 font-medium">Settings</h3>
							<div class="space-y-2">
								<Button
									variant="outline"
									class="w-full justify-start gap-2"
									onclick={() => genUsers(5)}
									disabled={generating}
								>
									{#if generating}
										<LoaderCircle class="animate-spin" />
									{:else}
										<Plus size={20} />
									{/if}
									Generate 5 More Users
								</Button>
							</div>
						</div>
					</div>
				</Popover.Content>
			</Popover.Root>
		</div>
	</div>
	<ScrollArea class="min-h-0 flex-1">
		{#each users as user (user.account_id)}
			{@const hasUnread = $transactionLogStore.some(
				(log) => log.to_id === user.account_id || log.from_id === user.account_id
			)}
			<a
				class="block w-full border-b p-4 text-left duration-200 dark:border-none"
				href="/users/{user.account_id}"
			>
				<div class="flex items-center gap-3">
					<div class="relative h-16 w-16">
						{#if hasUnread}
							<div
								class="absolute top-0 right-0 z-10 h-4 w-4 rounded-full border-2 border-background bg-red-500"
							></div>
						{/if}
						<DiceBearAvatar seed={`${user.account_id}-${user.name}`} fallback={getInitials(user.name)} />
					</div>
					<div class="min-w-0 flex-1">
						<div class="flex items-center justify-between">
							<h3 class="truncate font-medium">{user.name}</h3>
						</div>
						<div class="mt-1 flex items-center gap-2">
							<Phone size={12} class="" />
							<span class="text-sm">+{user.phone}</span>
						</div>
						<div class="mt-1 flex items-center gap-2">
							<Wallet size={12} class="" />
							<span class="text-sm font-bold text-green-700 dark:text-green-600">
								{formatAmount(user.balance / 100)}
							</span>
						</div>
					</div>
				</div>
			</a>
		{/each}
	</ScrollArea>
</div>
