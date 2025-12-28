/**
 * This file defines the core API for interacting with the backend.
 * It uses a dependency injection pattern for the `invoke` function.
 * The actual implementation of `invoke` is provided by the specific application
 */

import { writable } from 'svelte/store';
import { formatAmount } from './utils';

export const isApiReady = writable(false);

export type Invoke = <T>(cmd: string, args?: any) => Promise<T>;

const defaultInvoke: Invoke = (cmd) => {
	throw new Error(`'invoke' function has not been provided. Called with: ${cmd}`);
};

export let invoke: Invoke = defaultInvoke;

export const provideInvoke = (implementation: Invoke) => {
	invoke = implementation;
};

export type UnlistenFn = () => Promise<void> | void;
export type ListenerFn<T = any> = (event: { event: string; payload: T }) => void;
export type Listen = <T = any>(event: string, handler: ListenerFn<T>) => Promise<UnlistenFn>;

// Default implementation throws if not provided.
const defaultListen: Listen = async (event) => {
	throw new Error(`'listen' function has not been provided. Called with: ${event}`);
};

// Holds the actual implementations.
let listenImpl: Listen = defaultListen;

// Dependency injector for the event API.
export const provideListen = (implementation: Listen) => {
	listenImpl = implementation;
};

// Public interface to use.
export const listen: Listen = (...args) => listenImpl(...args);

// All the function and type exports from the old file remain the same.
// They will now use the `invoke` variable defined above.

export enum AccountType {
	User = 'user',
	System = 'system',
	Mmf = 'mmf',
	Utility = 'utility'
}

export interface Account {
	id: number;
	account_type: AccountType;
	balance: number;
	created_at: string;
	disabled: boolean;
}

export async function getAccount(id: number): Promise<Account | null> {
	return invoke('get_account', {
		id
	});
}

export async function createAccount(
	accountType: AccountType,
	initialBalance: number
): Promise<number> {
	return await invoke('create_account', { accountType, initialBalance });
}

/**
 * Represents the data structure for creating a new business.
 */
export interface BusinessData {
	name: string;
	short_code: string;
	initial_working_balance: number;
	initial_utility_balance: number;
}

/**
 * Represents the data structure for updating an existing business.
 */
export interface UpdateBusinessData {
	name?: string;
	short_code?: string;
}

export interface MmfAccount {
	account_id: number;
	business_id: number;
	balance: number;
	created_at: string;
	disabled: boolean;
}

export interface UtilityAccount {
	account_id: number;
	business_id: number;
	balance: number;
	created_at: string;
	disabled: boolean;
}

export interface Business {
	id: number;
	name: string;
	short_code: string;
}

export interface BusinessDetails {
	id: number;
	name: string;
	short_code: string;

	mmf_account: MmfAccount;
	utility_account: UtilityAccount;
	charges_amount: number;
}

export interface BusinessSummary {
	id: number;
	name: string;
	short_code: string;
}

export interface BusinessOperator {
	id: number;
	username: string;
	password: string;
}

export async function createBusiness(input: BusinessData): Promise<Business> {
	return await invoke('create_business', { input });
}

export async function getBusiness(id: number): Promise<BusinessDetails> {
	return await invoke('get_business', { id });
}

export async function getBusinesses(): Promise<BusinessSummary[]> {
	return await invoke('get_businesses');
}

export async function revenueSettlements(businessId: number) {
	return await invoke('revenue_settlement', {
		businessId
	});
}

export async function updateBusiness(
	id: number,
	input: UpdateBusinessData
): Promise<Business | null> {
	return await invoke('update_business', { id, input });
}

export async function deleteBusiness(id: number): Promise<void> {
	return await invoke('delete_business', { id });
}

export async function createOperator(
	business_id: number,
	username: string,
	password_raw: string
): Promise<void> {
	return await invoke('create_operator', {
		input: {
			business_id,
			username,
			password: password_raw
		}
	});
}

export async function getOperatorsByBusinessId(businessId: number): Promise<BusinessOperator[]> {
	return await invoke('get_operators_by_business', {
		businessId
	});
}

export async function deleteOperator(operatorId: number): Promise<void> {
	return await invoke('delete_operator', {
		operatorId
	});
}

export enum C2BResponseType {
	Canceled = 'Canceled',
	Completed = 'Completed'
}

/**
 * Represents the data structure for creating a new paybill account.
 */
export interface CreatePaybillAccountData {
	business_id: number;
	paybill_number: number;
	account_validation_regex?: string;
	response_type?: C2BResponseType;
	validation_url?: string;
	confirmation_url?: string;
}

