/**
 * In-app observability — prefers Tauri direct scrape; browser uses Vite proxies.
 */

import { invoke } from '@tauri-apps/api/core';

export type ServiceProbe = {
  id: string;
  name: string;
  url: string;
  ok: boolean;
  status?: number | null;
  latencyMs: number;
  error?: string | null;
  expected: boolean;
};

export type ChartPoint = { t: number; v: number };

export type ChartSeries = {
  id: string;
  title: string;
  unit: string;
  points: ChartPoint[];
  latest?: number | null;
  note?: string | null;
};

export type ObsSnapshot = {
  probes: ServiceProbe[];
  charts: ChartSeries[];
  collectedAtNs: number;
  source: string;
};

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

type RustProbe = {
  id: string;
  name: string;
  url: string;
  ok: boolean;
  status?: number | null;
  latencyMs: number;
  error?: string | null;
  expected: boolean;
};

type RustChart = {
  id: string;
  title: string;
  unit: string;
  points: { t: number; v: number }[];
  latest?: number | null;
  note?: string | null;
};

type RustSnap = {
  probes: RustProbe[];
  charts: RustChart[];
  collectedAtNs: number;
  source: string;
};

function mapSnap(s: RustSnap): ObsSnapshot {
  return {
    probes: s.probes.map((p) => ({
      id: p.id,
      name: p.name,
      url: p.url,
      ok: p.ok,
      status: p.status,
      latencyMs: p.latencyMs,
      error: p.error,
      expected: p.expected,
    })),
    charts: s.charts.map((c) => ({
      id: c.id,
      title: c.title,
      unit: c.unit,
      points: c.points.map((p) => ({ t: p.t, v: p.v })),
      latest: c.latest,
      note: c.note,
    })),
    collectedAtNs: s.collectedAtNs,
    source: s.source,
  };
}

/** Browser-side ring for rates when not in Tauri. */
const browserRings = new Map<
  string,
  { points: ChartPoint[]; lastRaw?: { t: number; v: number } }
>();
const RING_CAP = 180;

type BrowserTarget = {
  id: string;
  name: string;
  expected: boolean;
  healthPath: string;
  metricsPath?: string;
};

const BROWSER_TARGETS: BrowserTarget[] = [
  {
    id: 'gateway',
    name: 'Gateway',
    expected: true,
    healthPath: '/__obs/svc/gateway/health',
    metricsPath: '/__obs/svc/gateway/metrics',
  },
  {
    id: 'matching',
    name: 'Matching Engine',
    expected: true,
    healthPath: '/__obs/svc/matching/health',
    metricsPath: '/__obs/svc/matching/metrics',
  },
  {
    id: 'risk',
    name: 'Risk',
    expected: true,
    healthPath: '/__obs/svc/risk/health',
    metricsPath: '/__obs/svc/risk/metrics',
  },
  {
    id: 'liquidity',
    name: 'Liquidity Graph',
    expected: true,
    healthPath: '/__obs/svc/liquidity/health',
    metricsPath: '/__obs/svc/liquidity/metrics',
  },
  {
    id: 'execution',
    name: 'Execution Engine',
    expected: true,
    healthPath: '/__obs/svc/execution/health',
    metricsPath: '/__obs/svc/execution/metrics',
  },
  {
    id: 'smc',
    name: 'fx-smc Advisory',
    expected: true,
    healthPath: '/__obs/svc/smc/health',
  },
];

const BROWSER_CHARTS: {
  id: string;
  title: string;
  unit: string;
  metric: string;
  isCounter: boolean;
}[] = [
  {
    id: 'gateway-rps',
    title: 'Gateway requests (rate)',
    unit: '/s',
    metric: 'gateway_requests_total',
    isCounter: true,
  },
  {
    id: 'gateway-ws',
    title: 'Gateway WebSocket clients',
    unit: '',
    metric: 'gateway_active_websocket_clients',
    isCounter: false,
  },
  {
    id: 'matching-orders',
    title: 'Matching orders submitted (rate)',
    unit: '/s',
    metric: 'matching_engine_orders_submitted_total',
    isCounter: true,
  },
  {
    id: 'matching-trades',
    title: 'Matching trades executed (rate)',
    unit: '/s',
    metric: 'matching_engine_trades_executed_total',
    isCounter: true,
  },
  {
    id: 'risk-checks',
    title: 'Risk checks (rate)',
    unit: '/s',
    metric: 'risk_checks_total',
    isCounter: true,
  },
  {
    id: 'exec-success',
    title: 'Execution success (rate)',
    unit: '/s',
    metric: 'exec_success_total',
    isCounter: true,
  },
  {
    id: 'liq-recompute',
    title: 'Liquidity graph recomputes (rate)',
    unit: '/s',
    metric: 'liquidity_graph_recomputes_total',
    isCounter: true,
  },
];

