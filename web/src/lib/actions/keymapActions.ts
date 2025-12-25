import { goto } from '$app/navigation';
import type { KeymapAction } from '$lib/keymap';
import { sandboxes } from '$lib/stores/sandboxStatus';
import { get } from 'svelte/store';

import { getProjects, startSandbox, type ProjectSummary, stopSandbox } from '$lib/api';
import { toast } from 'svelte-sonner';

export function back() {
	window.history.back();
}

export function forward() {
	window.history.forward();
}

export const globalKeymapActions: Omit<KeymapAction, 'shortcut'>[] = [
	// Navigation
	{
		id: 'navigation.back',
		name: 'Go Back',
		defaultShortcut: 'alt+left',
		callback: back
	},
	{
		id: 'navigation.forward',
		name: 'Go Forward',
		defaultShortcut: 'alt+right',
		callback: forward
	},
	// Project
	{
		id: 'project.create',
		name: 'Create new project',
		defaultShortcut: 'ctrl+shift+p',
		callback: () => goto('/projects/new')
	},
	// User
	{
		id: 'users.manage',
		name: 'Manage Users',
		defaultShortcut: 'ctrl+shift+u',
		callback: () => goto('/users')
	},
	// Business
	{
		id: 'businesses.manage',
		name: 'Manage Businesses',
		defaultShortcut: 'ctrl+shift+b',
		callback: () => goto('/businesses')
	},
	// Settings
	{
		id: 'settings.open',
		name: 'Open Settings',
		defaultShortcut: 'ctrl+,',
		callback: () => goto('/settings')
	}
];

/**
 * Dynamically generates keymap actions for toggling project sandboxes.
 * Assigns shortcuts like Ctrl+Shift+1, Ctrl+Shift+2, etc., to the first 9 projects.
 * @returns An array of dynamically generated KeymapAction objects.
 */
export async function generateProjectSandboxKeymaps(): Promise<Omit<KeymapAction, 'shortcut'>[]> {
	const projectKeymaps: Omit<KeymapAction, 'shortcut'>[] = [];
	try {
		const projects = (await getProjects()) as ProjectSummary[];
		projects.slice(0, 9).forEach((project, index) => {
			const numberKey = (index + 1).toString(); // 1-9
			projectKeymaps.push({
				id: `project.sandbox.toggle.${project.id}`,
				name: `Toggle Sandbox: ${project.name}`,
				defaultShortcut: `ctrl+shift+${numberKey}`,
				callback: async () => {
					try {
						const sandboxesMap = get(sandboxes);
						const sandboxInfo = sandboxesMap.get(project.id);
						const status = sandboxInfo?.status ?? 'off';

						if (status === 'on') {
							await stopSandbox(project.id);
						} else if (status === 'off' || status === 'error') {
							await startSandbox(project.id);
						}
					} catch (error) {
						toast.error(`Failed to toggle sandbox for "${project.name}": ${error}`);
						console.error(`Error toggling sandbox for project ${project.id}:`, error);
					}
				}
			});
		});
	} catch (error) {
		console.error('Failed to load projects for sandbox keymaps:', error);
		toast.error('Failed to load projects for sandbox keymaps.');
	}
	return projectKeymaps;
}
