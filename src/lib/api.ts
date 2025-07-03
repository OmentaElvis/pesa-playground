import { invoke } from "@tauri-apps/api/core";

/**
 * Represents the data structure for creating a new project.
 * Corresponds to `ProjectData` in Rust.
 */
export interface ProjectData {
  name: string;
  shortcode?: string;
  callback_url?: string;
  simulation_mode: string;
  stk_delay: number;
  initial_users?: number;
  prefix?: string;
  created_at?: string;
}

/**
 * Represents the data structure for updating an existing project.
 * All fields are optional as not all may be updated at once.
 * Corresponds to `UpdateProjectData` in Rust.
 */
export interface UpdateProjectData {
  name?: string;
  shortcode?: string;
  callback_url?: string;
  simulation_mode?: string;
  stk_delay?: number;
  prefix?: string;
}

/**
 * Represents the credentials returned after creating a project.
 * Corresponds to `ProjectCredentials` in Rust.
 */
export interface ProjectCredentials {
  consumer_key: string;
  consumer_secret: string;
  project_id: number;
}

/**
 * Represents the full details of a project, including its API keys.
 * Corresponds to `ProjectDetails` in Rust.
 */
export interface ProjectDetails {
  id: number;
  name: string;
  shortcode?: string;
  callback_url?: string;
  simulation_mode: string;
  stk_delay: number;
  prefix?: string;
  created_at?: string;
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
  name: string;
  simulation_mode: string;
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
): Promise<ProjectCredentials> {
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

/**
 * Calls the `update_project` Tauri command to update an existing project.
 * @param id The ID of the project to update.
 * @param input The partial project data to update.
 * @returns A Promise resolving to `void` on success, or rejecting with an error string.
 */
export async function updateProject(
  id: number,
  input: UpdateProjectData
): Promise<void> {
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
    id: number,
    project_id: number,
    phone: String,
    name: String,
    balance: number,
    pin: String,
    status?: String,
    created_at?: String,
}

export async function getUsers(project: number): Promise<User[]> {
  return await invoke("get_users", {
    projectId: project
  })
}

export async function getUser(user_id: number): Promise<User | null> {
  return await invoke("get_user", {
    userId: user_id
  })
}
export async function createUser(user: Partial<User>): Promise<number> {
  return await invoke("create_user", {
    projectId: user.project_id,
    name: user.name,
    phone: user.phone,
    balance: user.balance,
    pin: user.pin
  })
}
export async function removeUser(user_id: number): Promise<void> {
  return await invoke("remove_user", {
    userId: user_id
  })
}
export async function updateUser(user_id: number, user: Partial<User>): Promise<void> {
  return await invoke("remove_user", {
    userId: user_id,
    ...user
  })
}

export async function generateUser(): Promise<{name: string, phone: string, balance: number, pin: string}> {
  return await invoke("generate_user");  
}

// Main Transaction interface
export interface Transaction {
  id: string;
  project_id: string;
  user_id: string;
  phone: string;
  amount: number;
  short_code?: string;
  account_reference?: string;
  transaction_desc?: string;
  status: string;
  result_code?: string;
  result_desc?: string;
  checkout_request_id?: string;
  merchant_request_id?: string;
  created_at: string;
  completed_at?: string;
  mpesa_receipt_number?: string;
}

// Request interfaces
export interface CreateTransactionRequest {
  project_id: string;
  user_id: string;
  phone: string;
  amount: number;
  short_code?: string;
  account_reference?: string;
  transaction_desc?: string;
  status: string;
  checkout_request_id?: string;
  merchant_request_id?: string;
}

export interface UpdateTransactionRequest {
  status?: string;
  result_code?: string;
  result_desc?: string;
  completed_at?: string;
  mpesa_receipt_number?: string;
}

// Statistics interface
export interface TransactionStats {
  total_count: number;
  successful_count: number;
  pending_count: number;
  failed_count: number;
}

// Filter interface for frontend use
export interface TransactionFilter {
  project_id?: string;
  user_id?: string;
  phone?: string;
  status?: string;
  result_code?: string;
  limit?: number;
  offset?: number;
}

// Core CRUD operations
export async function createTransaction(request: CreateTransactionRequest): Promise<Transaction> {
  return await invoke("create_transaction", { request });
}

export async function getTransaction(transactionId: string): Promise<Transaction | null> {
  return await invoke("get_transaction", { transactionId });
}

export async function updateTransaction(
  transactionId: string,
  request: UpdateTransactionRequest
): Promise<Transaction | null> {
  return await invoke("update_transaction", { transactionId, request });
}

export async function deleteTransaction(transactionId: string): Promise<boolean> {
  return await invoke("delete_transaction", { transactionId });
}

export async function listTransactions(filter: TransactionFilter = {}): Promise<Transaction[]> {
  return await invoke("list_transactions", {
    projectId: filter.project_id,
    userId: filter.user_id,
    phone: filter.phone,
    status: filter.status,
    resultCode: filter.result_code,
    limit: filter.limit,
    offset: filter.offset,
  });
}

export async function countTransactions(filter: TransactionFilter = {}): Promise<number> {
  return await invoke("count_transactions", {
    projectId: filter.project_id,
    userId: filter.user_id,
    phone: filter.phone,
    status: filter.status,
    resultCode: filter.result_code,
  });
}

// Specialized functions
export async function getTransactionByCheckoutRequest(
  checkoutRequestId: string
): Promise<Transaction | null> {
  return await invoke("get_transaction_by_checkout_request", { checkoutRequestId });
}

export async function getUserTransactions(
  userId: string,
  limit: number = 20,
  offset: number = 0
): Promise<Transaction[]> {
  return await invoke("get_user_transactions", { userId, limit, offset });
}

export async function getProjectTransactions(
  projectId: string,
  status?: string,
  limit: number = 50,
  offset: number = 0
): Promise<Transaction[]> {
  return await invoke("get_project_transactions", { projectId, status, limit, offset });
}

export async function getRecentTransactions(limit: number = 10): Promise<Transaction[]> {
  return await invoke("get_recent_transactions", { limit });
}

export async function getTransactionStats(projectId?: string): Promise<TransactionStats> {
  return await invoke("get_transaction_stats", { projectId });
}

// Utility functions for common use cases
export async function getPendingTransactions(
  projectId?: string,
  limit: number = 50
): Promise<Transaction[]> {
  return await listTransactions({
    project_id: projectId,
    status: "PENDING",
    limit,
    offset: 0,
  });
}

export async function getSuccessfulTransactions(
  projectId?: string,
  limit: number = 50
): Promise<Transaction[]> {
  return await listTransactions({
    project_id: projectId,
    status: "SUCCESS",
    limit,
    offset: 0,
  });
}

export async function getFailedTransactions(
  projectId?: string,
  limit: number = 50
): Promise<Transaction[]> {
  return await listTransactions({
    project_id: projectId,
    status: "FAILED",
    limit,
    offset: 0,
  });
}

export async function getTransactionsByPhone(
  phone: string,
  projectId?: string,
  limit: number = 20
): Promise<Transaction[]> {
  return await listTransactions({
    project_id: projectId,
    phone,
    limit,
    offset: 0,
  });
}

// Pagination helper
export async function getTransactionsPage(
  page: number,
  pageSize: number = 20,
  filter: Omit<TransactionFilter, 'limit' | 'offset'> = {}
): Promise<{ transactions: Transaction[]; total: number; page: number; pageSize: number }> {
  const offset = (page - 1) * pageSize;
  
  const [transactions, total] = await Promise.all([
    listTransactions({ ...filter, limit: pageSize, offset }),
    countTransactions(filter),
  ]);

  return {
    transactions,
    total,
    page,
    pageSize,
  };
}

// Transaction status helpers
export const TransactionStatus = {
  PENDING: "PENDING",
  SUCCESS: "SUCCESS", 
  FAILED: "FAILED",
  CANCELLED: "CANCELLED",
} as const;

export type TransactionStatusType = typeof TransactionStatus[keyof typeof TransactionStatus];

// Helper function to format transaction amount
export function formatTransactionAmount(amount: number, currency: string = "KES"): string {
  return new Intl.NumberFormat('en-KE', {
    style: 'currency',
    currency: currency,
  }).format(amount);
}

// Helper function to format transaction date
export function formatTransactionDate(dateString: string): string {
  return new Intl.DateTimeFormat('en-KE', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(dateString));
}

// Helper function to get transaction status color/variant for UI
export function getTransactionStatusVariant(status: string): 'success' | 'warning' | 'error' | 'default' {
  switch (status.toUpperCase()) {
    case 'SUCCESS':
      return 'success';
    case 'PENDING':
      return 'warning';
    case 'FAILED':
    case 'CANCELLED':
      return 'error';
    default:
      return 'default';
  }
}

// Main ApiLog interface
export interface ApiLog {
  id: number;
  project_id: number;
  method: string;
  path: string;
  status_code: number;
  request_body?: string;
  response_body?: string;
  created_at: string;
  error_desc?: string;
}

// Request interfaces
export interface CreateApiLogRequest {
  project_id: number;
  method: string;
  path: string;
  status_code: number;
  request_body?: string;
  response_body?: string;
}

export interface UpdateApiLogRequest {
  status_code?: number;
  request_body?: string;
  response_body?: string;
}

// Statistics interface
export interface ApiLogStats {
  total_count: number;
  success_count: number;
  client_error_count: number;
  server_error_count: number;
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

// Core CRUD operations
export async function createApiLog(request: CreateApiLogRequest): Promise<ApiLog> {
  return await invoke("create_api_log", { request });
}

export async function getApiLog(logId: string): Promise<ApiLog | null> {
  return await invoke("get_api_log", { logId });
}

export async function updateApiLog(
  logId: string,
  request: UpdateApiLogRequest
): Promise<ApiLog | null> {
  return await invoke("update_api_log", { logId, request });
}

export async function deleteApiLog(logId: string): Promise<boolean> {
  return await invoke("delete_api_log", { logId });
}

export async function listApiLogs(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("list_api_logs", {
    projectId: filter.project_id,
    method: filter.method,
    path: filter.path,
    statusCode: filter.status_code,
    limit: filter.limit,
    offset: filter.offset,
  });
}

export async function countApiLogs(filter: ApiLogFilter = {}): Promise<number> {
  return await invoke("count_api_logs", {
    projectId: filter.project_id,
    method: filter.method,
    path: filter.path,
    statusCode: filter.status_code,
  });
}

export async function getProjectApiLogs(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_project_api_logs", {
    projectId: filter.project_id,
    method: filter.method,
    limit: filter.limit,
    offset: filter.offset,    
  })
}
export async function getApiLogsByMethod(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_api_logs_by_method", {
    projectId: filter.project_id,
    method: filter.method,
    limit: filter.limit,
  })
}

export async function getErrorApiLogs(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_error_api_logs", {
    projectId: filter.project_id,
    method: filter.method,
    limit: filter.limit,
  })
}

export async function getRecentApiLogs(filter: ApiLogFilter = {}): Promise<ApiLog[]> {
  return await invoke("get_recent_api_logs", {
    projectId: filter.project_id,
    method: filter.method,
    limit: filter.limit,
  })
}

export async function cleanupOldApiLogs(days_to_keep: number): Promise<number> {
  return await invoke("cleanup_old_api_logs", {
    daysTokeep: days_to_keep
  })
}

export async function getApiLogStats(project_id: number): Promise<ApiLogStats> {
  return await invoke("get_recent_api_logs", {
    projectId: project_id,
  })
}

export async function startSandbox(project_id: number) : Promise<String> {
  return await invoke("start_sandbox", {
    projectId: project_id
  })
}

export async function stopSandbox(project_id: number): Promise<void> {
  return await invoke("stop_sandbox", {
    projectId: project_id
  })
}

export async function sandboxStatus(project_id: number): Promise<{status: String, port: number, error?: string}> {
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
