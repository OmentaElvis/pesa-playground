import { provideInvoke } from '../api';
import { invoke } from '@tauri-apps/api/core';

provideInvoke(invoke);
