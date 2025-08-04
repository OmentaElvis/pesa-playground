<script lang="ts">
  import * as Drawer from "$lib/components/ui/drawer";
  import { Button } from "../ui/button";
  import { Input } from "$lib/components/ui/input/index.js";
  import type { User } from "$lib/api";
  import { X, ArrowLeft } from "lucide-svelte";

  export let open = false;
  export let user: User;

  interface SimMenu {
    title: string;
    options: { id: string; label: string; action: () => void }[];
    message?: string;
    isInfo?: boolean;
    isSuccess?: boolean;
  }

  let simMenuStack: string[] = [];
  let currentSimMenu = "main";
  let simFormData: Record<string, any> = {};
  let simInputMode: string | null = null;
  let simInputValue = "";
  let simInputPrompt = "";
  let simInputCallback: ((arg0: string) => void) | null = null;

  type SubmitType = "SendMoney" | "BuyAirtime" | "Paybill" | "BuyGoodsTill";

  function navigateSimMenu(menuId: string, data = {}) {
    if (currentSimMenu !== menuId) {
      simMenuStack.push(currentSimMenu);
    }
    currentSimMenu = menuId;
    simFormData = { ...simFormData, ...data };
  }

  function goBackSimMenu() {
    if (simMenuStack.length > 0) {
      currentSimMenu = simMenuStack.pop() || "";
    } else {
      open = false;
    }
  }

  function showSimInput(
    prompt: string,
    callback: ((arg0: string) => void) | null,
    type = "text",
  ) {
    simInputMode = type;
    simInputPrompt = prompt;
    simInputCallback = callback;
    simInputValue = "";
  }

  function handleSimInput() {
    if (simInputCallback) {
      simInputCallback(simInputValue);
    }
  }

  function cancelSimInput() {
    simInputMode = null;
    simInputValue = "";
  }

  function submit(submit_type: SubmitType) {
    switch (submit_type) {
      case "SendMoney":
        break;
      case "BuyAirtime":
        break;
    }
  }

  function sendMoney() {
    showSimInput("Enter phone number:", (value) => {
      simFormData.phone = value;
      showSimInput(
        "Enter amount:",
        (amount) => {
          simFormData.amount = amount;
          showSimInput(
            "Enter M-PESA PIN:",
            (pin) => {
              simFormData.pin = pin;
              submit("SendMoney");
            },
            "password",
          );
        },
        "number",
      );
    });
  }

  const simMenus: Record<string, SimMenu> = {
    main: {
      title: "M-PESA",
      options: [
        {
          id: "send_money",
          label: "1. Send Money",
          action: () => navigateSimMenu("send_money"),
        },
        {
          id: "withdraw",
          label: "2. Withdraw Cash",
          action: () => navigateSimMenu("withdraw"),
        },
        {
          id: "buy_airtime",
          label: "3. Buy Airtime",
          action: () => navigateSimMenu("buy_airtime"),
        },
        {
          id: "lipa_na_mpesa",
          label: "4. Lipa na M-PESA",
          action: () => navigateSimMenu("lipa_na_mpesa"),
        },
        {
          id: "my_account",
          label: "5. My Account",
          action: () => navigateSimMenu("my_account"),
        },
      ],
    },
    send_money: {
      title: "Send Money",
      options: [
        {
          id: "enter_phone",
          label: "1. Enter Phone No.",
          action: () => sendMoney(),
        },
        {
          id: "from_phonebook",
          label: "2. From Phonebook",
          action: () => navigateSimMenu("phonebook"),
        },
      ],
    },
    buy_airtime: {
      title: "Buy Airtime",
      options: [
        {
          id: "my_phone",
          label: "1. My Phone",
          action: () =>
            showSimInput(
              "Enter amount:",
              (amount) => {
                simFormData.amount = amount;
                simFormData.phone = user.phone;
                showSimInput(
                  "Enter M-PESA PIN:",
                  (pin) => {
                    simFormData.pin = pin;
                    submit("BuyAirtime");
                  },
                  "password",
                );
              },
              "number",
            ),
        },
        {
          id: "another_phone",
          label: "2. Another Phone",
          action: () =>
            showSimInput("Enter phone number:", (phone) => {
              simFormData.phone = phone;
              showSimInput(
                "Enter amount:",
                (amount) => {
                  simFormData.amount = amount;
                  showSimInput(
                    "Enter M-PESA PIN:",
                    (pin) => {
                      simFormData.pin = pin;
                      submit("BuyAirtime");
                    },
                    "password",
                  );
                },
                "number",
              );
            }),
        },
      ],
    },
    lipa_na_mpesa: {
      title: "Lipa na M-PESA",
      options: [
        {
          id: "paybill",
          label: "1. Pay Bill",
          action: () =>
            showSimInput("Enter business number:", (business) => {
              simFormData.business = business;
              showSimInput("Enter account number:", (account) => {
                simFormData.account = account;
                showSimInput(
                  "Enter amount:",
                  (amount) => {
                    simFormData.amount = amount;
                    showSimInput(
                      "Enter M-PESA PIN:",
                      (pin) => {
                        simFormData.pin = pin;
                        submit("Paybill");
                      },
                      "password",
                    );
                  },
                  "number",
                );
              });
            }),
        },
        {
          id: "buy_goods",
          label: "2. Buy Goods and Services",
          action: () =>
            showSimInput("Enter till number:", (till) => {
              simFormData.business = `Till ${till}`;
              showSimInput(
                "Enter amount:",
                (amount) => {
                  simFormData.amount = amount;
                  showSimInput(
                    "Enter M-PESA PIN:",
                    (pin) => {
                      simFormData.pin = pin;
                      submit("BuyGoodsTill");
                    },
                    "password",
                  );
                },
                "number",
              );
            }),
        },
      ],
    },
    my_account: {
      title: "My Account",
      options: [
        {
          id: "mini_statement",
          label: "1. Mini Statement",
          action: () => navigateSimMenu("mini_statement"),
        },
        {
          id: "account_balance",
          label: "2. Account Balance",
          action: () => navigateSimMenu("account_balance"),
        },
      ],
    },
    mini_statement: {
      title: "Mini statement",
      message: "Not Implemented yet",
      isInfo: true,
      options: [],
    },
    account_balance: {
      title: "Account Balance",
      message: `Your Account balance is Ksh <b class="text-green-500">${user.balance.toFixed(2) || "0.00"}</b>`,
      isInfo: true,
      options: [],
    },
    success: {
      title: "Transaction Successful",
      message: "Your transaction has been processed successfully.",
      isSuccess: true,
      options: [],
    },
  };
