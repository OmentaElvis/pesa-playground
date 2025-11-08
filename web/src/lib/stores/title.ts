import { writable } from 'svelte/store';

const defaultTitle = 'Pesa Playground';
const titleStore = writable(defaultTitle);

const isTauri = import.meta.env.MODE === 'tauri';

export async function setTitle(newTitle: string) {
	titleStore.set(newTitle);

	try {
		if (isTauri) {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			getCurrentWindow().setTitle(newTitle).catch(console.error);
		}
	} catch {
		document.title = newTitle;
	}
}

export async function resetTitle() {
	titleStore.set(defaultTitle);

	try {
		if (isTauri) {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			getCurrentWindow().setTitle(defaultTitle).catch(console.error);
		}
	} catch {
		document.title = defaultTitle;
	}
}

export const title = {
	subscribe: titleStore.subscribe,
	set: setTitle,
	update: titleStore.update
};
