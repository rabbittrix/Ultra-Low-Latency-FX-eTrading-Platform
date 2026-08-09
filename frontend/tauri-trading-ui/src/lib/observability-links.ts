/**
 * Deep-link helpers kept for env documentation; Observability UI is fully in-app.
 */

const trimSlash = (s: string) => s.replace(/\/+$/, '');

function envUrl(key: keyof ImportMetaEnv, fallback: string): string {
  const value = import.meta.env[key];
  if (typeof value === 'string' && value.length > 0) {
    return trimSlash(value);
  }
  return trimSlash(fallback);
}

/** Local Prometheus base (charts query this via Tauri or Vite proxy). */
export function getPrometheusUrl(): string {
  return envUrl('VITE_PROMETHEUS_URL', 'http://127.0.0.1:9099');
}

/** SMC advisory API base. */
export function getSmcApiUrl(): string {
  return envUrl('VITE_SMC_API_URL', 'http://127.0.0.1:8094');
}
