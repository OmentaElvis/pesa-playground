import { writable } from 'svelte/store';

/**
 * Stores the ID of the user detail page that is currently active.
 * This is used to prevent creating notifications for transactions
 * that the user is already looking at.
 */
export const activeUserPageId = writable<number | null>(null);
