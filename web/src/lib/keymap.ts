import { getContext, setContext } from 'svelte';
import { writable, type Writable } from 'svelte/store';

// --- Types ---

/** Represents a keyboard shortcut action. */
export interface KeymapAction {
	/** A unique identifier for the action (e.g., 'project.create'). */
	id: string;
	/** A human-readable name for the action. */
	name: string;
	/** The keyboard shortcut combination (e.g., 'ctrl+shift+p'). Modifiers must be in alphabetical order. */
	shortcut: string;
	/** The callback function to execute when the shortcut is triggered. */
	callback: (event: KeyboardEvent) => void;
}

// --- Store ---

/** A Svelte store that holds all registered keyboard shortcuts, keyed by their shortcut string. */
export const registeredShortcuts: Writable<Map<string, KeymapAction>> = writable(new Map());

// --- Keymap Manager ---

const KEYMAP_CONTEXT_KEY = 'keymap_manager';

/**
 * Creates a canonical string representation of a keyboard shortcut from an event.
 * @param event The KeyboardEvent.
 * @returns A normalized string (e.g., 'alt+ctrl+p').
 */
function eventToShortcutString(event: KeyboardEvent): string {
	const modifiers = [];
	if (event.altKey) modifiers.push('alt');
	if (event.ctrlKey) modifiers.push('ctrl');
	if (event.metaKey) modifiers.push('meta');
	if (event.shiftKey) modifiers.push('shift');

	const key = event.key.toLowerCase();

	// Avoid registering shortcuts with only modifiers
	if (['alt', 'control', 'shift', 'meta'].includes(key)) {
		return '';
	}

	modifiers.sort(); // Ensure consistent order
	modifiers.push(key);

	return modifiers.join('+');
}

/**
 * A class to manage keyboard shortcuts throughout the application.
 * It handles registration, unregistration, and processing of keydown events.
 */
export class KeymapManager {
	constructor() {
		this.handleKeyDown = this.handleKeyDown.bind(this);
	}

	/**
	 * Registers a list of keymap actions.
	 * @param actions - An array of KeymapAction objects to register.
	 * @returns An `unregister` function to clean up the registered shortcuts.
	 */
	register(actions: KeymapAction[]) {
		registeredShortcuts.update((shortcuts) => {
			for (const action of actions) {
				// Prevent overwriting existing shortcuts silently
				if (shortcuts.has(action.shortcut)) {
					console.warn(`Shortcut "${action.shortcut}" is already registered. Overwriting.`);
				}
				shortcuts.set(action.shortcut, action);
			}
			return shortcuts;
		});

		// Return an unregister function for cleanup (e.g., in onDestroy).
		return () => {
			registeredShortcuts.update((shortcuts) => {
				for (const action of actions) {
					shortcuts.delete(action.shortcut);
				}
				return shortcuts;
			});
		};
	}

	/**
	 * Handles the global keydown event, checks for matching shortcuts, and executes them.
	 * @param event - The KeyboardEvent from the event listener.
	 */
	public handleKeyDown(event: KeyboardEvent) {
		const shortcutString = eventToShortcutString(event);
		if (!shortcutString) return;

		let shortcutsMap: Map<string, KeymapAction> = new Map();
		const unsubscribe = registeredShortcuts.subscribe((value) => {
			shortcutsMap = value;
		});
		unsubscribe();

		const action = shortcutsMap.get(shortcutString);

		if (action) {
			event.preventDefault();
			event.stopPropagation();
			action.callback(event);
		}
	}
}

// --- Context Functions ---

/**
 * Creates a new KeymapManager instance and sets it in the Svelte context.
 * This should be called once in the root layout component.
 * @returns The created KeymapManager instance.
 */
export function createKeymapManager(): KeymapManager {
	const manager = new KeymapManager();
	setContext(KEYMAP_CONTEXT_KEY, manager);
	return manager;
}

/**
 * Retrieves the KeymapManager instance from the Svelte context.
 * This can be used in child components to register shortcuts.
 * @returns The KeymapManager instance.
 */
export function getKeymapManager(): KeymapManager {
	const manager = getContext<KeymapManager>(KEYMAP_CONTEXT_KEY);
	if (!manager) {
		throw new Error(
			'KeymapManager not found in context. Make sure createKeymapManager() is called in a parent component.'
		);
	}
	return manager;
}
