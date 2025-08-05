import { invoke } from "@tauri-apps/api/core";

/**
 * Represents the data structure for creating a new business.
 */
export interface BusinessData {
  name: string;
  short_code: string;
}

/**
 * Represents the data structure for updating an existing business.
 */
export interface UpdateBusinessData {
  name?: string;
  short_code?: string;
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
}

export interface BusinessSummary {
  id: number;
  name: string;
  short_code: string;
}

export async function createBusiness(input: BusinessData): Promise<Business> {
  return await invoke("create_business", { input });
}

export async function getBusiness(id: number): Promise<BusinessDetails> {
  return await invoke("get_business", { id });
}

export async function getBusinesses(): Promise<BusinessSummary[]> {
  return await invoke("get_businesses");
}

export async function updateBusiness(
  id: number,
  input: UpdateBusinessData
): Promise<Business | null> {
  return await invoke("update_business", { id, input });
}

export async function deleteBusiness(id: number): Promise<void> {
  return await invoke("delete_business", { id });
}

/**
 * Represents the data structure for creating a new paybill account.
 */
export interface CreatePaybillAccountData {
  business_id: number;
  paybill_number: number;
  account_validation_regex?: string;
  validation_url?: string;
  confirmation_url?: string;
  initial_balance: number;
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
}

export interface PaybillAccount {
  account_id: number;
  business_id: number;
  paybill_number: number;
  account_validation_regex?: string;
  validation_url?: string;
  confirmation_url?: string;
}

export interface PaybillAccountDetails {
  account_id: number;
  business_id: number;
  paybill_number: number;
  account_validation_regex?: string;
  validation_url?: string;
  confirmation_url?: string;
  balance: number;
  created_at: string;
}

export async function createPaybillAccount(
  input: CreatePaybillAccountData
): Promise<PaybillAccount> {
  return await invoke("create_paybill_account", { input });
}

export async function getPaybillAccount(
  id: number
): Promise<PaybillAccountDetails> {
  return await invoke("get_paybill_account", { id });
}

export async function getPaybillAccounts(): Promise<PaybillAccountDetails[]> {
  return await invoke("get_paybill_accounts");
}

export async function getPaybillAccountsByBusinessId(business_id: number): Promise<PaybillAccountDetails[]> {
  return await invoke("get_paybill_accounts_by_business_id", { businessId: business_id });
}

export async function updatePaybillAccount(
  id: number,
  input: UpdatePaybillAccountData
): Promise<PaybillAccount | null> {
  return await invoke("update_paybill_account", { id, input });
}

export async function deletePaybillAccount(id: number): Promise<void> {
  return await invoke("delete_paybill_account", { id });
}

/**
 * Represents the data structure for creating a new till account.
 */
export interface CreateTillAccountData {
  business_id: number;
  till_number: number;
  store_number: number;
  initial_balance: number;
  location_description?: string;
}

export interface UpdateTillAccountData {
  business_id?: number;
  till_number?: number;
  store_number?: number;
  location_description?: string;
}

export interface TillAccount {
  account_id: number;
  business_id: number;
  till_number: number;
  store_number: number;
  location_description?: string;
}

export interface TillAccountDetails {
  account_id: number;
  business_id: number;
  till_number: number;
  store_number: number;
  location_description?: string;
  balance: number;
  created_at: string;
}

export async function createTillAccount(
  input: CreateTillAccountData
): Promise<TillAccount> {
  return await invoke("create_till_account", { input });
}

export async function getTillAccount(id: number): Promise<TillAccountDetails> {
  return await invoke("get_till_account", { id });
}

export async function getTillAccounts(): Promise<TillAccountDetails[]> {
  return await invoke("get_till_accounts");
}

export async function getTillAccountsByBusinessId(business_id: number): Promise<TillAccountDetails[]> {
  return await invoke("get_till_accounts_by_business_id", { businessId: business_id });
}

export async function updateTillAccount(
  id: number,
  input: UpdateTillAccountData
): Promise<TillAccount | null> {
  return await invoke("update_till_account", { id, input });
}

export async function deleteTillAccount(id: number): Promise<void> {
  return await invoke("delete_till_account", { id });
}

export async function createAccount(account_type: string, initial_balance: number): Promise<number> {
  return await invoke("create_account", { accountType: account_type, initialBalance: initial_balance });
}


