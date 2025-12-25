import { getContext, setContext } from 'svelte';
import { writable, type Writable, get } from 'svelte/store';
import { settings } from './stores/settings';
import type { AppSettings } from './api';

// --- Types ---

/** Represents a keyboard shortcut action. */
export interface KeymapAction {
	/** A unique identifier for the action (e.g., 'project.create'). */
	id: string;
	/** A human-readable name for the action. */
	name: string;
	/** The default keyboard shortcut combination (e.g., 'ctrl+shift+p'). Modifiers must be in alphabetical order. */
	defaultShortcut: string;
	/** The currently active keyboard shortcut. This can be customized by the user. */
	shortcut: string;
	/** The callback function to execute when the shortcut is triggered. */
	callback: (event: KeyboardEvent) => void;
}

// --- Store ---

/**
 * A Svelte store that holds all globally registered keymap actions,
 * keyed by their ID. This map stores the original definition of actions.
 */
const allKeymapActions: Writable<Map<string, KeymapAction>> = writable(new Map());

/**
 * A Svelte store that holds the currently active keyboard shortcuts,
 * keyed by their `shortcut` string. This is what the KeymapManager listens to.
 */
export const activeKeymaps: Writable<Map<string, KeymapAction>> = writable(new Map());

// --- Keymap Manager ---

const KEYMAP_CONTEXT_KEY = 'keymap_manager';

/**
 * Creates a canonical string representation of a keyboard shortcut from an event.
 * This version uses `event.code` to avoid issues with modifier keys changing the key value.
 * @param event The KeyboardEvent.
 * @returns A normalized string (e.g., 'alt+ctrl+p').
 */
export function eventToShortcutString(event: KeyboardEvent): string {
	const modifiers = [];
	if (event.altKey) modifiers.push('alt');
	if (event.ctrlKey) modifiers.push('ctrl');
	if (event.metaKey) modifiers.push('meta');
	if (event.shiftKey) modifiers.push('shift');

	const code = event.code;

	// Ignore lone modifier key presses
	if (
		code.startsWith('Control') ||
		code.startsWith('Shift') ||
		code.startsWith('Alt') ||
		code.startsWith('Meta')
	) {
		return '';
	}

	let key = '';

	if (code.startsWith('Key')) {
		key = code.substring(3).toLowerCase();
	} else if (code.startsWith('Digit')) {
		key = code.substring(5);
	} else if (code.startsWith('Arrow')) {
		key = code.substring(5).toLowerCase();
	} else {
		// Handle other keys that don't have a standard prefix
		switch (code) {
			case 'Comma':
				key = ',';
				break;
			case 'Period':
				key = '.';
				break;
			case 'Slash':
				key = '/';
				break;
			case 'Semicolon':
				key = ';';
				break;
			case 'Quote':
				key = "'";
				break;
			case 'Backquote':
				key = '`';
				break;
			case 'BracketLeft':
				key = '[';
				break;
			case 'BracketRight':
				key = ']';
				break;
			case 'Backslash':
				key = '\\';
				break;
			case 'Minus':
				key = '-';
				break;
			case 'Equal':
				key = '=';
				break;
			case 'Enter':
				key = 'enter';
				break;
			case 'Escape':
				key = 'esc';
				break;
			case 'Backspace':
				key = 'backspace';
				break;
			case 'Tab':
				key = 'tab';
				break;
			case 'Space':
				key = 'space';
				break;
			case 'Delete':
				key = 'del';
				break;
			default:
				key = code.toLowerCase();
		}
	}

	modifiers.sort();
	modifiers.push(key);

	return modifiers.join('+');
}

/**
 * A class to manage keyboard shortcuts throughout the application.
 * It handles registration, unregistration, and processing of keydown events,
 * and allows for user customization.
 */
export class KeymapManager {
	constructor() {
		this.handleKeyDown = this.handleKeyDown.bind(this);
	}

	/**
	 * Initializes and registers a list of default keymap actions.
	 * This should be called once, typically in the root layout.
	 * It also loads any custom keybindings from the settings store.
	 * @param defaults An array of default KeymapAction objects.
	 * @param appSettings The current application settings from the backend.
	 */
	initialize(defaults: Omit<KeymapAction, 'shortcut'>[], appSettings: AppSettings) {
		const customKeymaps = appSettings.custom_keymaps || {};
		const resolvedKeymaps: KeymapAction[] = [];
		const newAllKeymapActions = new Map<string, KeymapAction>();

		for (const def of defaults) {
			const customShortcut = customKeymaps[def.id];
			const action: KeymapAction = {
				...def,
				shortcut: customShortcut || def.defaultShortcut
			};
			newAllKeymapActions.set(action.id, action);
			resolvedKeymaps.push(action);
		}

		allKeymapActions.set(newAllKeymapActions);
		this.updateActiveKeymaps(resolvedKeymaps);
	}

