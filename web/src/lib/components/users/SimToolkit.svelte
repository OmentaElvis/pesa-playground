<script lang="ts">
	import { Button } from '../ui/button';
	import { Input } from '$lib/components/ui/input/index.js';
	import type { UserDetails, PaybillAccountDetails, TillAccountDetails } from '$lib/api';
	import { ArrowLeft, LoaderCircle } from 'lucide-svelte';
	import {
		getUsers,
		getPaybillAccounts,
		getTillAccounts,
		transfer,
		getUserByPhone,
		TransactionType,
		lipa,
		LipaPaymentType
	} from '$lib/api';
	import { toast } from 'svelte-sonner';

	let props: { user: UserDetails } = $props();
	let user: UserDetails = $derived(props.user || {});

	interface SimMenu {
		title: string;
		options: { id: string; label: string; action: () => void }[];
		message?: string;
		isInfo?: boolean;
		isSuccess?: boolean;
	}

	let simMenuStack: string[] = $state([]);
	let currentSimMenu = $state('main');
	let simFormData: Record<string, any> = {};
	let simInputMode: string | null = $state(null);
	let simInputValue = $state('');
	let simInputPrompt = $state('');
	let simInputCallback: ((arg0: string) => void) | null = $state(null);
	let suggestions: (UserDetails | PaybillAccountDetails | TillAccountDetails)[] = [];
	let suggestionType: 'phone' | 'paybill' | 'till' | null = $state(null);
	let suggestionsLoading = $state(false);
	let submitting = $state(false);

	type SubmitType = 'SendMoney' | 'BuyAirtime' | 'Paybill' | 'BuyGoodsTill';

	function navigateSimMenu(menuId: string, data = {}) {
		if (currentSimMenu !== menuId) {
			simMenuStack.push(currentSimMenu);
		}
		currentSimMenu = menuId;
		simFormData = { ...simFormData, ...data };
	}

	function goBackSimMenu() {
		if (simMenuStack.length > 0) {
			currentSimMenu = simMenuStack.pop() || '';
		}
	}

	async function showSimInput(
		prompt: string,
		callback: ((arg0: string) => void) | null,
		type = 'text',
		suggType: 'phone' | 'paybill' | 'till' | null = null
	) {
		simInputMode = type;
		simInputPrompt = prompt;
		simInputCallback = callback;
		simInputValue = '';
		suggestionType = suggType;

		if (suggestionType) {
			suggestionsLoading = true;
			if (suggestionType === 'phone') {
				suggestions = (await getUsers()).filter((u) => {
					return u.id != user.id;
				});
			} else if (suggestionType === 'paybill') {
				suggestions = await getPaybillAccounts();
			} else if (suggestionType === 'till') {
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
		simInputValue = '';
		suggestionType = null;
		suggestions = [];
	}

	function resetMenu() {
		cancelSimInput();
		currentSimMenu = 'main';
		simMenuStack = [];
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
			resetMenu();
		} catch (err) {
			toast.error(`${err}`);
		}
	}

	async function lipaNaPaybill() {
		try {
			let amount = Number(simFormData.amount) * 100;
			await lipa({
				amount,
				payment_type: LipaPaymentType.Paybill,
				business_number: Number(simFormData.business),
				account_number: simFormData.account,
				user_phone: user.phone
			});
			toast.info(
				`Transaction to paybill: ${simFormData.business} account name ${simFormData.account} initiated.`
			);
			resetMenu();
		} catch (err) {
			toast.error(`${err}`);
		}
	}

	async function lipaNaTill() {
		try {
			let amount = Number(simFormData.amount) * 100;
			await lipa({
				amount,
				payment_type: LipaPaymentType.Till,
				business_number: Number(simFormData.business),
				account_number: undefined,
				user_phone: user.phone
			});
			toast.info(`Transaction to till: ${simFormData.business} initiated.`);
			resetMenu();
		} catch (err) {
			toast.error(`${err}`);
		}
	}

	function incorrectPin() {
		toast.error(
			`Incorrect pin for: ${user.name}, use the suggested pin or type 0000 to override checks.`
		);
	}

	async function submit(submit_type: SubmitType) {
		submitting = true;
		switch (submit_type) {
			case 'SendMoney':
				if (simFormData.pin == '0000' || user.pin == simFormData.pin) {
					await transferMoneyToUser();
				} else {
					incorrectPin();
				}
				break;
			case 'Paybill':
				if (simFormData.pin == '0000' || user.pin == simFormData.pin) {
					await lipaNaPaybill();
				} else {
					incorrectPin();
				}
				break;
			case 'BuyGoodsTill':
				if (simFormData.pin == '0000' || user.pin == simFormData.pin) {
					await lipaNaTill();
				} else {
					incorrectPin();
				}
				break;
			case 'BuyAirtime':
				break;
		}
		submitting = false;
	}

	function sendMoney() {
		showSimInput(
			'Enter phone number:',
			(value) => {
				simFormData.phone = value;
				showSimInput(
					'Enter amount:',
					(amount) => {
						simFormData.amount = amount;
						showSimInput(
							'Enter M-PESA PIN:',
							async (pin) => {
								simFormData.pin = pin;
								await submit('SendMoney');
							},
							'password'
						);
					},
					'number'
				);
			},
			'text',
			'phone'
		);
	}

	async function selectFromPhonebook() {
		const users = (await getUsers()).filter((u) => {
			return u.id != user.id;
		});

		simMenus.phonebook = {
			title: 'Phonebook',
			options: users.map((user, i) => ({
				id: `user_${user.id}`,
				label: `${i + 1}. ${user.name}`,
				action: () => {
					simFormData.phone = user.phone;
					showSimInput(
						'Enter amount:',
						(amount) => {
							simFormData.amount = amount;
							showSimInput(
								'Enter M-PESA PIN:',
								async (pin) => {
									simFormData.pin = pin;
									await submit('SendMoney');
								},
								'password'
							);
						},
						'number'
					);
				}
			}))
		};
		navigateSimMenu('phonebook');
	}

	const simMenus: Record<string, SimMenu> = $derived({
		main: {
			title: 'M-PESA',
			options: [
				{
					id: 'send_money',
					label: '1. Send Money',
					action: () => navigateSimMenu('send_money')
				},
				{
					id: 'withdraw',
					label: '2. Withdraw Cash',
					action: () => navigateSimMenu('withdraw')
				},
				{
					id: 'buy_airtime',
					label: '3. Buy Airtime',
					action: () => navigateSimMenu('buy_airtime')
				},
				{
					id: 'lipa_na_mpesa',
					label: '4. Lipa na M-PESA',
					action: () => navigateSimMenu('lipa_na_mpesa')
				},
				{
					id: 'my_account',
					label: '5. My Account',
					action: () => navigateSimMenu('my_account')
				}
			]
		},
		send_money: {
			title: 'Send Money',
			options: [
				{
					id: 'enter_phone',
					label: '1. Enter Phone No.',
					action: () => sendMoney()
				},
				{
					id: 'from_phonebook',
					label: '2. From Phonebook',
					action: selectFromPhonebook
				}
			]
		},
		buy_airtime: {
			title: 'Buy Airtime',
			options: [
				{
					id: 'my_phone',
					label: '1. My Phone',
					action: () =>
						showSimInput(
							'Enter amount:',
							(amount) => {
								simFormData.amount = amount;
								simFormData.phone = user.phone;
								showSimInput(
									'Enter M-PESA PIN:',
									async (pin) => {
										simFormData.pin = pin;
										await submit('BuyAirtime');
									},
									'password'
								);
							},
							'number'
						)
				},
				{
					id: 'another_phone',
					label: '2. Another Phone',
					action: () =>
						showSimInput(
							'Enter phone number:',
							(phone) => {
								simFormData.phone = phone;
								showSimInput(
									'Enter amount:',
									(amount) => {
										simFormData.amount = amount;
										showSimInput(
											'Enter M-PESA PIN:',
											async (pin) => {
												simFormData.pin = pin;
												await submit('BuyAirtime');
											},
											'password'
										);
									},
									'number'
								);
							},
							'text',
							'phone'
						)
				}
			]
		},
		lipa_na_mpesa: {
			title: 'Lipa na M-PESA',
			options: [
				{
					id: 'paybill',
					label: '1. Pay Bill',
					action: () =>
						showSimInput(
							'Enter business number:',
							(business) => {
								simFormData.business = business;
								showSimInput('Enter account number:', (account) => {
									simFormData.account = account;
									showSimInput(
										'Enter amount:',
										(amount) => {
											simFormData.amount = amount;
											showSimInput(
												'Enter M-PESA PIN:',
												async (pin) => {
													simFormData.pin = pin;
													await submit('Paybill');
												},
												'password'
											);
										},
										'number'
									);
								});
							},
							'text',
							'paybill'
						)
				},
				{
					id: 'buy_goods',
					label: '2. Buy Goods and Services',
					action: () =>
						showSimInput(
							'Enter till number:',
							(till) => {
								simFormData.business = till;
								showSimInput(
									'Enter amount:',
									(amount) => {
										simFormData.amount = amount;
										showSimInput(
											'Enter M-PESA PIN:',
											async (pin) => {
												simFormData.pin = pin;
												await submit('BuyGoodsTill');
											},
											'password'
										);
									},
									'number'
								);
							},
							'text',
							'till'
						)
				}
			]
		},
		my_account: {
			title: 'My Account',
			options: [
				{
					id: 'mini_statement',
					label: '1. Mini Statement',
					action: () => navigateSimMenu('mini_statement')
				},
				{
					id: 'account_balance',
					label: '2. Account Balance',
					action: () => navigateSimMenu('account_balance')
				}
			]
		},
		mini_statement: {
			title: 'Mini statement',
			message: 'Not Implemented yet',
			isInfo: true,
			options: []
		},
		account_balance: {
			title: 'Account Balance',
			message: `Your Account balance is Ksh <b class="text-green-500">${
				user.balance.toFixed(2) || '0.00'
			}</b>`,
			isInfo: true,
			options: []
		},
		success: {
			title: 'Transaction Successful',
			message: 'Your transaction has been processed successfully.',
			isSuccess: true,
			options: []
		}
	});

	let filteredSuggestions = $derived(
		suggestions.filter((suggestion) => {
			if (!simInputValue) return true;
			if (suggestionType === 'phone') {
				const u = suggestion as UserDetails;
				return (
					u.phone.includes(simInputValue) ||
					u.name.toLowerCase().includes(simInputValue.toLowerCase())
				);
			} else if (suggestionType === 'paybill') {
				const p = suggestion as PaybillAccountDetails;
				return p.paybill_number.toString().includes(simInputValue);
			} else if (suggestionType === 'till') {
				const t = suggestion as TillAccountDetails;
				return t.till_number.toString().includes(simInputValue);
			}
			return true;
		})
	);
</script>

<div>
	<div class="h-full">
		<div class="h-full w-full rounded-lg text-sm">
			<div class="flex h-full flex-col p-3">
				{#if simInputMode}
					<div class="mt-8 flex flex-1 flex-col">
						<div class="mb-4">
							<div class="mb-2">STK Menu</div>
							<div class="text-xs">{simInputPrompt}</div>
						</div>

						<div class="flex flex-1 flex-col">
							<div>
								<div class="mb-4 p-2">
									<Input
										type={simInputMode === 'password'
											? 'password'
											: simInputMode === 'number'
												? 'number'
												: 'text'}
										bind:value={simInputValue}
										class="w-full bg-transparent outline-none"
										placeholder={simInputMode === 'password' ? '****' : 'Enter...'}
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

							<div class="mt-4 flex-1 space-y-1 overflow-y-auto">
								{#if simInputMode === 'password'}
									<p class="text-xs text-muted-foreground">Suggestion:</p>
									<button
										onclick={() => {
											simInputValue = user.pin;
											handleSimInput();
										}}
										class="radius w-full cursor-pointer p-2 text-left text-xs transition-colors hover:bg-muted"
									>
										{user.pin}
									</button>
								{/if}

								{#if suggestionType}
									{#if suggestionsLoading}
										<p>Loading...</p>
									{:else if filteredSuggestions.length > 0}
										<p class="text-xs text-muted-foreground">Suggestions:</p>
										<div class="max-h-full overflow-y-auto">
											{#each filteredSuggestions as suggestion}
												<button
													onclick={() => {
														if (suggestionType === 'phone') {
															simInputValue = (suggestion as UserDetails).phone;
														} else if (suggestionType === 'paybill') {
															simInputValue = (
																suggestion as PaybillAccountDetails
															).paybill_number.toString();
														} else if (suggestionType === 'till') {
															simInputValue = (
																suggestion as TillAccountDetails
															).till_number.toString();
														}
														handleSimInput();
													}}
													class="radius w-full cursor-pointer p-2 text-left text-xs transition-colors hover:bg-muted"
												>
													{#if suggestionType === 'phone'}
														{(suggestion as UserDetails).name} -{(suggestion as UserDetails).phone}
													{:else if suggestionType === 'paybill'}
														{(suggestion as PaybillAccountDetails).paybill_number}
													{:else if suggestionType === 'till'}
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
					<div class="flex w-full items-center justify-between">
						<div></div>
						{#if simMenuStack.length > 0}
							<Button variant="ghost" onclick={goBackSimMenu}>
								<ArrowLeft />
							</Button>
						{/if}
					</div>

					<div class="flex-1">
						{#if simMenus[currentSimMenu]?.options}
							<div class="space-y-1">
								{#each simMenus[currentSimMenu].options as option}
									<button
										class="radius w-full cursor-pointer p-2 text-left text-xs transition-colors hover:bg-muted"
										onclick={option.action}
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
</div>
