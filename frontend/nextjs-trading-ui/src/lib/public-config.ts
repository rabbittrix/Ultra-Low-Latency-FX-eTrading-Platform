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
  const raw = process.env.NEXT_PUBLIC_API_URL?.trim() || 'http://127.0.0.1:8080';
  const base = raw.replace(/\/+$/, '');
  return normalizeLoopbackHostForBrowser(base);
}

/**
 * Full WebSocket URL for the gateway stream (`/ws`).
 * Derived from `NEXT_PUBLIC_API_URL`, or set `NEXT_PUBLIC_WS_URL` for a different host/port.
 * (Next.js rewrites do not proxy WebSocket upgrades — e.g. Turbopack — so the browser talks to the gateway directly.)
 */
export function getGatewayWebSocketUrl(): string {
  const explicit = process.env.NEXT_PUBLIC_WS_URL?.trim();
  if (explicit) {
    const u = explicit.replace(/\/+$/, '');
    const withPath = u.endsWith('/ws') ? u : `${u}/ws`;
    return normalizeLoopbackHostForBrowser(withPath);
  }

  const base = getPublicApiBaseUrl();
  const ws = base.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:');
  return normalizeLoopbackHostForBrowser(`${ws}/ws`);
}
