/**
 * Browser-accessible API base URL (no trailing slash).
 * Use for REST (fetch) and WebSocket origin.
 *
 * In the browser, host `localhost` is rewritten to `127.0.0.1`. On Windows, `localhost`
 * often resolves to IPv6 (::1) while a local gateway listens on IPv4 only, which breaks WebSockets.
 */

function normalizeLoopbackHostForBrowser(url: string): string {
  if (typeof window === 'undefined') return url;
  return url
    .replace(/^http:\/\/localhost\b/i, 'http://127.0.0.1')
    .replace(/^https:\/\/localhost\b/i, 'https://127.0.0.1')
    .replace(/^ws:\/\/localhost\b/i, 'ws://127.0.0.1')
    .replace(/^wss:\/\/localhost\b/i, 'wss://127.0.0.1');
}

export function getPublicApiBaseUrl(): string {
  const raw = import.meta.env.VITE_API_URL?.trim() || 'http://127.0.0.1:8080';
  const base = raw.replace(/\/+$/, '');
  return normalizeLoopbackHostForBrowser(base);
}

/**
 * Full WebSocket URL for the gateway stream (`/ws`).
 * Derived from `VITE_API_URL`, or set `VITE_WS_URL` for a different host/port.
 * The desktop/web UI talks to the gateway WebSocket directly (no local reverse proxy).
 */
export function getGatewayWebSocketUrl(): string {
  const explicit = import.meta.env.VITE_WS_URL?.trim();
  if (explicit) {
    const u = explicit.replace(/\/+$/, '');
    const withPath = u.endsWith('/ws') ? u : `${u}/ws`;
    return normalizeLoopbackHostForBrowser(withPath);
  }

  const base = getPublicApiBaseUrl();
  const ws = base.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:');
  return normalizeLoopbackHostForBrowser(`${ws}/ws`);
}