/**
 * Represents the data structure for updating an existing paybill account.
 */
export interface UpdatePaybillAccountData {
	business_id?: number;
	paybill_number?: number;
	account_validation_regex?: string;
	validation_url?: string;
	confirmation_url?: string;
	response_type?: C2BResponseType;
}

export interface PaybillAccount {
	id: number;
	business_id: number;
	paybill_number: number;
	account_validation_regex?: string;
	validation_url?: string;
	confirmation_url?: string;
	response_type?: C2BResponseType;
}

export interface PaybillAccountDetails {
	id: number;
	business_id: number;
	paybill_number: number;
	account_validation_regex?: string;
	validation_url?: string;
	confirmation_url?: string;
	response_type?: C2BResponseType;
	balance: number;
	created_at: string;
}

export async function createPaybillAccount(
	input: CreatePaybillAccountData
): Promise<PaybillAccount> {
	return await invoke('create_paybill_account', { input });
}

export async function getPaybillAccount(id: number): Promise<PaybillAccountDetails> {
	return await invoke('get_paybill_account', { id });
}

export async function getPaybillAccounts(): Promise<PaybillAccountDetails[]> {
	return await invoke('get_paybill_accounts');
}

export async function getMmfAccount(id: number): Promise<MmfAccount> {
	return await invoke('get_mmf_account', { id });
}

export async function getUtilityAccount(id: number): Promise<UtilityAccount> {
	return await invoke('get_utility_account', { id });
}

export async function getMmfAccountByBusinessId(businessId: number): Promise<MmfAccount> {
	return await invoke('get_mmf_account_by_business_id', { businessId });
}

export async function getUtilityAccountByBusinessId(businessId: number): Promise<UtilityAccount> {
	return await invoke('get_utility_account_by_business_id', { businessId });
}

export async function getPaybillAccountsByBusinessId(
	business_id: number
): Promise<PaybillAccountDetails[]> {
	return await invoke('get_paybill_accounts_by_business_id', { businessId: business_id });
}

export async function updatePaybillAccount(
	id: number,
	input: UpdatePaybillAccountData
): Promise<PaybillAccount | null> {
	return await invoke('update_paybill_account', { id, input });
}

export async function deletePaybillAccount(id: number): Promise<void> {
	return await invoke('delete_paybill_account', { id });
}

/**
 * Represents the data structure for creating a new till account.
 */
export interface CreateTillAccountData {
	business_id: number;
	till_number: number;
	store_number: number;
	response_type?: C2BResponseType;
	validation_url?: string;
	confirmation_url?: string;
	location_description?: string;
}

export interface UpdateTillAccountData {
	business_id?: number;
	till_number?: number;
	store_number?: number;
	location_description?: string;
	response_type?: C2BResponseType;
	validation_url?: string;
	confirmation_url?: string;
}

export interface TillAccount {
	id: number;
	business_id: number;
	till_number: number;
	store_number: number;
	location_description?: string;
}

export interface TillAccountDetails {
	id: number;
	business_id: number;
	till_number: number;
	store_number: number;
	location_description?: string;
	balance: number;
	created_at: string;
	response_type?: C2BResponseType;
	validation_url?: string;
	confirmation_url?: string;
}

export async function createTillAccount(input: CreateTillAccountData): Promise<TillAccount> {
	return await invoke('create_till_account', { input });
}

export async function getTillAccount(id: number): Promise<TillAccountDetails> {
	return await invoke('get_till_account', { id });
}

export async function getTillAccounts(): Promise<TillAccountDetails[]> {
	return await invoke('get_till_accounts');
}

export async function getTillAccountsByBusinessId(
	business_id: number
): Promise<TillAccountDetails[]> {
	return await invoke('get_till_accounts_by_business_id', { businessId: business_id });
}

export async function updateTillAccount(
	id: number,
	input: UpdateTillAccountData
): Promise<TillAccount | null> {
	return await invoke('update_till_account', { id, input });
}

export async function deleteTillAccount(id: number): Promise<void> {
	return await invoke('delete_till_account', { id });
}

export enum SimulationMode {
	AlwaysSuccess = 'AlwaysSuccess',
	AlwaysFail = 'AlwaysFail',
	Realistic = 'Realistic',
	Random = 'Random'
}

/**
 * Represents the data structure for creating a new project.
 * Corresponds to `ProjectData` in Rust.
 */
