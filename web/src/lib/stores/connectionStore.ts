
import { writable } from 'svelte/store';

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected';

function createConnectionStore() {
    const { subscribe, set } = writable<ConnectionStatus>('connecting');

    return {
        subscribe,
        setStatus: (status: ConnectionStatus) => set(status),
    };
}

export const connectionStore = createConnectionStore();