</script>

<Drawer.Root bind:open direction="right">
  <Drawer.Content class="mt-[36px]">
    <div class="">
      <div class="text-sm rounded-lg w-full">
        <div class="p-3 flex flex-col">
          {#if simInputMode}
            <div class="mt-8 flex-1 flex flex-col">
              <div class="mb-4">
                <div class="mb-2">STK Menu</div>
                <div class="text-xs">{simInputPrompt}</div>
              </div>

              <div class="flex-1 flex flex-col justify-center">
                <div class="p-2 mb-4">
                  <Input
                    type={simInputMode === "password"
                      ? "password"
                      : simInputMode === "number"
                        ? "number"
                        : "text"}
                    bind:value={simInputValue}
                    class="w-full bg-transparent outline-none"
                    placeholder={simInputMode === "password"
                      ? "****"
                      : "Enter..."}
                  />
                </div>

                <div class="flex gap-2 text-xs">
                  <Button onclick={handleSimInput} disabled={!simInputValue}>
                    OK
                  </Button>
                  <Button onclick={cancelSimInput}>Cancel</Button>
                </div>
              </div>
            </div>
          {:else}
            <div class="flex py-8 justify-between items-center w-full">
              <div class="mb-1">STK Menu</div>
              <div class="">
                <Button variant="ghost" onclick={goBackSimMenu}>
                  <ArrowLeft />
                </Button>
                <Button variant="ghost" onclick={() => (open = false)}>
                  <X />
                </Button>
              </div>
            </div>

            <div class="flex-1">
              {#if simMenus[currentSimMenu]?.options}
                <div class="space-y-1">
                  {#each simMenus[currentSimMenu].options as option}
                    <button
                      class="w-full text-left p-2 text-xs cursor-pointer hover:bg-muted transition-colors radius"
                      on:click={option.action}
                    >
                      {option.label}
                    </button>
                  {/each}
                </div>
              {/if}
              {#if simMenus[currentSimMenu]?.isInfo}
                <div class="p-4">
                  {@html simMenus[currentSimMenu].message}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </Drawer.Content>
</Drawer.Root>
