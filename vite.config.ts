import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// @see https://tauri.app/start/frontend/sveltekit/
export default defineConfig({
  plugins: [sveltekit()],
  // Tauri expects a fixed port and fails if it is unavailable.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      // Don't watch the Rust side; cargo handles that.
      ignored: ['**/src-tauri/**']
    }
  }
});