export interface ProjectData {
	business_id: number;
	name: string;
	callback_url?: string;
	simulation_mode: SimulationMode;
	stk_delay: number;
	prefix?: string;
}

/**
 * Represents the data structure for updating an existing project.
 * All fields are optional as not all may be updated at once.
 * Corresponds to `UpdateProjectData` in Rust.
 */
export interface UpdateProjectData {
	business_id?: number;
	name?: string;
	callback_url?: string;
	simulation_mode?: SimulationMode;
	stk_delay?: number;
	prefix?: string;
}

export interface Project {
	id: number;
	business_id: number;
	name: string;
	callback_url?: string;
	simulation_mode: SimulationMode;
	stk_delay: number;
	prefix?: string;
	created_at: string;
}

/**
 * Represents the full details of a project, including its API keys.
 * Corresponds to `ProjectDetails` in Rust.
 */
export interface ProjectDetails {
	id: number;
	business_id: number;
	name: string;
	callback_url?: string;
	simulation_mode: SimulationMode;
	stk_delay: number;
	prefix?: string;
	created_at: string;
	consumer_key: string;
	consumer_secret: string;
	passkey: string;
}

/**
 * Represents a summarized version of a project, suitable for listing.
 * Corresponds to `ProjectSummary` in Rust.
 */
export interface ProjectSummary {
	id: number;
	business_id: number;
	business_name: String;
	short_code: String;
	name: string;
	simulation_mode: SimulationMode;
	created_at: string;
}

// Functions to call the Tauri commands

/**
 * Calls the `create_project` Tauri command to create a new project.
 * @param input The project data to create.
 * @returns A Promise resolving to `ProjectCredentials` on success, or rejecting with an error string.
 */
export async function createProject(input: ProjectData): Promise<Project> {
	return await invoke('create_project', { input });
}

/**
 * Calls the `get_project` Tauri command to retrieve a single project by its ID.
 * @param id The ID of the project to retrieve.
 * @returns A Promise resolving to `ProjectDetails` on success, or rejecting with an error string.
 */
export async function getProject(id: number): Promise<ProjectDetails> {
	return await invoke('get_project', { id });
}

/**
 * Calls the `get_projects` Tauri command to retrieve a list of all projects.
 * @returns A Promise resolving to an array of `ProjectSummary` on success, or rejecting with an error string.
 */
export async function getProjects(): Promise<ProjectSummary[]> {
	return await invoke('get_projects');
}

export async function getProjectsByBusinessId(business_id: number): Promise<ProjectSummary[]> {
	return await invoke('get_projects_by_business_id', { businessId: business_id });
}

/**
 * Calls the `update_project` Tauri command to update an existing project.
 * @param id The ID of the project to update.
 * @param input The partial project data to update.
 * @returns A Promise resolving to `void` on success, or rejecting with an error string.
 */
export async function updateProject(id: number, input: UpdateProjectData): Promise<Project | null> {
	return await invoke('update_project', { id, input });
}

/**
 * Calls the `delete_project` Tauri command to delete a project and its associated data.
 * @param id The ID of the project to delete.
 * @returns A Promise resolving to `void` on success, or rejecting with an error string.
 */
export async function deleteProject(id: number): Promise<void> {
	return await invoke('delete_project', { id });
}

export interface User {
	account_id: number;
	name: string;
	phone: string;
	pin: string;
  balance: number,
  disabled: boolean,
  created_at: string,
  registered_at: String,
  last_swap_date?: String,
  imsi: String,
}

export async function getUsers(): Promise<User[]> {
	return await invoke('get_users');
}

export async function getUser(user_id: number): Promise<User | null> {
	return await invoke('get_user', {
		userId: user_id
	});
}

export async function getUserByPhone(phone: string): Promise<User | null> {
	return await invoke('get_user_by_phone', {
		phone
	});
}

export async function createUser(
	name: string,
	phone: string,
	balance: number,
	pin: string
): Promise<number> {
	return await invoke('create_user', {
		name,
		phone,
		balance,
		pin
	});
}
export async function removeUser(user_id: number): Promise<void> {
	return await invoke('remove_user', {
		userId: user_id
	});
}
export async function updateUser(
	user_id: number,
	name?: string,
	pin?: string,
	phone?: string
): Promise<void> {
	return await invoke('update_user', {
		userId: user_id,
		name,
		pin,
		phone
	});
}

export async function generateUser(): Promise<User> {
	return await invoke('generate_user');
}
export async function generateUsers(count: number): Promise<User[]> {
	return await invoke('generate_users', { count: count });
}

