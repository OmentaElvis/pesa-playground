<script lang="ts">
	import { Phone, Wallet, Settings, User as UserIcon } from 'lucide-svelte';
	import { formatAmount, getInitials } from '$lib/utils';
	import { getUsers, type UserDetails } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';
	import DiceBearAvatar from '$lib/components/ui/avatar/DiceBearAvatar.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { toast } from 'svelte-sonner';
	import { listen, type UnlistenFn } from '$lib/api';
	import { transactionLogStore } from '$lib/stores/transactionLogStore';

	let users: UserDetails[] = $state([]);
	const unlistenFunctions: UnlistenFn[] = [];

	async function fetchUsers() {
		try {
			users = await getUsers();
		} catch (err) {
			toast(`Failed to fetch users: ${err}`);
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
							<h3 class="mb-2 font-medium text-gray-900">Settings</h3>
							<div class="space-y-2">
								<button
									class="w-full rounded-md px-3 py-2 text-left text-sm transition-colors duration-200"
									onclick={() => {}}
								>
									Generate 5 More Users
								</button>
							</div>
						</div>
					</div>
				</Popover.Content>
			</Popover.Root>
		</div>
	</div>
	<ScrollArea class="min-h-0 flex-1">
		{#each users as user}
			{@const hasUnread = $transactionLogStore.some(
				(log) => log.to_id === user.id || log.from_id === user.id
			)}
			<a
				class="block w-full border-b p-4 text-left duration-200 dark:border-none"
				href="/users/{user.id}"
			>
				<div class="flex items-center gap-3">
					<div class="relative h-16 w-16">
						{#if hasUnread}
							<div
								class="absolute top-0 right-0 z-10 h-4 w-4 rounded-full border-2 border-background bg-red-500"
							></div>
						{/if}
						<DiceBearAvatar seed={`${user.id}-${user.name}`} fallback={getInitials(user.name)} />
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
								{formatAmount(user.balance)}
							</span>
						</div>
					</div>
				</div>
			</a>
		{/each}
	</ScrollArea>
</div>