	/**
	 * Updates the `activeKeymaps` store based on a list of resolved KeymapAction objects.
	 * This rebuilds the map keyed by the current shortcut string.
	 * @param resolvedKeymaps - The list of KeymapAction objects with their current shortcuts.
	 */
	private updateActiveKeymaps(resolvedKeymaps: KeymapAction[]) {
		const newActiveMap = new Map<string, KeymapAction>();
		for (const action of resolvedKeymaps) {
			newActiveMap.set(action.shortcut, action);
		}
		activeKeymaps.set(newActiveMap);
	}

	/**
	 * Updates the shortcut for a specific action and saves it to the backend settings.
	 * @param actionId - The ID of the action to update.
	 * @param newShortcut - The new shortcut string.
	 */
	updateKeybinding(actionId: string, newShortcut: string) {
		const action = get(allKeymapActions).get(actionId);
		if (!action) {
			console.warn(`Action with ID "${actionId}" not found.`);
			return;
		}

		// Check for conflicts before updating
		const currentActiveKeymaps = get(activeKeymaps);
		if (
			currentActiveKeymaps.has(newShortcut) &&
			currentActiveKeymaps.get(newShortcut)?.id !== actionId
		) {
			console.warn(`Shortcut "${newShortcut}" is already assigned to another action.`);
			// Potentially return a status or throw error to UI
			return false;
		}

		// Remove old shortcut from activeKeymaps if it exists
		activeKeymaps.update((map) => {
			const oldShortcut = action.shortcut;
			if (map.has(oldShortcut)) {
				map.delete(oldShortcut);
			}
			return map;
		});

		action.shortcut = newShortcut;
		allKeymapActions.update((map) => map.set(actionId, action)); // Update allKeymapActions with the new shortcut

		const currentSettings = get(settings);
		const newCustomKeymaps = { ...(currentSettings.custom_keymaps || {}) };
		newCustomKeymaps[actionId] = newShortcut;
		settings.set({ custom_keymaps: newCustomKeymaps });

		// Rebuild activeKeymaps with the updated action
		this.updateActiveKeymaps(Array.from(get(allKeymapActions).values()));
		return true;
	}

	/**
	 * Resets the shortcut for a specific action to its default.
	 * @param actionId - The ID of the action to reset.
	 */
	resetKeybinding(actionId: string) {
		const action = get(allKeymapActions).get(actionId);
		if (!action) {
			console.warn(`Action with ID "${actionId}" not found.`);
			return;
		}

		// If the shortcut is already the default, do nothing
		if (action.shortcut === action.defaultShortcut) {
			return;
		}

		// Remove old shortcut from activeKeymaps if it exists
		activeKeymaps.update((map) => {
			const oldShortcut = action.shortcut;
			if (map.has(oldShortcut)) {
				map.delete(oldShortcut);
			}
			return map;
		});

		action.shortcut = action.defaultShortcut;
		allKeymapActions.update((map) => map.set(actionId, action)); // Update allKeymapActions with the default shortcut

		const currentSettings = get(settings);
		const newCustomKeymaps = { ...(currentSettings.custom_keymaps || {}) };
		delete newCustomKeymaps[actionId];
		settings.set({ custom_keymaps: newCustomKeymaps });

		// Rebuild activeKeymaps with the updated action
		this.updateActiveKeymaps(Array.from(get(allKeymapActions).values()));
	}

	/**
	 * Resets all custom keybindings to their defaults.
	 */
	resetAllKeybindings() {
		settings.set({ custom_keymaps: {} });
		const resolvedKeymaps: KeymapAction[] = [];

		allKeymapActions.update((map) => {
			for (const def of map.values()) {
				const action: KeymapAction = {
					...def,
					shortcut: def.defaultShortcut
				};
				map.set(action.id, action);
				resolvedKeymaps.push(action);
			}
			return map;
		});

		this.updateActiveKeymaps(resolvedKeymaps);
	}

	/**
	 * Handles the global keydown event, checks for matching shortcuts, and executes them.
	 * @param event - The KeyboardEvent from the event listener.
	 */
	public handleKeyDown(event: KeyboardEvent) {
		// If an input or textarea is focused, do not trigger shortcuts
		const target = event.target as HTMLElement;
		if (target.matches('input') || target.matches('textarea')) {
			return;
		}

		const shortcutString = eventToShortcutString(event);
		if (!shortcutString) return;

		const action = get(activeKeymaps).get(shortcutString);

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

/**
 * Retrieves all registered (default and customized) keymap actions.
 * @returns A Map of all keymap actions, keyed by their ID.
 */
export function getAllKeymapActionsStore(): Writable<Map<string, KeymapAction>> {
	return allKeymapActions;
}
