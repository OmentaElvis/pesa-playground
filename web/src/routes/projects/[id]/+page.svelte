<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardDescription
	} from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import LogSheet from '$lib/components/LogSheet.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import {
		Copy,
		Key,
		Activity,
		Plus,
		Settings,
		CheckCircle,
		XCircle,
		AlertCircle,
		RefreshCw,
		LoaderCircle,
		ChevronRight,
		Building,
		Users,
		Phone,
		CreditCard,
		ArrowLeftRight,
		ArrowRight
	} from 'lucide-svelte';
	import {
		getProject,
		type ProjectDetails,
		type ApiLog,
		getProjectApiLogs,
		getBusiness,
		getPaybillAccountsByBusinessId,
		type PaybillAccountDetails,
		getTillAccountsByBusinessId,
		type TillAccountDetails,
		getUsers,
		type UserDetails,
		createUser,
		generateUser,
		type BusinessDetails
	} from '$lib/api';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { listen, type UnlistenFn } from '$lib/api';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import HelpPopover from '$lib/components/ui/helppopover/HelpPopover.svelte';
	import TransactionList from '$lib/components/TransactionList.svelte';
	import SandboxToggle from '$lib/components/SandboxToggle.svelte';
	import DiceBearAvatar from '$lib/components/ui/avatar/DiceBearAvatar.svelte';
	import { debounce, formatAmount, getInitials } from '$lib/utils';

	let id = $derived(page.params.id);
	let project: ProjectDetails | null = $state(null);
	let apiLogs: ApiLog[] = $state([]);
	let business: BusinessDetails | null = $state(null);
	let users: UserDetails[] = $state([]);

	let selectedLog: ApiLog | null = $state(null);
	let logSidebarOpen: boolean = $state(false);
	let apiLogsLoading: boolean = $state(false);
	let paybills: PaybillAccountDetails[] = $state([]);
	let tills: TillAccountDetails[] = $state([]);

	// New user form
	let creatingUser = $state(false);
	let newUser = $state({
		name: '',
		phone: '',
		balance: 10000,
		pin: '0000'
	});

	async function addUser() {
		creatingUser = true;
		try {
			await createUser(newUser.name, newUser.phone, newUser.balance, newUser.pin);
			newUser = { name: '', phone: '', balance: 10000, pin: '0000' };
			users = await getUsers();
		} catch (err) {
			toast(`Failed to create user: ${err}`);
		} finally {
			creatingUser = false;
		}
	}

	async function generateRandomUser() {
		newUser = await generateUser();
	}

	function copyToClipboard(text: String = '') {
		navigator.clipboard.writeText(text as string);
		toast(`Copied to clipboard`);
	}

	function getStatusColor(status: number) {
		if (status >= 200 && status < 300) return 'text-green-600';
		if (status >= 400 && status < 500) return 'text-yellow-600';
		if (status >= 500) return 'text-red-600';
		return 'text-gray-600';
	}

	function getStatusIcon(status: number) {
		if (status >= 200 && status < 300) return CheckCircle;
		if (status >= 400) return XCircle;
		return AlertCircle;
	}

	async function refreshLogs() {
		apiLogsLoading = true;
		try {
			apiLogs = await getProjectApiLogs(Number(id), { limit: 20 });
		} finally {
			apiLogsLoading = false;
		}
	}

	async function loadBusinessAccounts() {
		if (business) {
			paybills = await getPaybillAccountsByBusinessId(business.id);
			tills = await getTillAccountsByBusinessId(business.id);
		}
	}

	const debouncedRefresh = debounce(() => {
		refreshLogs();
		loadBusinessAccounts();
	}, 10000);

	let unlisten: UnlistenFn;

	listen('new-api-log', (event) => {
		let project_id = event.payload;
		if (project_id == Number(id)) {
			debouncedRefresh();
		}
	}).then((un) => {
		unlisten = un;
	});

	onDestroy(() => {
		if (unlisten) unlisten();
	});

	let currentTab = $state('transactions');

	$effect(() => {
		const url = new URL(page.url);
		if (url.searchParams.get('tab') !== currentTab) {
			url.searchParams.set('tab', currentTab);
			goto(url, { replaceState: true, keepFocus: true, noScroll: true });
		}
	});

	onMount(async () => {
		const tab = page.url.searchParams.get('tab');
		if (tab) {
			currentTab = tab;
		}

		async function loadData(projectId: number) {
			project = await getProject(projectId);
			business = await getBusiness(project.business_id);
			users = await getUsers();
			await loadBusinessAccounts();
			await refreshLogs();
		}

		loadData(Number(id));
	});
