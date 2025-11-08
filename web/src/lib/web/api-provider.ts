import { provideInvoke, type Invoke, isApiReady } from '../api';

console.log('Providing Web API providers (fetch & WebSocket)');

const webInvoke: Invoke = async (cmd, args) => {
  const response = await fetch('/rpc', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: Math.random(),
      method: cmd,
      params: args ? args : {},
    })
  });

  if (!response.ok) {
    const errorBody = await response.json().catch(() => null);
    const errorMessage = errorBody?.error?.message || `HTTP error! status: ${response.status}`;
    throw new Error(errorMessage);
  }

  const json = await response.json();

  if (json.error) {
    throw new Error(`Call '${cmd}' failed with ${json.error.message}`);
  }

  return json.result;
};

provideInvoke(webInvoke);
isApiReady.set(true);
