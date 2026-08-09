import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    // Bind IPv4 explicitly: on Windows, `localhost` often resolves to ::1 while
    // Tauri polls `devUrl` at 127.0.0.1, so the desktop shell never opens.
    port: 1420,
    strictPort: true,
    host: host || '127.0.0.1',
    proxy: {
      // Browser-mode: scrape services without CORS (Tauri uses Rust obs_collect).
      '/__obs/svc/gateway': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/gateway/, ''),
      },
      '/__obs/svc/matching': {
        target: 'http://127.0.0.1:8083',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/matching/, ''),
      },
      '/__obs/svc/risk': {
        target: 'http://127.0.0.1:8084',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/risk/, ''),
      },
      '/__obs/svc/liquidity': {
        target: 'http://127.0.0.1:8091',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/liquidity/, ''),
      },
      '/__obs/svc/execution': {
        target: 'http://127.0.0.1:8092',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/execution/, ''),
      },
      '/__obs/svc/smc': {
        target: 'http://127.0.0.1:8094',
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/__obs\/svc\/smc/, ''),
      },
    },
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : {
          protocol: 'ws',
          host: '127.0.0.1',
          port: 1420,
        },
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
