<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import * as Tabs from "$lib/components/ui/tabs/index.js";
	import { Separator } from "$lib/components/ui/separator";
	import { onMount } from "svelte";
	import { page } from "$app/state";
	import {
		getBusiness,
		updateBusiness,
		deleteBusiness,
		getPaybillAccountsByBusinessId,
		getTillAccountsByBusinessId,
		getProjectsByBusinessId,
		type FullTransactionLog,
		listAccountsFullTransactionLogs,
		type PaybillAccountDetails,
		type TillAccountDetails,
		type ProjectSummary,
		type BusinessDetails,
		formatTransactionAmount,
	} from "$lib/api";
	import { goto } from "$app/navigation";
	import {
		ArrowRightLeft,
		ChevronsLeftRightEllipsis,
		DollarSign,
		LoaderCircle,
		MoveDownLeft,
		MoveUpRight,
		Pencil,
		Save,
		Trash,
		WalletMinimal,
	} from "lucide-svelte";
	import PaybillAccounts from "$lib/components/businesses/PaybillAccounts.svelte";
	import TillAccounts from "$lib/components/businesses/TillAccounts.svelte";
	import Projects from "$lib/components/businesses/Projects.svelte";
	import { Label } from "$lib/components/ui/label";
	import { Input } from "$lib/components/ui/input";
	import * as Table from "$lib/components/ui/table/index.js";
	import { formatDate } from "$lib/utils";

	let business: BusinessDetails | null = $state(null);
	let paybillAccounts: PaybillAccountDetails[] = $state([]);
	let tillAccounts: TillAccountDetails[] = $state([]);
	let projects: ProjectSummary[] = $state([]);

	interface Transaction extends FullTransactionLog {
		account_type: "Till" | "Paybill";
	}
	let transactions: Transaction[] = $state([]);

	let businessId: number | undefined = $state(undefined);

	$effect(() => {
		if (page.params.id) {
			businessId = parseInt(page.params.id);
		}
	});

	async function loadBusinessDetails() {
		if (businessId) {
			business = await getBusiness(businessId);
			paybillAccounts = await getPaybillAccountsByBusinessId(businessId);
			tillAccounts = await getTillAccountsByBusinessId(businessId);
			projects = await getProjectsByBusinessId(businessId);
		}
	}

	async function loadTransactions() {
		if (businessId) {
			let paybillTransactions: Transaction[] = (
				await listAccountsFullTransactionLogs(
					paybillAccounts.map((acc) => acc.account_id),
				)
			).map((txn) => {
				return {
					account_type: "Paybill",
					...txn,
				};
			});

			let tillTransactions: Transaction[] = (
				await listAccountsFullTransactionLogs(
					tillAccounts.map((acc) => acc.account_id),
				)
			).map((txn) => {
				return {
					account_type: "Till",
					...txn,
				};
			});
			transactions = paybillTransactions.concat(tillTransactions);
		}
	}

	async function handleUpdateBusiness() {
		if (business) {
			await updateBusiness(business.id, {
				name: business.name,
			});
			await loadBusinessDetails();
		}
	}

	async function handleDeleteBusiness() {
		if (business && confirm("Are you sure you want to delete this business?")) {
			await deleteBusiness(business.id);
			goto("/businesses");
		}
	}

	onMount(async () => {
		await loadBusinessDetails();
		await loadTransactions();
	});

	let filterText = $state("");
	let sortKey: keyof FullTransactionLog | null = $state("transaction_date");
	let sortOrder: "asc" | "desc" = $state("desc");

	let processedTransactions: Transaction[] = $state([]);

	$effect(() => {
		let filtered = transactions;
		if (filterText) {
			filtered = transactions.filter((t) => {
				return (
					t.transaction_id.toLowerCase().includes(filterText.toLowerCase()) ||
					t.from_name.toLowerCase().includes(filterText.toLowerCase()) ||
					t.to_name.toLowerCase().includes(filterText.toLowerCase())
				);
			});
		}

		if (!sortKey) {
			processedTransactions = filtered;
			return;
		}

		processedTransactions = [...filtered].sort((a, b) => {
			if (!sortKey) return 0;
			const aValue = a[sortKey];
			const bValue = b[sortKey];

			if (aValue === null || aValue === undefined) return 1;
			if (bValue === null || bValue === undefined) return -1;

			if (sortKey === "transaction_date") {
				const dateA = new Date(aValue as string).getTime();
				const dateB = new Date(bValue as string).getTime();
				return sortOrder === "asc" ? dateA - dateB : dateB - dateA;
			}

			if (typeof aValue === "number" && typeof bValue === "number") {
				return sortOrder === "asc" ? aValue - bValue : bValue - aValue;
			}

			if (typeof aValue === "string" && typeof bValue === "string") {
				return sortOrder === "asc"
					? aValue.localeCompare(bValue)
					: bValue.localeCompare(aValue);
			}

			return 0;
		});
	});

	function setSortKey(key: keyof FullTransactionLog) {
		if (sortKey === key) {
			sortOrder = sortOrder === "asc" ? "desc" : "asc";
		} else {
			sortKey = key;
			sortOrder = "desc";
		}
	}

	// URL state management
	let currentTab = $state("accounts");

	$effect(() => {
		const url = new URL(page.url);
		if (url.searchParams.get("biz_tab") !== currentTab) {
			url.searchParams.set("biz_tab", currentTab);
			goto(url, { replaceState: true, keepFocus: true, noScroll: true });
		}
	});

	function clearActionParams() {
		const url = new URL(page.url);
		url.searchParams.delete("biz_action");
		url.searchParams.delete("biz_edit_paybill");
		url.searchParams.delete("biz_edit_till");
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	function setAction(action: string, id?: number) {
		const url = new URL(page.url);
		url.searchParams.set("biz_action", action);
		if (id) {
			if (action === "edit_paybill") {
				url.searchParams.set("biz_edit_paybill", id.toString());
			} else if (action === "edit_till") {
				url.searchParams.set("biz_edit_till", id.toString());
			}
		}
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	onMount(() => {
		const tab = page.url.searchParams.get("biz_tab");
		if (tab && tab !== currentTab) {
			currentTab = tab;
		}
	});
</script>

<div class="space-y-6 p-6">
	{#if business && businessId}
		<div>
			<div>
				<h3 class="text-lg font-medium">{business.name}</h3>
				<p class="text-sm text-muted-foreground">
					Manage business information.
				</p>
			</div>
			<Dialog.Root>
				<Dialog.Trigger>
					<Button><Pencil /> edit</Button>
				</Dialog.Trigger>
				<Dialog.Content>
					<Dialog.Header>
						<Dialog.Title>Edit Business</Dialog.Title>
						<Dialog.Description
							>Update details of this business</Dialog.Description
						>
					</Dialog.Header>
					<div class="grid gap-2">
						<Label for="name">Business Name</Label>
						<Input id="name" type="text" bind:value={business.name} />
					</div>
					<div class="grid gap-2 mt-2">
						<Label for="shortCode">Short Code</Label>
						<Input
							id="shortCode"
							type="text"
							bind:value={business.short_code}
						/>
					</div>
					<div>
						<Button onclick={handleUpdateBusiness} class="mt-4"
							><Save /> Update Business</Button
						>
					</div>
				</Dialog.Content>
			</Dialog.Root>
			<Button
				onclick={handleDeleteBusiness}
				class="mt-4 ml-2"
				variant="destructive"><Trash /> Delete Business</Button
			>
		</div>
		<Separator />
		<Tabs.Root bind:value={currentTab} class="">
			<Tabs.List>
				<Tabs.Trigger value="accounts"><WalletMinimal /> Accounts</Tabs.Trigger>
				<Tabs.Trigger value="projects"
					><ChevronsLeftRightEllipsis /> Projects</Tabs.Trigger
				>
				<Tabs.Trigger value="transactions"
					><DollarSign /> Transactions</Tabs.Trigger
				>
			</Tabs.List>
			<Tabs.Content value="accounts">
				<h3 class="text-lg font-medium mt-6">Associated Accounts</h3>
				<div class="grid gap-4 md:grid-cols-2">
					<PaybillAccounts
						{paybillAccounts}
						{businessId}
						refresh={() => {
							loadBusinessDetails();
							clearActionParams();
						}}
						create={() => setAction("new_paybill")}
						edit={(e) => setAction("edit_paybill", e.id)}
						isCreating={page.url.searchParams.get("biz_action") ===
							"new_paybill"}
						editingId={page.url.searchParams.get("biz_edit_paybill")}
						cancel={clearActionParams}
					/>
					<TillAccounts
						{tillAccounts}
						{businessId}
						refresh={() => {
							loadBusinessDetails();
							clearActionParams();
						}}
						create={() => setAction("new_till")}
						edit={(e) => setAction("edit_till", e.id)}
						isCreating={page.url.searchParams.get("biz_action") === "new_till"}
						editingId={page.url.searchParams.get("biz_edit_till")}
						cancel={clearActionParams}
					/>
				</div>
			</Tabs.Content>
			<Tabs.Content value="projects">
				<h3 class="text-lg font-medium mt-6">Associated Projects</h3>
				<Projects {projects} {businessId} on:refresh={loadBusinessDetails} />
			</Tabs.Content>
			<Tabs.Content value="transactions">
				<h3 class="text-lg font-medium mt-6">Transactions</h3>
				<div class="flex items-center py-4">
					<Input
						type="text"
						placeholder="Filter by transaction id, from, or to..."
						bind:value={filterText}
						class="max-w-sm"
					/>
				</div>
				<div class="overflow-x-auto">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head
									><ArrowRightLeft class="text-foreground/50" /></Table.Head
								>
								<Table.Head
									><button onclick={() => setSortKey("transaction_amount")}
										>Amount</button
									></Table.Head
								>
								<Table.Head class="font-bold">Txn Id</Table.Head>
								<Table.Head class="font-bold"
									><button onclick={() => setSortKey("from_name")}>From</button
									></Table.Head
								>
								<Table.Head class="font-bold"
									><button onclick={() => setSortKey("to_name")}>To</button
									></Table.Head
								>
								<Table.Head class="font-bold"
									><button onclick={() => setSortKey("transaction_date")}
										>Date</button
									></Table.Head
								>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each processedTransactions as transaction}
								<Table.Row>
									<Table.Cell>
										{#if transaction.direction == "Credit"}
											<MoveDownLeft class="text-green-700" />
										{:else}
											<MoveUpRight class="text-red-500" />
										{/if}
									</Table.Cell>
									<Table.Cell>
										<b>
											{formatTransactionAmount(transaction.transaction_amount)}
										</b>
									</Table.Cell>
									<Table.Cell
										><pre>{transaction.transaction_id}</pre></Table.Cell
									>
									<Table.Cell>{transaction.from_name}</Table.Cell>
									<Table.Cell>{transaction.to_name}</Table.Cell>
									<Table.Cell
										>{formatDate(transaction.transaction_date)}</Table.Cell
									>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>
			</Tabs.Content>
		</Tabs.Root>
	{:else}
		<p><LoaderCircle class="animate-spin" /> Loading business details...</p>
	{/if}
</div>
