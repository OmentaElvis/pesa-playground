import { listen as tauriListen } from '@tauri-apps/api/event';
import { provideListen } from '$lib/api';

provideListen(tauriListen);

