<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import TransactionCostsTab from "$lib/components/settings/TransactionCostsTab.svelte";
</script>

<div class="p-4">
    <h1 class="text-2xl font-bold mb-4">Settings</h1>

    <Tabs.Root value="transaction-costs" class="w-full">
        <Tabs.List>
            <Tabs.Trigger value="transaction-costs">Transaction Costs</Tabs.Trigger>
            <!-- Add other settings tabs here -->
        </Tabs.List>
        <Tabs.Content value="transaction-costs">
            <TransactionCostsTab />
        </Tabs.Content>
    </Tabs.Root>
</div>
