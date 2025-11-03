import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { type FullTransactionLog } from '$lib/api'
import { activeUserPageId } from './activePageStore';

export type TransactionDirection = 'Credit' | 'Debit';

const STORAGE_KEY = 'unread-transaction-logs';

// Helper to get the initial value from localStorage
function getInitialValue(): FullTransactionLog[] {
	if (!browser) {
		return [];
	}
	const storedValue = localStorage.getItem(STORAGE_KEY);
	return storedValue ? JSON.parse(storedValue) : [];
}

// --- The Store ---
function createTransactionLogStore() {
	const { subscribe, update, set } = writable<FullTransactionLog[]>(getInitialValue());

	subscribe((value) => {
		if (browser) {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
		}
	});

	return {
		subscribe,
		/**
		 * Adds a new transaction log to the store.
		 * Avoids adding duplicates based on transactionId.
		 */
		add: (log: FullTransactionLog) => {
			const currentActiveId = get(activeUserPageId);
			const isRelatedToActivePage = log.to_id === currentActiveId || log.from_id === currentActiveId;

			if (isRelatedToActivePage) {
				// If the user is on the page related to this transaction, don't add it to unread notifications.
				return;
			}

			update((logs) => {
				// A log is a duplicate only if it has the same transaction ID AND the same direction.
				if (
					logs.some(
						(existingLog) =>
							existingLog.transaction_id === log.transaction_id &&
							existingLog.direction === log.direction
					)
				) {
					return logs; // Do not add true duplicates
				}
				return [...logs, log];
			});
		},

		/**
		 * Removes all logs associated with a specific user ID.
		 * This is useful for "marking all as read" for a user.
		 */
		removeByUser: (userId: number) => {
			update((logs) => logs.filter((log) => log.to_id !== userId && log.from_id !== userId));
		},

		/**
		 * Removes a single log by its unique transaction ID.
		 */
		remove: (transactionId: string) => {
			update((logs) => logs.filter((log) => log.transaction_id !== transactionId));
		},

		/**
		 * Clears all notifications from the store.
		 */
		reset: () => {
			set([]);
		}
	};
}

export const transactionLogStore = createTransactionLogStore();
