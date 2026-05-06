/**
 * Live gateway reachability (REST /health + hint for WebSocket).
 */

'use client';

import { fetchGatewayHealth, type GatewayHealthResult } from '@/lib/api';
import { getGatewayWebSocketUrl, getPublicApiBaseUrl } from '@/lib/public-config';
import { useEffect, useState } from 'react';

export default function BackendStatusPanel() {
  const [health, setHealth] = useState<GatewayHealthResult | null>(null);

  useEffect(() => {
    void fetchGatewayHealth().then(setHealth);
    const id = setInterval(() => void fetchGatewayHealth().then(setHealth), 10_000);
    return () => clearInterval(id);
  }, []);

  const base = getPublicApiBaseUrl();
  const ws = getGatewayWebSocketUrl();

  if (health === null) {
    return (
      <div className="text-xs text-gray-500">
        Checking gateway at <span className="font-mono text-gray-400">{base}</span>…
      </div>
    );
  }

  if (!health.reachable) {
    return (
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
        <span className="rounded border border-amber-700/80 bg-amber-950/50 px-2 py-1 text-amber-100">
          <span className="font-semibold text-amber-50">Gateway</span> unreachable at{' '}
          <span className="font-mono">{base}</span> — {health.message}. Run{' '}
          <code className="rounded bg-black/30 px-1">npm run dev:stack</code> or{' '}
          <code className="rounded bg-black/30 px-1">cargo run --bin gateway-service</code>.
        </span>
        <span className="text-gray-500">
          WebSocket: <span className="font-mono text-gray-400">{ws}</span>
        </span>
      </div>
    );
  }

  const status = health.health.status != null ? String(health.health.status) : 'ok';
  const service = health.health.service != null ? String(health.health.service) : '';

  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
      <span className="rounded border border-emerald-800/70 bg-emerald-950/40 px-2 py-1 text-emerald-100">
        <span className="font-semibold text-emerald-50">Gateway</span> {status}
        {service ? <span className="text-emerald-200/80"> — {service}</span> : null}
        <span className="text-emerald-200/70"> · </span>
        <span className="font-mono text-emerald-100/90">{base}</span>
      </span>
      <span className="text-gray-500">
        WS <span className="font-mono text-gray-400">{ws}</span>
      </span>
    </div>
  );
}
