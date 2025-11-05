import { writable } from 'svelte/store';
import { listApiLogs, listen, type ApiLog, type UnlistenFn } from '$lib/api';

function createApiLogStore() {
  const logs = writable<ApiLog[]>([]);
  let isInitialized = false;
  let unlisten: UnlistenFn | null = null;

  async function init() {
    if (isInitialized) {
      return;
    }
    isInitialized = true;

    try {
      const initialLogs = await listApiLogs({ limit: 50 });
      logs.set(initialLogs.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()));

      unlisten = await listen<ApiLog>('new_api_log', (event) => {
        logs.update(currentLogs => [event.payload, ...currentLogs]);
      });
    } catch (error) {
      console.error('Failed to initialize API log store:', error);
      isInitialized = false; // Allow retrying if initialization failed
    }
  }

  function destroy() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    isInitialized = false;
    logs.set([]);
  }

  return {
    subscribe: logs.subscribe,
    init,
    destroy,
  };
}

export const apiLogStore = createApiLogStore();
