/**
 * SMC / advisory research dashboard (fx-smc-*)
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { getSmcHealth, runSmcAnalyze, SmcAnalyzeResponse } from '@/lib/api';
import { useCallback, useEffect, useState } from 'react';

export default function SmcAdvisoryPage() {
  const [health, setHealth] = useState<string | null>(null);
  const [result, setResult] = useState<SmcAnalyzeResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [tickCount, setTickCount] = useState(400);

  const ping = useCallback(async () => {
    try {
      const h = await getSmcHealth();
      setHealth(`${h.service}: ${h.status}`);
      setError(null);
    } catch (e) {
      setHealth(null);
      setError(
        e instanceof Error
          ? `${e.message} — is fx-smc-advisory-api on :8094? (npm run dev:stack starts it)`
          : String(e),
      );
    }
  }, []);

  useEffect(() => {
    void ping();
  }, [ping]);

  const analyze = async () => {
    setLoading(true);
    setError(null);
    try {
      const r = await runSmcAnalyze(tickCount);
      setResult(r);
    } catch (e) {
      setResult(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-8 p-8 text-gray-100">
      <div>
        <h1 className="text-2xl font-bold text-white">fx-smc Advisory</h1>
        <p className="mt-1 max-w-3xl text-gray-400">
          Research path: structure → liquidity pools → sweeps → trade plans. Outputs are{' '}
          <span className="text-amber-200">informational only</span> — not investment advice and not
          a promise of returns. Always define invalidation and size risk before acting.
        </p>
      </div>

      <div className="flex flex-wrap items-end gap-3">
        <div>
          <label className="mb-1 block text-xs text-gray-500">Synth tick count</label>
          <input
            type="number"
            min={32}
            max={2500}
            value={tickCount}
            onChange={(e) => setTickCount(Number(e.target.value))}
            className="w-40 rounded border border-gray-700 bg-gray-900 px-3 py-2"
          />
          <p className="mt-1 text-[10px] text-gray-500">32–2500 (debug builds: keep ≤800)</p>
        </div>
        <button
          type="button"
          onClick={() => void analyze()}
          disabled={loading}
          className="rounded bg-emerald-700 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-600 disabled:opacity-50"
        >
          {loading ? 'Analyzing…' : 'Run analyze'}
        </button>
        <button
          type="button"
          onClick={() => void ping()}
          disabled={loading}
          className="rounded border border-gray-700 px-4 py-2 text-sm text-gray-200 hover:border-gray-500 disabled:opacity-50"
        >
          Ping health
        </button>
      </div>

      {health && <p className="font-mono text-sm text-emerald-400">{health}</p>}
      {error && (
        <p className="rounded border border-red-900 bg-red-950/40 p-3 text-sm text-red-200">
          {error}
        </p>
      )}

      {result && (
        <div className="space-y-6">
          <div className="rounded border border-amber-900/60 bg-amber-950/30 p-4 text-sm text-amber-100">
            {result.disclaimer}
          </div>

          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <Stat label="Ticks" value={String(result.tick_count)} />
            <Stat label="Pools" value={String(result.pool_count)} />
            <Stat
              label="Sweeps"
              value={`${result.sweeps.length}${result.sweep_total != null ? ` / ${result.sweep_total}` : ''}`}
            />
            <Stat
              label="Plans"
              value={`${result.plans.length}${result.plan_total != null ? ` / ${result.plan_total}` : ''}`}
            />
            <Stat label="Regime" value={result.regime.label} />
            <Stat label="Window score" value={String(result.window.score)} />
            <Stat
              label="Suitability"
              value={result.suitability.suitable ? 'pass' : 'blocked'}
            />
            {result.window_color != null && (
              <div className="rounded border border-gray-800 bg-gray-900 p-4">
                <p className="text-xs text-gray-500">Entry window</p>
                <div className="mt-1 flex flex-wrap items-center gap-2">
                  <WindowColorBadge color={result.window_color} />
                  <span className="text-sm text-gray-300">
                    {result.window_side ?? '—'} · raw {result.window_raw ?? '—'}
                  </span>
                </div>
              </div>
            )}
          </div>

          {result.facts && result.facts.length > 0 && (
            <div className="rounded border border-gray-800 bg-gray-950/60 p-4">
              <p className="mb-2 text-xs uppercase tracking-wide text-gray-500">Facts</p>
              <ul className="max-h-48 space-y-1 overflow-y-auto font-mono text-xs text-gray-400">
                {result.facts.map((f) => (
                  <li key={f}>{f}</li>
                ))}
              </ul>
            </div>
          )}

          {result.plans.length > 0 && (
            <div className="overflow-x-auto rounded border border-gray-800">
              <table className="min-w-full text-left text-sm">
                <thead className="bg-gray-900 text-gray-400">
                  <tr>
                    <th className="px-3 py-2">Id</th>
                    <th className="px-3 py-2">Side</th>
                    <th className="px-3 py-2">Entry</th>
                    <th className="px-3 py-2">Stop</th>
                    <th className="px-3 py-2">Target</th>
                    <th className="px-3 py-2">Conf</th>
                    <th className="px-3 py-2">Invalidation</th>
                  </tr>
                </thead>
                <tbody>
                  {result.plans.map((p) => (
                    <tr key={p.id} className="border-t border-gray-800">
                      <td className="px-3 py-2 font-mono">{p.id}</td>
                      <td className="px-3 py-2">{p.side}</td>
                      <td className="px-3 py-2">{p.entry_ticks}</td>
                      <td className="px-3 py-2">{p.stop_ticks}</td>
                      <td className="px-3 py-2">{p.target_ticks}</td>
                      <td className="px-3 py-2">{p.confluence}</td>
                      <td className="max-w-md px-3 py-2 text-xs text-gray-400">{p.invalidation}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded border border-gray-800 bg-gray-900 p-4">
      <p className="text-xs text-gray-500">{label}</p>
      <p className="mt-1 text-lg font-semibold text-white">{value}</p>
    </div>
  );
}

function WindowColorBadge({ color }: { color: string }) {
  const tone =
    color === 'Green'
      ? 'bg-emerald-900/80 text-emerald-200 border-emerald-700'
      : color === 'Yellow'
        ? 'bg-amber-900/80 text-amber-100 border-amber-700'
        : 'bg-red-950/80 text-red-200 border-red-800';
  return (
    <span className={`rounded border px-2 py-0.5 text-sm font-semibold ${tone}`}>{color}</span>
  );
}
