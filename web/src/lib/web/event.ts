/**
 * Web platform implementation of the event API.
 *
 * Provides a compatible `listen` function that mimics Tauri's event API,
 * but uses a WebSocket connection to an Axum backend instead.
 */

import type { Listen } from '$lib/api';
import { provideListen } from '$lib/api';

// Registry of event listeners by event name
const listeners: Record<string, Array<(event: { event: string; payload: any }) => void>> = {};

let socket: WebSocket | null = null;
let socketStatus: 'disconnected' | 'connecting' | 'connected' = 'disconnected';

/**
 * Establishes a WebSocket connection if one does not already exist.
 */
function connect() {
  if (socket && socketStatus !== 'disconnected') return;

  socketStatus = 'connecting';
  console.log('[WebSocket] Connecting...');

  const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

  socket = new WebSocket(wsUrl);

  socket.onopen = () => {
    socketStatus = 'connected';
    console.log('[WebSocket] Connected');
  };

  socket.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data);

      if (message.event && listeners[message.event]) {
        const payload = { event: message.event, payload: message.payload };
        for (const handler of listeners[message.event]) handler(payload);
      }
    } catch (err) {
      console.error('[WebSocket] Failed to parse message:', err);
    }
  };

  socket.onclose = () => {
    socketStatus = 'disconnected';
    socket = null;
    console.log('[WebSocket] Disconnected');
    // Optional: auto-reconnect logic can go here
  };

  socket.onerror = (err) => {
    console.error('[WebSocket] Error:', err);
    socket?.close();
  };
}

/**
 * Web implementation of `listen(event, handler)`, compatible with the unified API.
 */
export const webListen: Listen = async (event, handler) => {
  connect();

  if (!listeners[event]) {
    listeners[event] = [];
  }

  listeners[event].push(handler);

  const unlisten = async () => {
    const handlers = listeners[event];
    if (!handlers) return;
    const index = handlers.indexOf(handler);
    if (index !== -1) handlers.splice(index, 1);
    if (handlers.length === 0) delete listeners[event];
  };

  return unlisten;
};

provideListen(webListen);