</script>

<main class="container mx-auto space-y-6 p-6">
	{#if !project || !business}
		<div class="flex size-full items-center justify-center p-10">
			<LoaderCircle class="animate-spin" />
			<span class="ml-2">Loading project...</span>
		</div>
	{:else}
		<div class="flex flex-col gap-4">
			<div class="flex items-center justify-between">
				<div>
					<div class="flex items-center gap-2">
						<h1 class="text-3xl font-bold tracking-tight text-foreground">{project.name}</h1>
						<Badge variant="outline">#{project.id}</Badge>
					</div>
					<div class="mt-2 flex items-center gap-2 text-sm text-muted-foreground">
						<span>
							Business:
							<a
								href="/businesses/{project.business_id}"
								class="font-semibold text-primary hover:underline"
							>
								{business.name}
							</a>
						</span>
						<span class="text-muted-foreground">•</span>
						<span>Shortcode: {business?.short_code}</span>
					</div>
				</div>
				<div class="flex gap-2">
					<Button href={`/projects/${id}/settings`} variant="outline" class="gap-2">
						<Settings class="h-4 w-4" />
						Settings
					</Button>
				</div>
			</div>
			<SandboxToggle id={Number(id)} />
		</div>

		<!-- Key Information -->
		<div>
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<Key class="h-5 w-5" />
						API Credentials
						<HelpPopover slug="daraja-auth" />
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4 font-mono">
					<div class="space-y-1">
						<Label class="text-sm">Consumer Key</Label>
						<div class="flex items-center gap-2">
							<Input type="text" value={project.consumer_key} readonly class="flex-1" />
							<Button
								size="icon"
								variant="outline"
								onclick={() => copyToClipboard(project?.consumer_key)}
							>
								<Copy class="h-4 w-4" />
							</Button>
						</div>
					</div>
					<div class="space-y-1">
						<Label class="text-sm">Consumer Secret</Label>
						<div class="flex items-center gap-2">
							<Input value={project.consumer_secret} readonly class="flex-1" />
							<Button
								size="icon"
								variant="outline"
								onclick={() => copyToClipboard(project?.consumer_secret)}
							>
								<Copy class="h-4 w-4" />
							</Button>
						</div>
					</div>
					<div class="space-y-1">
						<Label class="text-sm">Passkey</Label>
						<div class="flex items-center gap-2">
							<Input value={project.passkey} readonly class="flex-1" />
							<Button
								size="icon"
								variant="outline"
								onclick={() => copyToClipboard(project?.passkey)}
							>
								<Copy class="h-4 w-4" />
							</Button>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>

		<!-- Main Content Tabs -->
		<Tabs.Root bind:value={currentTab} class="w-full">
			<Tabs.List class="grid w-full grid-cols-4">
				<Tabs.Trigger value="transactions">Transactions</Tabs.Trigger>
				<Tabs.Trigger value="accounts">Business Accounts</Tabs.Trigger>
				<Tabs.Trigger value="api-logs">API Logs</Tabs.Trigger>
				<Tabs.Trigger value="users">Test Users</Tabs.Trigger>
			</Tabs.List>

			<!-- Transactions Tab -->
			<Tabs.Content value="transactions">
				<TransactionList
					scope={{
						type: 'Business',
						id: business.id
					}}
				/>
			</Tabs.Content>

			<!-- Business Accounts Tab -->
			<Tabs.Content value="accounts">
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<CardTitle class="flex items-center gap-2">
							<Building class="h-5 w-5" />
							Business Accounts ({business.name}) <Badge>#{business.short_code}</Badge>
						</CardTitle>
						<Button size="sm" variant="outline" href={`/businesses/${business.id}`}>
							Open
							<ArrowRight class="mr-2 h-4 w-4" />
						</Button>
					</CardHeader>
					<CardContent class="overflow-x-auto">
						<div class="grid grid-cols-3 gap-4 max-md:grid-cols-2 max-sm:grid-cols-1">
							<Card>
								<CardHeader>
									<CardTitle>Working account</CardTitle>
									<CardDescription>
										Funds parked in the working (MMF) account for liquidity and balance management.
									</CardDescription>
								</CardHeader>
								<CardContent>
									<p>{formatAmount(business.mmf_account.balance / 100)}</p>
								</CardContent>
							</Card>
							<Card>
								<CardHeader>
									<CardTitle>Utility account</CardTitle>
									<CardDescription>
										Operational balance used to process business payment transactions.
									</CardDescription>
								</CardHeader>
								<CardContent>
									<p>{formatAmount(business.utility_account.balance / 100)}</p>
								</CardContent>
							</Card>
							<Card>
								<CardHeader>
									<CardTitle>Business charges</CardTitle>
									<CardDescription>
										Cumulative charges applied to business transactions. Needs tobe reconciled.
									</CardDescription>
								</CardHeader>
								<CardContent>
									<p>{formatAmount(business.charges_amount || 0)}</p>
								</CardContent>
							</Card>
						</div>
					</CardContent>
				</Card>
				<Card class="mt-4">
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<div class="flex flex-1 items-center gap-2">
								<ArrowLeftRight class="h-5 w-5" />
								Business payment endpoints
							</div>
							<Button size="sm" variant="outline" href={`/businesses/${business.id}?tab=accounts`}>
								<Plus class="mr-2 h-4 w-4" />
								Add Account
							</Button>
						</CardTitle>
						<CardDescription>
							These are the externally exposed paybill and till numbers used by customers and
							systems to initiate payment transactions.
						</CardDescription>
					</CardHeader>
					<CardContent class="overflow-x-auto">
						<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
							{#each paybills as account}
								<div class="flex items-center justify-between rounded-md border p-3">
									<a
										class="cursor-pointer text-sm text-muted-foreground hover:underline"
										href="/businesses/{business.id}?collapsed=true&biz_tab=accounts&biz_action=edit_paybill&biz_edit_paybill={account.id}"
									>
										Paybill: {account.paybill_number}
									</a>
								</div>
							{:else}
								<p class="text-muted-foreground">No paybill accounts</p>
							{/each}
							{#each tills as account}
								<div class="flex items-center justify-between rounded-md border p-3">
									<div>
										<div class="font-semibold">{account.location_description || 'Till'}</div>
										<a
											class="cursor-pointer text-sm text-muted-foreground hover:underline"
											href="/businesses/{business.id}?collapsed=true&biz_tab=accounts&biz_action=edit_paybill&biz_edit_till={account.id}"
										>
											Till: {account.till_number}
										</a>
									</div>
								</div>
							{:else}
								<p class="text-muted-foreground">No till accounts</p>
							{/each}
						</div>
					</CardContent>
				</Card>
			</Tabs.Content>

			<!-- API Logs Tab -->
			<Tabs.Content value="api-logs">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle class="flex items-center gap-2">
								<Activity class="h-5 w-5" />
								Recent API Activity
							</CardTitle>
							<Button size="sm" variant="outline" onclick={refreshLogs} disabled={apiLogsLoading}>
								{#if apiLogsLoading}
									<LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
								{:else}
									<RefreshCw class="mr-2 h-4 w-4" />
								{/if}
								Refresh
							</Button>
						</div>
					</CardHeader>
					<CardContent>
						<div class="space-y-2">
							{#each apiLogs as log (log.id)}
								{@const StatusIcon = getStatusIcon(log.status_code)}
								<button
									class="w-full space-y-2 rounded-lg border p-3 text-left hover:bg-muted/50"
									onclick={() => {
										selectedLog = log;
										logSidebarOpen = true;
									}}
								>
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-2">
											<StatusIcon class="h-4 w-4 {getStatusColor(log.status_code)}" />
											<span class="font-mono text-sm font-medium">{log.method}</span>
											<span class="font-mono text-sm">{log.path}</span>
											<Badge variant="outline" class={getStatusColor(log.status_code)}>
												{log.status_code}
											</Badge>
										</div>
										<ChevronRight class="h-4 w-4 text-muted-foreground" />
									</div>
									<div class="text-xs text-muted-foreground">
										{new Date(log.created_at).toLocaleString()}
									</div>
								</button>
							{:else}
								<div class="py-8 text-center text-muted-foreground">No API logs yet.</div>
							{/each}
						</div>
					</CardContent>
				</Card>
			</Tabs.Content>

			<!-- Test Users Tab -->
			<Tabs.Content value="users" class="space-y-6">
				<!-- Add New User Card -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Plus class="h-5 w-5" />
							Add Test User
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="grid gap-4 md:grid-cols-4">
							<div>
								<Label for="user-name" class="text-sm font-medium">Name</Label>
								<Input id="user-name" bind:value={newUser.name} placeholder="John Doe" />
							</div>
							<div>
								<Label for="user-phone" class="text-sm font-medium">Phone Number</Label>
								<Input id="user-phone" bind:value={newUser.phone} placeholder="254712345678" />
							</div>
							<div>
								<Label for="user-balance" class="text-sm font-medium">Initial Balance (KES)</Label>
								<Input id="user-balance" type="number" bind:value={newUser.balance} min="0" />
							</div>
							<div>
								<Label for="pin" class="text-sm font-medium">Pin</Label>
								<Input id="pin" bind:value={newUser.pin} />
							</div>
							<div class="flex items-end gap-2">
								{#if !creatingUser}
									<Button onclick={addUser} disabled={!newUser.name || !newUser.phone}>
										Add User
									</Button>
								{:else}
									<Button disabled>
										<LoaderCircle class="animate-spin" />
										Add User
									</Button>
								{/if}
								<Button variant="outline" onclick={generateRandomUser}>
									<RefreshCw class="h-4 w-4" />
								</Button>
							</div>
						</div>
					</CardContent>
				</Card>

				<!-- Users List -->
				<Card>
					<CardHeader>
						<CardTitle class="flex items-center gap-2">
							<Users class="h-5 w-5" />
							Test Users
						</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="space-y-4">
							{#each users as user (user.id)}
								<div class="rounded-lg border p-4">
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-4">
											<div
												class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10"
											>
												<DiceBearAvatar
													seed={`${user.id}-${user.name}`}
													fallback={getInitials(user.name)}
												/>
											</div>
											<div>
												<h4 class="font-medium">
													<a href="/users/{user.id}" class="hover:underline">{user.name}</a>
												</h4>
												<div class="flex items-center gap-4 text-sm text-muted-foreground">
													<span class="flex items-center gap-1">
														<Phone class="h-3 w-3" />
														{user.phone}
														<Button
															size="icon"
															variant="ghost"
															class="h-6 w-6"
															onclick={() => copyToClipboard(user.phone)}
														>
															<Copy class="h-3 w-3" />
														</Button>
													</span>
													<span class="flex items-center gap-1">
														<CreditCard class="h-3 w-3" />
														KES {user.balance.toLocaleString()}
													</span>
												</div>
											</div>
										</div>
									</div>
								</div>
							{:else}
								<div class="py-8 text-center text-muted-foreground">
									<Users class="h-12 w-12 mx-auto mb-4 opacity-50" />
									<p>No test users yet. Add your first user above.</p>
								</div>
							{/each}
						</div>
					</CardContent>
				</Card>
			</Tabs.Content>
		</Tabs.Root>
	{/if}

	{#if selectedLog}
		<LogSheet log={selectedLog} bind:open={logSidebarOpen} />
	{/if}
</main>