// Transaction Notes
export type AccountTypeForFunding = 'Utility' | 'Mmf' | 'User';

export interface PaybillPaymentNoteData {
	paybill_number: number;
	bill_ref_number: string;
}

export interface TillPaymentNoteData {
	till_number: number;
}

export interface AccountSetupFundingNoteData {
	account_type: AccountTypeForFunding;
}

export type TransactionNote =
	| { type: 'PaybillPayment'; data: PaybillPaymentNoteData }
	| { type: 'TillPayment'; data: TillPaymentNoteData }
	| { type: 'AccountSetupFunding'; data: AccountSetupFundingNoteData };

// Main Transaction interface
export enum TransactionStatus {
	Pending = 'pending',
	Completed = 'completed',
	Failed = 'failed',
	Reversed = 'reversed'
}

export enum TransactionType {
	Paybill = 'paybill',
	BuyGoods = 'buy_goods',
	SendMoney = 'send_money',
	Withdraw = 'withdraw',
	Deposit = 'deposit',
	ChargeSettlement = 'charge_settlement',
	RevenueSweep = 'revenue_sweep'
}

export interface Transaction {
	id: string;
	from?: number;
	to: number;
	amount: number;
	currency: string;
	transaction_type: TransactionType;
	status: TransactionStatus;
	reversal_of?: string;
	created_at: string;
	updated_at?: string;
	notes?: TransactionNote;
}

// Filter interface for frontend use
export interface TransactionFilter {
	to?: number;
	from?: number;
	transaction_type?: string;
	status?: TransactionStats;
	result_code?: string;
	limit?: number;
	offset?: number;
}

// Statistics interface
export interface TransactionStats {
	total_count: number;
	successful_count: number;
	pending_count: number;
	failed_count: number;
	total_fees: number;
	total_volume: number;
}

export async function transfer(
	from: number | null,
	destination: number,
	amount: number,
	txnType: TransactionType,
	notes?: TransactionNote
): Promise<Transaction> {
	return await invoke('transfer', {
		source: from,
		destination,
		amount,
		txnType,
		notes
	});
}

export async function reverse(id: String): Promise<Transaction> {
	return await invoke('reverse', { id });
}

export async function getTransaction(transaction_id: string): Promise<Transaction | null> {
	return await invoke('get_transaction', { transactionId: transaction_id });
}

export async function listTransactions(filter: TransactionFilter = {}): Promise<Transaction[]> {
	return await invoke('list_transactions', { filter });
}
export async function listSystemTransactions(
	limit: number | null = null,
	offset: number | null = null
): Promise<Transaction[]> {
	return await invoke('list_system_transactions', { limit, offset });
}

export async function countTransactions(filter: TransactionFilter = {}): Promise<number> {
	return await invoke('count_transactions', { filter });
}

export async function getTransactionByCheckoutRequest(
	checkoutRequestId: string
): Promise<Transaction | null> {
	return await invoke('get_transaction_by_checkout_request', { checkoutRequestId });
}

export async function getUserTransactions(
	user_id: number,
	limit?: number,
	offset?: number
): Promise<Transaction[]> {
	return await invoke('get_user_transactions', { userId: user_id, limit, offset });
}

export async function getRecentTransactions(limit?: number): Promise<Transaction[]> {
	return await invoke('get_recent_transactions', { limit });
}

export async function getTransactionStats(projectId?: number): Promise<TransactionStats> {
	return await invoke('get_transaction_stats', { projectId });
}

export interface TransactionCost {
	id: number;
	transaction_type: string;
	min_amount: number;
	max_amount: number;
	fee_fixed?: number;
	fee_percentage?: number;
}

export interface TransactionCostData {
	transaction_type: string;
	min_amount: number;
	max_amount: number;
	fee_fixed?: number;
	fee_percentage?: number;
}

export async function createTransactionCost(data: TransactionCostData): Promise<TransactionCost> {
	return await invoke('create_transaction_cost', { data });
}

export async function listTransactionCosts(): Promise<TransactionCost[]> {
	return await invoke('list_transaction_costs');
}

export async function updateTransactionCost(
	id: number,
	data: TransactionCostData
): Promise<TransactionCost> {
	return await invoke('update_transaction_cost', { id, data });
}

export async function deleteTransactionCost(id: number): Promise<void> {
	return await invoke('delete_transaction_cost', { id });
}

