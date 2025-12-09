import { type Transport } from '@codemirror/lsp-client';
import { forwardLspRequest, listen, type UnlistenFn } from './api';

let unlisten: UnlistenFn | null = null;
let listeners: ((value: string) => void)[] = [];

const listenerFn = () => {
	return listen('lsp_notification', (arg) => {
		for (let h of listeners) h(arg.payload);
	});
};

let transport: Transport = {
	async send(message: string) {
		await forwardLspRequest(message);
	},
	async subscribe(handler) {
		if (!unlisten) {
			unlisten = await listenerFn();
		}

		listeners.push(handler);
	},
	unsubscribe(handler) {
		listeners = listeners.filter((h) => h != handler);

		if (listeners.length == 0 && unlisten) unlisten();
	}
};

export default transport;
