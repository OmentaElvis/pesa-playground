import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { visualizer } from 'rollup-plugin-visualizer';

import path from 'path';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async ({ mode }) => {
	const target = mode === 'tauri' ? 'tauri' : 'web';

	return {
		plugins: [
			sveltekit(),
			tailwindcss(),
			visualizer({
				filename: 'build/monaco-bundle-report.html',
				brotliSize: true,
				gzipSize: true,
				open: false
			})
		],
		resolve: {
			alias: {
				'$runtime/api-provider': path.resolve(`src/lib/${target}/api-provider.ts`),
				'$runtime/event': path.resolve(`src/lib/${target}/event.ts`)
			}
		},
		// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
		//
		// 1. prevent Vite from obscuring rust errors
		clearScreen: false,
		// 2. tauri expects a fixed port, fail if that port is not available
		server: {
			port: target === 'tauri' ? 1420 : 5173,
			strictPort: true,
			host: host || false,
			hmr: host
				? {
						protocol: 'ws',
						host,
						port: 1421
					}
				: undefined,
			watch: {},

			proxy:
				target === 'web'
					? {
							'/rpc': {
								target: 'http://127.0.0.1:3000',
								changeOrigin: true
							},
							'/ws': {
								target: 'http://127.0.0.1:3000',
								ws: true
							}
						}
					: undefined
		}
	};
});
