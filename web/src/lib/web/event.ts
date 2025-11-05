import { provideListen, type Listen, type ListenerFn } from '$lib/api';
import { connectionStore } from '../stores/connectionStore';
import { footerWidgetStore } from '../stores/footerWidgetStore';
import ConnectionStatusWidget from '../components/widgets/ConnectionStatusWidget.svelte';

console.log('Providing Web (WebSocket) listen implementation');

const eventListeners = new Map<string, Set<ListenerFn>>();
let socket: WebSocket | null = null;
let retryTimeout: any = null;
let retryCount = 0;

function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws`;

    console.log('[WebSocket] Attempting to connect...');
    connectionStore.setStatus('connecting');

    socket = new WebSocket(url);

    socket.onopen = () => {
        console.log('[WebSocket] Connection established.');
        connectionStore.setStatus('connected');
        retryCount = 0;
        if (retryTimeout) {
            clearTimeout(retryTimeout);
            retryTimeout = null;
        }
    };

    socket.onmessage = (event) => {
        try {
            const parsed = JSON.parse(event.data);
            if (parsed.event && eventListeners.has(parsed.event)) {
                eventListeners.get(parsed.event)?.forEach(handler => handler(parsed));
            }
        } catch (e) {
            console.error('[WebSocket] Failed to parse message:', e);
        }
    };

    socket.onclose = () => {
        console.log('[WebSocket] Connection closed.');
        connectionStore.setStatus('disconnected');
        socket = null;

        // Exponential backoff retry logic
        const delay = Math.min(1000 * Math.pow(2, retryCount), 30000); // Max 30 seconds
        retryCount++;
        console.log(`[WebSocket] Will attempt to reconnect in ${delay / 1000} seconds...`);
        retryTimeout = setTimeout(connect, delay);
    };

    socket.onerror = (err) => {
        console.error('[WebSocket] Error:', err);
        // onclose will be called automatically after an error, triggering the retry logic.
    };
}

const webListen: Listen = async (event, handler) => {
    if (!eventListeners.has(event)) {
        eventListeners.set(event, new Set());
    }
    eventListeners.get(event)?.add(handler);

    // Return an unlisten function
    return () => {
        if (eventListeners.has(event)) {
            eventListeners.get(event)?.delete(handler);
            if (eventListeners.get(event)?.size === 0) {
                eventListeners.delete(event);
            }
        }
    };
};

// --- Initialization ---

// Provide the listen implementation to the rest of the app
provideListen(webListen);

// Start the WebSocket connection
connect();

// Register the connection status widget in the footer
footerWidgetStore.register({
    id: 'connection-status',
    component: ConnectionStatusWidget,
});