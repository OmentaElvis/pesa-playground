<script>
  import { Phone, Wallet, Smartphone } from "lucide-svelte";
  import { formatAmount, getInitials } from "$lib/utils";
  import TransactionList from "./TransactionList.svelte";
    import DiceBearAvatar from "../ui/avatar/DiceBearAvatar.svelte";

  export let selectedUser;
  export let transactions;

  let showSimToolkit = false;
</script>

<div class="flex-1 flex flex-col">
  <!-- Header -->
  <div class="bg-white border-b border-gray-200 p-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10">
          <DiceBearAvatar seed={`${selectedUser.id}-${selectedUser.name}`} fallback={getInitials(selectedUser.name)} />
        </div>
        <div>
          <h1 class="text-xl font-semibold text-gray-900">{selectedUser.name}</h1>
          <div class="flex items-center gap-4 text-sm text-gray-600">
            <span class="flex items-center gap-1">
              <Phone size={14} />
              {selectedUser.phone}
            </span>
            <span class="flex items-center gap-1">
              <Wallet size={14} />
              Balance: {formatAmount(selectedUser.balance)}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Transactions -->
  <TransactionList transactions={transactions[selectedUser.id]} />
</div>
