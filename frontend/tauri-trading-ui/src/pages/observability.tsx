/**
 * In-app Observability — direct scrape via Tauri (or Vite proxy). No Prometheus UI.
 */

import { MetricSparkline } from '@/components/observability/MetricSparkline';
import { collectObservability, type ChartSeries, type ServiceProbe } from '@/lib/observability';
import { getSmcApiPublicBase, getSmcHealth, runSmcAnalyze, type SmcAnalyzeResponse } from '@/lib/api';
import { Activity, RefreshCw } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';

const POLL_MS = 5_000;

export default function ObservabilityPage() {
  const [probes, setProbes] = useState<ServiceProbe[]>([]);
  const [charts, setCharts] = useState<ChartSeries[]>([]);
  const [source, setSource] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<string | null>(null);
  const [smcHealth, setSmcHealth] = useState<string | null>(null);
  const [smcPreview, setSmcPreview] = useState<SmcAnalyzeResponse | null>(null);
  const [smcBusy, setSmcBusy] = useState(false);
  const [smcError, setSmcError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const snap = await collectObservability();
      setProbes(snap.probes);
      setCharts(snap.charts);
      setSource(snap.source);
      setError(null);
      setUpdatedAt(new Date().toLocaleTimeString());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }

    try {
      const h = await getSmcHealth();
      setSmcHealth(`${h.service}: ${h.status}`);
      setSmcError(null);
    } catch (e) {
      setSmcHealth(null);
      setSmcError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = setInterval(() => void refresh(), POLL_MS);
    return () => clearInterval(id);
  }, [refresh]);

  const runSmcPreview = async () => {
    setSmcBusy(true);
    setSmcError(null);
    try {
      const r = await runSmcAnalyze(200);
      setSmcPreview(r);
    } catch (e) {
      setSmcPreview(null);
      setSmcError(e instanceof Error ? e.message : String(e));
    } finally {
      setSmcBusy(false);
    }
  };

  const expected = useMemo(() => probes.filter((p) => p.expected), [probes]);
  const optional = useMemo(() => probes.filter((p) => !p.expected), [probes]);
  const upExpected = expected.filter((p) => p.ok).length;

  const metricCharts = charts.filter((c) => !c.id.startsWith('lat-'));
  const latencyCharts = charts.filter((c) => c.id.startsWith('lat-'));

  return (
    <div className="space-y-8 p-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="mb-1 flex items-center gap-2 text-2xl font-bold text-white">
            <Activity className="h-7 w-7 text-emerald-400" aria-hidden />
            Observability Dashboards
          </h1>
          <p className="max-w-3xl text-sm text-gray-400">
            Metrics collected <span className="text-gray-200">directly in the app</span> from each
            service <code className="text-xs text-gray-500">/health</code> and{' '}
            <code className="text-xs text-gray-500">/metrics</code> — no Prometheus or Grafana
            required. Source: <span className="font-mono text-gray-300">{source || '…'}</span>. Poll{' '}
            {POLL_MS / 1000}s.
          </p>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          className="inline-flex items-center gap-2 rounded border border-gray-700 px-3 py-2 text-sm text-gray-200 hover:border-gray-500"
        >
          <RefreshCw className="h-4 w-4" aria-hidden />
          Refresh
          {updatedAt ? <span className="text-xs text-gray-500">{updatedAt}</span> : null}
        </button>
      </div>

      {error ? (
        <p className="rounded border border-amber-800 bg-amber-950/40 p-3 text-sm text-amber-100">
          Collect error: {error}
        </p>
      ) : null}

      <section>
        <div className="mb-3 flex items-baseline justify-between gap-2">
          <h2 className="text-lg font-semibold text-white">Service health (dev stack)</h2>
          <p className="text-xs text-gray-500">
            {upExpected}/{expected.length || '—'} up
          </p>
        </div>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {expected.map((p) => (
            <ProbeCard key={p.id} probe={p} />
          ))}
        </div>
        {optional.length > 0 ? (
          <>
            <h3 className="mb-2 mt-6 text-sm font-medium text-gray-400">
              Optional (not started by default <code className="text-xs">dev:stack</code>)
            </h3>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {optional.map((p) => (
                <ProbeCard key={p.id} probe={p} muted />
              ))}
            </div>
          </>
        ) : null}
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold text-white">Platform metrics</h2>
        <p className="mb-4 text-xs text-gray-500">
          Rates need two samples (~10s). Flat zero usually means the counter has not moved yet —
          submit an order or hit the gateway to generate traffic.
        </p>
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          {metricCharts.map((c) => (
            <ChartCard key={c.id} chart={c} stroke="#34d399" />
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold text-white">Health latency</h2>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {latencyCharts.map((c) => (
            <ChartCard key={c.id} chart={c} stroke="#38bdf8" />
          ))}
        </div>
      </section>

      <section className="rounded border border-gray-800 bg-gray-900/60 p-5">
        <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-lg font-semibold text-white">fx-smc Advisory</h2>
            <p className="text-sm text-gray-400">
              Research signals stay in-app ({getSmcApiPublicBase()}). Informational only — not
              investment advice; no returns promised.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              disabled={smcBusy}
              onClick={() => void runSmcPreview()}
              className="rounded bg-emerald-800 px-3 py-2 text-sm text-white hover:bg-emerald-700 disabled:opacity-50"
            >
              {smcBusy ? 'Analyzing…' : 'Quick analyze (200 ticks)'}
            </button>
            <Link
              to="/smc-advisory"
              className="rounded border border-gray-600 px-3 py-2 text-sm text-gray-100 hover:border-gray-400"
            >
              Open full SMC page
            </Link>
          </div>
        </div>
        {smcHealth ? (
          <p className="mb-2 font-mono text-sm text-emerald-400">{smcHealth}</p>
        ) : (
          <p className="mb-2 text-sm text-amber-200">
            SMC API unreachable — start with <code className="text-xs">npm run dev:stack</code>.
          </p>
        )}
        {smcError ? (
          <p className="mb-2 rounded border border-red-900 bg-red-950/40 p-2 text-sm text-red-200">
            {smcError}
          </p>
        ) : null}
        {smcPreview ? (
          <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
            <MiniStat label="Ticks" value={String(smcPreview.tick_count)} />
            <MiniStat label="Pools" value={String(smcPreview.pool_count)} />
            <MiniStat
              label="Sweeps"
              value={String(smcPreview.sweep_total ?? smcPreview.sweeps.length)}
            />
            <MiniStat
              label="Plans"
              value={String(smcPreview.plan_total ?? smcPreview.plans.length)}
            />
            <MiniStat label="Regime" value={smcPreview.regime.label} />
            <MiniStat
              label="Window"
              value={smcPreview.window_color ?? String(smcPreview.window.score)}
            />
            <MiniStat label="Conf" value={smcPreview.conf_signal ?? '—'} />
            <MiniStat
              label="Suitable"
              value={smcPreview.suitability.suitable ? 'pass' : 'blocked'}
            />
          </div>
        ) : (
          <p className="text-xs text-gray-500">
            Run a quick analyze here or open the full SMC Advisory page for plans and invalidation.
          </p>
        )}
      </section>
    </div>
  );
}