export enum SimulationMode {
  AlwaysSuccess = "AlwaysSuccess",
  AlwaysFail = "AlwaysFail",
  Realistic = "Realistic",
  Random = "Random",
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
export async function createProject(
  input: ProjectData
): Promise<Project> {
  return await invoke("create_project", { input });
}

/**
 * Calls the `get_project` Tauri command to retrieve a single project by its ID.
 * @param id The ID of the project to retrieve.
 * @returns A Promise resolving to `ProjectDetails` on success, or rejecting with an error string.
 */
export async function getProject(id: number): Promise<ProjectDetails> {
  return await invoke("get_project", { id });
}

/**
 * Calls the `get_projects` Tauri command to retrieve a list of all projects.
 * @returns A Promise resolving to an array of `ProjectSummary` on success, or rejecting with an error string.
 */
export async function getProjects(): Promise<ProjectSummary[]> {
  return await invoke("get_projects");
}

export async function getProjectsByBusinessId(business_id: number): Promise<ProjectSummary[]> {
  return await invoke("get_projects_by_business_id", { businessId: business_id });
}

/**
 * Calls the `update_project` Tauri command to update an existing project.
 * @param id The ID of the project to update.
 * @param input The partial project data to update.
 * @returns A Promise resolving to `void` on success, or rejecting with an error string.
 */
export async function updateProject(
  id: number,
  input: UpdateProjectData
): Promise<Project | null> {
  return await invoke("update_project", { id, input });
}

/**
 * Calls the `delete_project` Tauri command to delete a project and its associated data.
 * @param id The ID of the project to delete.
 * @returns A Promise resolving to `void` on success, or rejecting with an error string.
 */
export async function deleteProject(id: number): Promise<void> {
  return await invoke("delete_project", { id });
}


export interface User {
    id: number;
    name: string;
    phone: string;
    pin: string;
}

export interface UserDetails {
    id: number;
    name: string;
    phone: string;
    pin: string;
    balance: number;
}

export async function getUsers(): Promise<UserDetails[]> {
  return await invoke("get_users")
}

export async function getUser(user_id: number): Promise<UserDetails | null> {
  return await invoke("get_user", {
    userId: user_id
  })
}
export async function createUser(
  name: string,
  phone: string,
  balance: number,
  pin: string
): Promise<number> {
  return await invoke("create_user", {
    name,
    phone,
    balance,
    pin,
  });
}
export async function removeUser(user_id: number): Promise<void> {
  return await invoke("remove_user", {
    userId: user_id
  })
}
export async function updateUser(
  user_id: number,
  name?: string,
  balance?: number,
  pin?: string,
): Promise<void> {
  return await invoke("update_user", {
    userId: user_id,
    name,
    balance,
    pin,
  });
}

export async function generateUser(): Promise<UserDetails> {
  return await invoke("generate_user");  
}
export async function generateUsers(count: number): Promise<UserDetails[]> {
  return await invoke("generate_users", {count: count});  
}

// Main Transaction interface
export enum TransactionStatus {
  Pending = "pending",
  Completed = "completed",
  Failed = "failed",
  Reversed = "reversed",
}

export enum TransactionType {
  Paybill = "paybill",
  BuyGoods = "buy_goods",
  SendMoney = "send_money",
  Withdraw = "withdraw",
  Deposit = "deposit",
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
}

// Filter interface for frontend use
export interface TransactionFilter {
  to?: number;
  from?: number;
  transaction_type?: string;
  status?: string;
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
}

export async function getTransaction(transaction_id: string): Promise<Transaction | null> {
  return await invoke("get_transaction", { transactionId: transaction_id });
}

export async function listTransactions(filter: TransactionFilter = {}): Promise<Transaction[]> {
  return await invoke("list_transactions", { filter });
}

export async function countTransactions(filter: TransactionFilter = {}): Promise<number> {
  return await invoke("count_transactions", { filter });
}

export async function getTransactionByCheckoutRequest(
  checkoutRequestId: string
): Promise<Transaction | null> {
  return await invoke("get_transaction_by_checkout_request", { checkoutRequestId });
}

export async function getUserTransactions(
  user_id: number,
  limit?: number,
  offset?: number,
): Promise<Transaction[]> {
  return await invoke("get_user_transactions", { userId: user_id, limit, offset });
}

export async function getRecentTransactions(limit?: number): Promise<Transaction[]> {
  return await invoke("get_recent_transactions", { limit });
}

export async function getTransactionStats(projectId?: number): Promise<TransactionStats> {
  return await invoke("get_transaction_stats", { projectId });
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
    return await invoke("create_transaction_cost", { data });
}

export async function listTransactionCosts(): Promise<TransactionCost[]> {
    return await invoke("list_transaction_costs");
}

export async function updateTransactionCost(id: number, data: TransactionCostData): Promise<TransactionCost> {
    return await invoke("update_transaction_cost", { id, data });
}

export async function deleteTransactionCost(id: number): Promise<void> {
    return await invoke("delete_transaction_cost", { id });
}

export interface FullTransactionLog {
    transaction_id: string;
    transaction_date: string;
    transaction_amount: number;
    transaction_type: string;
    from_name: string;
    to_name: string;
    new_balance: number;
    status: string;
    fee: number;
}

export async function listFullTransactionLogs(account_id: number, limit?: number, offset?: number): Promise<FullTransactionLog[]> {
    return await invoke("list_full_transaction_logs", { accountId: account_id, limit, offset });
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
  return await invoke("get_api_log", { logId: log_id });
}

export async function updateApiLog(
  log_id: string,
  status_code?: number,
  request_body?: string,
  response_body?: string,
  error_desc?: string,
): Promise<ApiLog | null> {
  return await invoke("update_api_log", { logId: log_id, statusCode: status_code, requestBody: request_body, responseBody: response_body, errorDesc: error_desc });
}

export async function deleteApiLog(log_id: string): Promise<boolean> {
  return await invoke("delete_api_log", { logId: log_id });
}

export async function countApiLogs(filter: ApiLogFilter = {}): Promise<number> {
  return await invoke("count_api_logs", { filter });
}

export async function getProjectApiLogs(projectId: number, filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_project_api_logs", { projectId, filter });
}

export async function getApiLogsByMethod(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_api_logs_by_method", { filter });
}

export async function startSandbox(project_id: number) : Promise<string> {
  return await invoke("start_sandbox", {
    projectId: project_id
  })
}

export async function stopSandbox(project_id: number): Promise<void> {
  return await invoke("stop_sandbox", {
    projectId: project_id
  })
}

export async function sandboxStatus(project_id: number): Promise<{status: string, port: number, error?: string}> {
  return await invoke("sandbox_status", {
    projectId: project_id
  })
}

export type UserResponse =
  | { accepted: { pin: string } }
  | "cancelled"
  | "offline"
  | "timeout"
  | { failed: string };

export async function resolveStkPrompt(checkout_id: string, result: UserResponse): Promise<void> {
  return await invoke("resolve_stk_prompt", {
    checkoutId: checkout_id,
    result: result,
  })
}

export function formatTransactionAmount(amount: number): string {
  return `KES ${(amount / 100).toFixed(2)}`;
}

export function formatTransactionDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}