export async function calculateTransactionFee(txnType: TransactionType, amount: number) {
	return await invoke('calculate_transaction_fee', {
		txnType,
		amount
	});
}

export interface FullTransactionLog {
	transaction_id: string;
	transaction_date: string;
	transaction_amount: number;
	transaction_type: TransactionType;
	from_name: string;
	to_name: string;
	from_id: number | null;
	to_id: number;
	new_balance: number;
	status: TransactionStatus;
	fee: number;
	direction: TransactionDirection;
}

export async function listFullTransactionLogs(
	account_id: number,
	limit?: number,
	offset?: number
): Promise<FullTransactionLog[]> {
	return await invoke('list_full_transaction_logs', { accountId: account_id, limit, offset });
}

export async function listAccountsFullTransactionLogs(
	accounts: number[],
	limit?: number,
	offset?: number
): Promise<FullTransactionLog[]> {
	return await invoke('list_accounts_full_transaction_logs', { accounts, limit, offset });
}

export async function countTransactionLogs(accounts: number[]): Promise<number> {
	return await invoke('count_transaction_logs', { accounts });
}

export type TransactionDirection = 'Inflow' | 'Outflow';
export type SortDirection = 'Asc' | 'Desc';

export interface TransactionHistoryEntry {
	transaction_id: string;
	date: string;
	status: TransactionStatus;
	transaction_type: TransactionType;
	fee: number;
	amount: number;

	sender_name: string;
	sender_id?: number;
	sender_balance?: number;

	receiver_name: string;
	receiver_id: number;
	receiver_balance?: number;

	notes?: TransactionNote;
}

export type HistoryScopeType = 'User' | 'Business' | 'All';

export interface HistoryScope {
	type: HistoryScopeType;
	id?: number;
}

export interface Sorting {
	by: string;
	direction: SortDirection;
}

export interface Filters {
	statuses?: TransactionStatus[];
	search_query?: string;
}

export interface Pagination {
	limit: number;
	offset: number;
}

export interface HistoryFilter {
	scope: HistoryScope;
	pagination: Pagination;
	sorting?: Sorting;
	filters?: Filters;
}

export async function getTransactionHistory(
	filter: HistoryFilter
): Promise<TransactionHistoryEntry[]> {
	return await invoke('get_transaction_history', { filter });
}

// Main ApiLog interface
export interface ApiLog {
	id: string;
	project_id: number;
	method: string;
	path: string;
	status_code: number;
	request_body?: string;
	response_body?: string;
	error_desc?: string;
	created_at: string;
}

export enum LogLevel {
	Trace = 'Trace',
	Debug = 'Debug',
	Info = 'Info',
	Warn = 'Warn',
	Error = 'Error'
}

export enum Theme {
	Dark = 'dark',
	Light = 'light'
}

export interface EncryptionKeys {
	public_key: string;
	// private_key: string, // private key is not really private in this sandbox
}

export interface AppSettings {
	theme: Theme;
	server_log_level: LogLevel;
	encryption_keys?: EncryptionKeys;
	custom_keymaps?: Record<string, string> | null;
}

export async function getSettings(): Promise<AppSettings> {
	return await invoke('get_settings');
}

export async function setSettings(settings: AppSettings): Promise<void> {
	return await invoke('set_settings', { settings });
}

export async function generateSecurityCredential(password: string): Promise<string> {
	return await invoke('generate_security_credential', { password });
}

// Filter interface for frontend use
export interface ApiLogFilter {
	project_id?: number;
	method?: string;
	path?: string;
	status_code?: number;
	limit?: number;
	offset?: number;
}

// Statistics interface
export interface ApiLogStats {
	total_count: number;
	success_count: number;
	client_error_count: number;
	server_error_count: number;
}

export async function getApiLog(log_id: string): Promise<ApiLog | null> {
	return await invoke('get_api_log', { logId: log_id });
}

export async function updateApiLog(
	log_id: string,
	status_code?: number,
	request_body?: string,
	response_body?: string,
	error_desc?: string
): Promise<ApiLog | null> {
	return await invoke('update_api_log', {
		logId: log_id,
		statusCode: status_code,
		requestBody: request_body,
		responseBody: response_body,
		errorDesc: error_desc
	});
}

export async function deleteApiLog(log_id: string): Promise<boolean> {
	return await invoke('delete_api_log', { logId: log_id });
}

export async function countApiLogs(filter: ApiLogFilter = {}): Promise<number> {
	return await invoke('count_api_logs', { filter });
}

