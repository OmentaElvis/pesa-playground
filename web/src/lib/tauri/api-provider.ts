import { provideInvoke, isApiReady } from '../api';
import { invoke } from '@tauri-apps/api/core';

provideInvoke(invoke);
isApiReady.set(true);
