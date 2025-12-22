import { get, type Writable } from 'svelte/store';
import { page } from '$app/stores';
import { browser } from '$app/environment';
import { replaceState } from '$app/navigation';

// --- Types ---
type SyncedStores<T extends Record<string, Writable<any>>> = {
	[K in keyof T]: T[K];
};

interface SyncOptions {
	/**
	 * Defines how values are written to the URL.
	 * - `explicit`: Always writes the value (e.g., `?param=value`). This is the default.
	 * - `clean`: Removes the parameter from the URL if the value matches the store's initial default value.
	 */
	strategy?: 'explicit' | 'clean';
}

interface SyncManager {
	destroy: () => void;
}

// --- Module State ---
let debounceTimer: number | null = null;

// --- Private Functions ---
function stringify<T>(value: T, defaultValue: T, strategy: 'explicit' | 'clean'): string | null {
	if (value === null || value === undefined) {
		return null;
	}
	if (strategy === 'clean' && value === defaultValue) {
		return null;
	}
	return String(value);
}

function flushChangesToUrl(pendingChanges: Map<string, string | null>): void {
	if (!browser || pendingChanges.size === 0) {
		return;
	}
	const url = new URL(get(page).url);
	for (const [key, value] of pendingChanges.entries()) {
		if (value === null) {
			url.searchParams.delete(key);
		} else {
			url.searchParams.set(key, value);
		}
	}
	pendingChanges.clear();
	replaceState(url, get(page).state);
}

// --- Public API ---
function sync<T extends Record<string, Writable<any>>>(
	stores: SyncedStores<T>,
	options: SyncOptions = {}
): SyncManager {
	if (!browser) {
		return { destroy: () => {} };
	}

	const { strategy = 'explicit' } = options;
	const unsubscribers: (() => void)[] = [];
	const storeDefaults = new Map<Writable<any>, any>();
	const pendingChanges = new Map<string, string | null>();

	const $page = get(page);
	for (const key in stores) {
		const store = stores[key];
		const defaultValue = get(store);
		storeDefaults.set(store, defaultValue);

		const urlValue = $page.url.searchParams.get(key);
		if (urlValue !== null) {
			let valueToSet: any;
			if (typeof defaultValue === 'number') {
				valueToSet = Number(urlValue) || defaultValue;
			} else if (typeof defaultValue === 'boolean') {
				valueToSet = urlValue === 'true';
			} else {
				valueToSet = urlValue;
			}
			if (get(store) !== valueToSet) {
				store.set(valueToSet);
			}
		}
	}

	for (const key in stores) {
		const store = stores[key];
		const unsubscribe = store.subscribe((value) => {
			const defaultValue = storeDefaults.get(store);
			pendingChanges.set(key, stringify(value, defaultValue, strategy));

			if (debounceTimer) clearTimeout(debounceTimer);
			debounceTimer = window.setTimeout(() => {
				flushChangesToUrl(pendingChanges);
				debounceTimer = null;
			}, 200);
		});
		unsubscribers.push(unsubscribe);
	}

	const destroy = () => {
		if (debounceTimer) clearTimeout(debounceTimer);
		unsubscribers.forEach((unsub) => unsub());
	};

	return { destroy };
}

export const urlStateManager = {
	sync
};
