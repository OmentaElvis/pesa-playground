import { goto } from '$app/navigation';
import type { KeymapAction } from '$lib/keymap';

export function back() {
	window.history.back();
}

export function forward() {
	window.history.forward();
}

export const globalKeymapActions: KeymapAction[] = [
	// Navigation
	{
		id: 'navigation.back',
		name: 'Go Back',
		shortcut: 'alt+arrowleft',
		callback: back
	},
	{
		id: 'navigation.forward',
		name: 'Go Forward',
		shortcut: 'alt+arrowright',
		callback: forward
	},
	// Project
	{
		id: 'project.create',
		name: 'Create new project',
		shortcut: 'ctrl+shift+p',
		callback: () => goto('/projects/new')
	},
	// User
	{
		id: 'users.manage',
		name: 'Manage Users',
		shortcut: 'ctrl+shift+u',
		callback: () => goto('/users')
	},
	// Business
	{
		id: 'businesses.manage',
		name: 'Manage Businesses',
		shortcut: 'ctrl+shift+b',
		callback: () => goto('/businesses')
	},
	// Settings
	{
		id: 'settings.open',
		name: 'Open Settings',
		shortcut: 'ctrl+,',
		callback: () => goto('/settings')
	}
];
