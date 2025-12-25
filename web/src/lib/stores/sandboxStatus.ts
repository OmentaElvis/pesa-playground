import { getProject, listRunningSandboxes, listen } from '$lib/api';
import { writable } from 'svelte/store';
import { toast } from 'svelte-sonner';

export type SandboxStatus = 'off' | 'starting' | 'on' | 'error';

interface SandboxStatusPayload {
	project_id: number;
	port: number;
	status: SandboxStatus;
	error?: string;
}

export interface SandboxInfo extends SandboxStatusPayload {
	name: string;
}

export async function getSandboxes() {
	try {
		let list: SandboxInfo[] = await listRunningSandboxes();
		let map = new Map();

		for (let info of list) {
			let project = await getProject(info.project_id);
			info.name = project.name;

			map.set(info.project_id, info);
		}

		sandboxes.update(() => map);
	} catch (err) {
		console.log(err);
	}
}

export const sandboxes = writable<Map<number, SandboxInfo>>(new Map());

let initialized = false;

export function initSandboxStatus() {
	if (initialized) {
		return;
	}

	listen('sandbox_status', async ({ payload }: { payload: SandboxStatusPayload }) => {
		let project = await getProject(payload.project_id);
		let info: SandboxInfo = {
			name: project.name,
			port: payload.port,
			project_id: payload.project_id,
			status: payload.status,
			error: payload.error
		};

		if (payload.status === 'on') {
			toast.success(`Sandbox for "${info.name}" started on port ${payload.port}.`);
		} else if (payload.status === 'error') {
			toast.error(`Sandbox for "${info.name}" failed: ${payload.error}`);
		} else if (payload.status === 'off') {
			toast.info(`Sandbox for "${info.name}" stopped.`);
		}

		sandboxes.update((m) => {
			if (payload.status === 'off') {
				m.delete(info.project_id);
			} else {
				m.set(info.project_id, info);
			}

			return m;
		});
	});

	getSandboxes();
	initialized = true;
}
