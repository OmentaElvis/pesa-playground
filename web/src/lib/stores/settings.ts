import { writable, type Writable } from 'svelte/store';
import { type AppSettings, LogLevel, Theme, getSettings, setSettings, listen } from '$lib/api';
import { toast } from 'svelte-sonner';

// Default settings
export const defaultAppSettings: AppSettings = {
	theme: Theme.Dark,
	server_log_level: LogLevel.Info
};

type SettingsStore = Writable<AppSettings> & {
	init: () => Promise<void>;
	set: (value: Partial<AppSettings>) => Promise<void>;
	reset: () => void;
};

function createSettingsStore(): SettingsStore {
	const { subscribe, set, update } = writable<AppSettings>(defaultAppSettings);

	return {
		subscribe,
		update,
		// Initialize the store by fetching from backend
		init: async () => {
			try {
				const initialSettings = await getSettings();
				set(initialSettings);
			} catch (error) {
				console.error('Failed to load initial settings:', error);
			}

			// Listen for settings updates from the backend
			await listen<AppSettings>('settings_updated', ({ payload }) => {
				set(payload);
			});
		},
		// Generic setter for partial updates
		set: async (partialSettings: Partial<AppSettings>) => {
			let newSettings: AppSettings | null = null;
			update((currentSettings) => {
				// Merge partial settings into current state
				newSettings = { ...currentSettings, ...partialSettings };
				return newSettings;
			});
			// After updating the local state, persist the full object to backend
			if (newSettings) {
				try {
					await setSettings(newSettings);
				} catch (error) {
					console.error('Failed to save settings to backend:', error);
					toast.error(`Failed to save settings to backend: ${error}`);
				}
			}
		},
		// A simple reset to default function
		reset: () => {
			set(defaultAppSettings);
			setSettings(defaultAppSettings);
		}
	};
}

export const settings = createSettingsStore();