function parseMetric(body: string, metric: string): number | null {
  for (const raw of body.split('\n')) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    if (
      !(
        line.startsWith(metric) &&
        (line[metric.length] === '{' ||
          line[metric.length] === ' ' ||
          line[metric.length] === '\t')
      )
    ) {
      continue;
    }
    const parts = line.split(/\s+/);
    const last = parts[parts.length - 1];
    const v = Number(last);
    if (Number.isFinite(v)) return v;
  }
  return null;
}

function pushRing(id: string, t: number, v: number) {
  let ring = browserRings.get(id);
  if (!ring) {
    ring = { points: [] };
    browserRings.set(id, ring);
  }
  ring.points.push({ t, v });
  while (ring.points.length > RING_CAP) ring.points.shift();
}

function pushRate(id: string, t: number, raw: number, isCounter: boolean) {
  let ring = browserRings.get(id);
  if (!ring) {
    ring = { points: [] };
    browserRings.set(id, ring);
  }
  if (isCounter) {
    if (ring.lastRaw) {
      const dt = Math.max(t - ring.lastRaw.t, 1);
      const dv = Math.max(raw - ring.lastRaw.v, 0);
      pushRing(id, t, dv / dt);
    }
    ring.lastRaw = { t, v: raw };
  } else {
    ring.lastRaw = { t, v: raw };
    pushRing(id, t, raw);
  }
}

async function collectBrowser(): Promise<ObsSnapshot> {
  const t = Math.floor(Date.now() / 1000);
  const probes: ServiceProbe[] = [];
  const scraped = new Map<string, number>();

  await Promise.all(
    BROWSER_TARGETS.map(async (target) => {
      const start = performance.now();
      try {
        const resp = await fetch(target.healthPath, { cache: 'no-store' });
        probes.push({
          id: target.id,
          name: target.name,
          url: target.healthPath,
          ok: resp.ok,
          status: resp.status,
          latencyMs: Math.round(performance.now() - start),
          error: resp.ok ? null : `HTTP ${resp.status}`,
          expected: target.expected,
        });
      } catch (e) {
        probes.push({
          id: target.id,
          name: target.name,
          url: target.healthPath,
          ok: false,
          status: null,
          latencyMs: Math.round(performance.now() - start),
          error: e instanceof Error ? e.message : String(e),
          expected: target.expected,
        });
      }

      if (target.metricsPath) {
        try {
          const resp = await fetch(target.metricsPath, { cache: 'no-store' });
          if (resp.ok) {
            const body = await resp.text();
            for (const c of BROWSER_CHARTS) {
              const v = parseMetric(body, c.metric);
              if (v != null) scraped.set(c.id, v);
            }
          }
        } catch {
          /* ignore */
        }
      }
    }),
  );

  const charts: ChartSeries[] = BROWSER_CHARTS.map((c) => {
    const raw = scraped.get(c.id);
    let note: string | null = null;
    if (raw != null) {
      pushRate(c.id, t, raw, c.isCounter);
    } else {
      note = 'metric not published by a reachable service yet';
    }
    const ring = browserRings.get(c.id);
    return {
      id: c.id,
      title: c.title,
      unit: c.unit,
      points: ring?.points ?? [],
      latest: ring?.points[ring.points.length - 1]?.v ?? null,
      note,
    };
  });

  for (const p of probes.filter((x) => x.expected)) {
    const id = `lat-${p.id}`;
    pushRing(id, t, p.latencyMs);
    const ring = browserRings.get(id);
    charts.push({
      id,
      title: `${p.name} health latency`,
      unit: 'ms',
      points: ring?.points ?? [],
      latest: p.latencyMs,
      note: null,
    });
  }

  return {
    probes,
    charts,
    collectedAtNs: Date.now() * 1_000_000,
    source: 'browser-proxy',
  };
}

export async function collectObservability(): Promise<ObsSnapshot> {
  if (isTauriRuntime()) {
    const raw = await invoke<RustSnap>('obs_collect');
    return mapSnap(raw);
  }
  return collectBrowser();
}
