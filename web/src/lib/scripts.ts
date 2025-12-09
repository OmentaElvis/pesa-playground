import { invoke } from './api';

export interface Script {
	name: string;
	content: string;
}

export async function scriptsList(): Promise<string[]> {
	return await invoke('scripts_list');
}

export async function scriptsRead(name: string): Promise<string> {
	return await invoke('scripts_read', { name });
}

export async function scriptsSave(name: string, content: string): Promise<void> {
	return await invoke('scripts_save', { name, content });
}

export async function scriptsDelete(name: string): Promise<void> {
	return await invoke('scripts_delete', { name });
}

export async function scriptsExecute(content: string): Promise<string> {
	return await invoke('scripts_execute', { content });
}
