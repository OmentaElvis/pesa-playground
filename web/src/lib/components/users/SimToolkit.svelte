<script lang="ts">
  import * as Drawer from "$lib/components/ui/drawer";
  import { Button } from "../ui/button";
  import { Input } from "$lib/components/ui/input/index.js";
  import type {
    UserDetails,
    PaybillAccountDetails,
    TillAccountDetails,
  } from "$lib/api";
  import { X, ArrowLeft, LoaderCircle } from "lucide-svelte";
  import { getUsers, getPaybillAccounts, getTillAccounts, transfer, getUserByPhone, TransactionType } from "$lib/api";
    import { toast } from "svelte-sonner";

  export let open = false;
  export let user: UserDetails;

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
  let suggestions: (UserDetails | PaybillAccountDetails | TillAccountDetails)[] =
    [];
  let suggestionType: "phone" | "paybill" | "till" | null = null;
  let suggestionsLoading = false;
  let submitting = false;

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

  async function showSimInput(
    prompt: string,
    callback: ((arg0: string) => void) | null,
    type = "text",
    suggType: "phone" | "paybill" | "till" | null = null,
  ) {
    simInputMode = type;
    simInputPrompt = prompt;
    simInputCallback = callback;
    simInputValue = "";
    suggestionType = suggType;

    if (suggestionType) {
      suggestionsLoading = true;
      if (suggestionType === "phone") {
        suggestions = (await getUsers()).filter((u) => {
          return u.id != user.id;
        });
      } else if (suggestionType === "paybill") {
        suggestions = await getPaybillAccounts();
      } else if (suggestionType === "till") {
        suggestions = await getTillAccounts();
      }
      suggestionsLoading = false;
    }
  }

  function handleSimInput() {
    if (simInputCallback) {
      simInputCallback(simInputValue);
    }
    suggestionType = null;
    suggestions = [];
  }

  function cancelSimInput() {
    simInputMode = null;
    simInputValue = "";
    suggestionType = null;
    suggestions = [];
  }

  async function transferMoneyToUser() {
   try {
      let phone = simFormData.phone;
      let receiver = await getUserByPhone(phone);
      if (!receiver) {
        toast.error(`Account with phone: ${phone} Not found.`);
        return;
      }
      let amount = Number(simFormData.amount) * 100;
      let txn = await transfer(user.id, receiver.id, amount, TransactionType.SendMoney);
      toast.info(`${txn.id}. Sent money to ${phone}. `);
      currentSimMenu = "main";
      open = false;
    } catch(err) {
      toast.error(`${err}`);
    }
  }

  async function submit(submit_type: SubmitType) {
    submitting = true;
    switch (submit_type) {
      case "SendMoney":
        if (simFormData.pin == "0000" || user.pin == simFormData.pin) {
          await transferMoneyToUser();
        } else {
          toast.error(`Incorrect pin for: ${user.name}, use the suggested pin or type 0000 to override checks.`);
        }
        break;
      case "Paybill":
        break;
      case "BuyGoodsTill":
      
        break;
      case "BuyAirtime":
        break;
    }
    submitting = false;
  }

  function sendMoney() {
    showSimInput(
      "Enter phone number:",
      (value) => {
        simFormData.phone = value;
        showSimInput(
          "Enter amount:",
          (amount) => {
            simFormData.amount = amount;
            showSimInput(
              "Enter M-PESA PIN:",
              async (pin) => {
                simFormData.pin = pin;
                await submit("SendMoney");
              },
              "password",
            );
          },
          "number",
        );
      },
      "text",
      "phone",
    );
  }

  async function selectFromPhonebook() {
    const users = (await getUsers()).filter((u) => {
      return u.id != user.id;
    });
    
    simMenus.phonebook = {
      title: "Phonebook",
      options: users.map((user, i) => ({
        id: `user_${user.id}`,
        label: `${i + 1}. ${user.name}`,
        action: () => {
          simFormData.phone = user.phone;
          showSimInput(
            "Enter amount:",
            (amount) => {
              simFormData.amount = amount;
              showSimInput(
                "Enter M-PESA PIN:",
                async (pin) => {
                  simFormData.pin = pin;
                  await submit("SendMoney");
                },
                "password",
              );
            },
            "number",
          );
        },
      })),
    };
    navigateSimMenu("phonebook");
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
          action: selectFromPhonebook,
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
                  async (pin) => {
                    simFormData.pin = pin;
                    await submit("BuyAirtime");
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
            showSimInput(
              "Enter phone number:",
              (phone) => {
                simFormData.phone = phone;
                showSimInput(
                  "Enter amount:",
                  (amount) => {
                    simFormData.amount = amount;
                    showSimInput(
                      "Enter M-PESA PIN:",
                      async (pin) => {
                        simFormData.pin = pin;
                        await submit("BuyAirtime");
                      },
                      "password",
                    );
                  },
                  "number",
                );
              },
              "text",
              "phone",
            ),
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
            showSimInput(
              "Enter business number:",
              (business) => {
                simFormData.business = business;
                showSimInput("Enter account number:", (account) => {
                  simFormData.account = account;
                  showSimInput(
                    "Enter amount:",
                    (amount) => {
                      simFormData.amount = amount;
                      showSimInput(
                        "Enter M-PESA PIN:",
                        async (pin) => {
                          simFormData.pin = pin;
                          await submit("Paybill");
                        },
                        "password",
                      );
                    },
                    "number",
                  );
                });
              },
              "text",
              "paybill",
            ),
        },
        {
          id: "buy_goods",
          label: "2. Buy Goods and Services",
          action: () =>
            showSimInput(
              "Enter till number:",
              (till) => {
                simFormData.business = `Till ${till}`;
                showSimInput(
                  "Enter amount:",
                  (amount) => {
                    simFormData.amount = amount;
                    showSimInput(
                      "Enter M-PESA PIN:",
                      async (pin) => {
                        simFormData.pin = pin;
                        await submit("BuyGoodsTill");
                      },
                      "password",
                    );
                  },
                  "number",
                );
              },
              "text",
              "till",
            ),
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
      message: `Your Account balance is Ksh <b class="text-green-500">${
        user.balance.toFixed(2) || "0.00"
      }</b>`,
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

  $: filteredSuggestions = suggestions.filter((suggestion) => {
    if (!simInputValue) return true;
    if (suggestionType === "phone") {
      const u = suggestion as UserDetails;
      return (
        u.phone.includes(simInputValue) ||
        u.name.toLowerCase().includes(simInputValue.toLowerCase())
      );
    } else if (suggestionType === "paybill") {
      const p = suggestion as PaybillAccountDetails;
      return p.paybill_number.toString().includes(simInputValue);
    } else if (suggestionType === "till") {
      const t = suggestion as TillAccountDetails;
      return t.till_number.toString().includes(simInputValue);
    }
    return true;
  });
</script>

<Drawer.Root bind:open direction="right">
  <Drawer.Content class="mt-[36px]">
    <div class="h-full">
      <div class="text-sm rounded-lg w-full h-full">
        <div class="p-3 flex flex-col h-full">
          {#if simInputMode}
            <div class="mt-8 flex-1 flex flex-col">
              <div class="mb-4">
                <div class="mb-2">STK Menu</div>
                <div class="text-xs">{simInputPrompt}</div>
              </div>

              <div class="flex-1 flex flex-col">
                <div>
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
                    <Button onclick={handleSimInput} disabled={!simInputValue || submitting}>
                      {#if submitting}
                      <LoaderCircle class="animate-spin" />
                      {/if}
                      OK
                    </Button>
                    <Button disabled={submitting} onclick={cancelSimInput}>Cancel</Button>
                  </div>
                </div>

                <div class="flex-1 mt-4 space-y-1 overflow-y-auto">
                  {#if simInputMode === 'password'}
                    <p class="text-xs text-muted-foreground">Suggestion:</p>
                    <button
                      on:click={() => {
                        simInputValue = user.pin;
                        handleSimInput();
                      }}
                      class="w-full text-left p-2 text-xs cursor-pointer hover:bg-muted transition-colors radius"
                    >
                      {user.pin}
                    </button>
                  {/if}

                  {#if suggestionType}
                    {#if suggestionsLoading}
                      <p>Loading...</p>
                    {:else if filteredSuggestions.length > 0}
                      <p class="text-xs text-muted-foreground">
                        Suggestions:
                      </p>
                      <div class="max-h-full overflow-y-auto">
                        {#each filteredSuggestions as suggestion}
                          <button
                            on:click={() => {
                              if (suggestionType === "phone") {
                                simInputValue = (
                                  suggestion as UserDetails
                                ).phone;
                              } else if (suggestionType === "paybill") {
                                simInputValue = (
                                  suggestion as PaybillAccountDetails
                                ).paybill_number.toString();
                              } else if (suggestionType === "till") {
                                simInputValue = (
                                  suggestion as TillAccountDetails
                                ).till_number.toString();
                              }
                              handleSimInput();
                            }}
                            class="w-full text-left p-2 text-xs cursor-pointer hover:bg-muted transition-colors radius"
                          >
                            {#if suggestionType === "phone"}
                              {(suggestion as UserDetails).name} -{(
                                suggestion as UserDetails
                              ).phone}
                            {:else if suggestionType === "paybill"}
                              {(
                                suggestion as PaybillAccountDetails
                              ).paybill_number}
                            {:else if suggestionType === "till"}
                              {(suggestion as TillAccountDetails).till_number}
                            {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  {/if}
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