export async function getProjectApiLogs(
	projectId: number,
	filter: ApiLogFilter = {}
): Promise<ApiLog[]> {
	return await invoke('get_project_api_logs', { projectId, filter });
}

export async function getApiLogsByMethod(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
	return await invoke('get_api_logs_by_method', { filter });
}

export async function listApiLogs(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
	return await invoke('list_api_logs', { filter });
}

export async function listRunningSandboxes(): Promise<any[]> {
	return await invoke('list_running_sandboxes');
}

export async function startSandbox(project_id: number): Promise<string> {
	return await invoke('start_sandbox', {
		projectId: project_id
	});
}

export async function stopSandbox(project_id: number): Promise<void> {
	return await invoke('stop_sandbox', {
		projectId: project_id
	});
}

export async function sandboxStatus(
	project_id: number
): Promise<{ status: string; port: number; error?: string }> {
	return await invoke('sandbox_status', {
		projectId: project_id
	});
}

export type UserResponse =
	| { accepted: { pin: string } }
	| 'cancelled'
	| 'offline'
	| 'timeout'
	| { failed: string };

export async function resolveStkPrompt(checkout_id: string, result: UserResponse): Promise<void> {
	return await invoke('resolve_stk_prompt', {
		checkoutId: checkout_id,
		result: result
	});
}

export function formatTransactionAmount(amount: number): string {
	return formatAmount(amount / 100);
}

export function formatTransactionDate(dateString: string): string {
	const date = new Date(dateString);
	return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

export async function resolveAccountAndNavigate(
	id: number | null,
	goto: (
		href: string,
		opts?: {
			replaceState?: boolean | undefined;
			noScroll?: boolean | undefined;
			keepfocus?: boolean | undefined;
			state?: any;
		}
	) => Promise<void>
) {
	if (!id) return;

	let account = await getAccount(id);
	if (!account) return;

	switch (account.account_type) {
		case AccountType.Mmf:
			let paybill = await getMmfAccount(account.id);
			if (!paybill) return;
			await goto(`/businesses/${paybill.business_id}`);
			break;
		case AccountType.System:
			await goto('/account/system');
			break;
		case AccountType.Utility:
			let till = await getUtilityAccount(account.id);
			if (!till) return;
			await goto(`/businesses/${till.business_id}`);
			break;
		case AccountType.User:
			let user = await getUser(account.id);
			if (!user) return;
			await goto(`/users/${user.account_id}`);
			break;
	}
}

export async function closeSplashscreen(): Promise<void> {
	return await invoke('close_splashscreen');
}

export interface AppInfo {
	name: string;
	version: string;
	description: string;
	authors: string;
}

export async function getAppInfo(): Promise<AppInfo> {
	return await invoke('get_app_info');
}

export async function clearAllData(): Promise<void> {
	return await invoke('clear_all_data');
}

export type CalculatedDirection = 'Inflow' | 'Outflow' | 'Internal' | 'None';

export function getTransactionDirection(
	transaction: TransactionHistoryEntry,
	perspective: number | number[] | null | undefined
): CalculatedDirection {
	if (transaction.transaction_type === TransactionType.RevenueSweep) {
		return 'Internal';
	}

	if (perspective == null) {
		return 'None';
	}

	// For User and Business scopes
	if (
		(typeof perspective === 'number' && perspective > 0) ||
		(Array.isArray(perspective) && perspective.length > 0)
	) {
		const isReceiver = Array.isArray(perspective)
			? perspective.includes(transaction.receiver_id)
			: transaction.receiver_id === perspective;
		const isSender = Array.isArray(perspective)
			? perspective.includes(transaction.sender_id ?? -1)
			: transaction.sender_id === perspective;

		if (isReceiver) {
			return 'Inflow';
		}
		if (isSender) {
			return 'Outflow';
		}
		return 'None';
	}

	// For 'All' scope (System Perspective)
	if (perspective === 0) {
		if (transaction.sender_id == null || transaction.sender_id === 0) {
			return 'Outflow';
		}
		if (transaction.receiver_id === 0) {
			return 'Inflow';
		}
		// This transaction does not involve the system, so it's neutral
		return 'None';
	}

	return 'None';
}

export enum LipaPaymentType {
	Paybill = 'Paybill',
	Till = 'Till'
}

export interface LipaArgs {
	user_phone: String;
	amount: number;
	payment_type: LipaPaymentType;
	business_number: number;
	account_number?: String;
}

export async function lipa(args: LipaArgs): Promise<void> {
	return await invoke('lipa', { args });
}