function ProbeCard({ probe: p, muted }: { probe: ServiceProbe; muted?: boolean }) {
  return (
    <div
      className={`rounded border p-4 ${
        p.ok
          ? 'border-emerald-900/70 bg-emerald-950/20'
          : muted
            ? 'border-gray-800 bg-gray-950/40'
            : 'border-red-900/70 bg-red-950/20'
      }`}
    >
      <div className="flex items-center justify-between gap-2">
        <p className="font-medium text-white">{p.name}</p>
        <span
          className={`rounded px-2 py-0.5 text-xs font-semibold ${
            p.ok
              ? 'bg-emerald-900 text-emerald-100'
              : muted
                ? 'bg-gray-800 text-gray-300'
                : 'bg-red-900 text-red-100'
          }`}
        >
          {p.ok ? 'UP' : muted ? 'OFF' : 'DOWN'}
        </span>
      </div>
      <p className="mt-1 break-all font-mono text-[11px] text-gray-500">{p.url}</p>
      <p className="mt-2 font-mono text-xs text-gray-400">
        {p.status != null ? `HTTP ${p.status}` : '—'} · {p.latencyMs} ms
      </p>
      {p.error && !muted ? <p className="mt-1 text-xs text-red-300">{p.error}</p> : null}
      {p.error && muted ? (
        <p className="mt-1 text-xs text-gray-500">Not running (optional service)</p>
      ) : null}
    </div>
  );
}

function ChartCard({ chart: c, stroke }: { chart: ChartSeries; stroke: string }) {
  return (
    <div className="rounded border border-gray-800 bg-gray-900/80 p-4">
      <div className="mb-2 flex items-baseline justify-between gap-2">
        <h3 className="text-sm font-semibold text-white">{c.title}</h3>
        <span className="font-mono text-[10px] text-gray-500">{c.unit || 'gauge'}</span>
      </div>
      {c.points.length === 0 && c.note ? (
        <p className="text-xs text-gray-500">{c.note}</p>
      ) : (
        <MetricSparkline points={c.points} stroke={stroke} />
      )}
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded border border-gray-800 bg-gray-950/50 p-3">
      <p className="text-[10px] uppercase tracking-wide text-gray-500">{label}</p>
      <p className="mt-1 truncate text-sm font-semibold text-white">{value}</p>
    </div>
  );
}
