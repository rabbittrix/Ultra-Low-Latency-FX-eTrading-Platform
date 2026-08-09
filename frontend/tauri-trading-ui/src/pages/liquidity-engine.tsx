/**
 * Global liquidity graph + AI execution dashboard
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import {
  ExecuteResponse,
  getLiquidityPlan,
  getLiquiditySnapshot,
  LiquidityGraphSnapshot,
  runExecutionPipeline,
} from '@/lib/api';
import { useCallback, useEffect, useState } from 'react';

export default function LiquidityEnginePage() {
  const [snapshot, setSnapshot] = useState<LiquidityGraphSnapshot | null>(null);
  const [planResult, setPlanResult] = useState<ExecuteResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const instrument = 'EURUSD';
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [qty, setQty] = useState(5_000_000);

  const refresh = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      const g = await getLiquiditySnapshot();
      setSnapshot(g);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const runPlanOnly = async () => {
    setError(null);
    setLoading(true);
    try {
      const p = await getLiquidityPlan(instrument, side, qty);
      if (p) {
        setPlanResult({
          client_id: '—',
          risk_ok: true,
          plan: p,
          fills: [],
          total_latency_us: 0,
          ai_notes: 'graph-only plan',
        });
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const runFull = async () => {
    setError(null);
    setLoading(true);
    try {
      const r = await runExecutionPipeline({
        instrument,
        side,
        quantity: qty,
        client_id: 'demo-client',
      });
      setPlanResult(r);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const edges = snapshot ? Object.values(snapshot.adj).flat() : [];

  return (
    <div className="space-y-8 p-8 text-gray-100">
      <div>
        <h1 className="text-2xl font-bold text-white">Liquidity graph engine</h1>
        <p className="mt-1 max-w-3xl text-gray-400">
          Live venue graph (internal book, LPs, simulated ECN), Dijkstra-weighted routing, ONNX /
          NumPy AI scoring, and parallel mock dispatch. Data flows through the API gateway (
          <code className="text-blue-300">/liquidity</code>,{' '}
          <code className="text-blue-300">/execution</code>
          ).
        </p>
      </div>

      <div className="flex flex-wrap items-end gap-3">
        <div>
          <label className="mb-1 block text-xs text-gray-500">Side</label>
          <select
            value={side}
            onChange={(e) => setSide(e.target.value as 'buy' | 'sell')}
            className="rounded border border-gray-700 bg-gray-900 px-3 py-2"
          >
            <option value="buy">buy</option>
            <option value="sell">sell</option>
          </select>
        </div>
        <div>
          <label className="mb-1 block text-xs text-gray-500">Quantity</label>
          <input
            type="number"
            value={qty}
            onChange={(e) => setQty(Number(e.target.value))}
            className="w-40 rounded border border-gray-700 bg-gray-900 px-3 py-2"
          />
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={loading}
          className="rounded border border-gray-600 bg-gray-800 px-4 py-2 hover:bg-gray-700"
        >
          Refresh graph
        </button>
        <button
          type="button"
          onClick={() => void runPlanOnly()}
          disabled={loading}
          className="rounded bg-indigo-700 px-4 py-2 hover:bg-indigo-600"
        >
          Plan (graph only)
        </button>
        <button
          type="button"
          onClick={() => void runFull()}
          disabled={loading}
          className="rounded bg-emerald-700 px-4 py-2 hover:bg-emerald-600"
        >
          Full pipeline (AI + dispatch)
        </button>
      </div>

      {error && (
        <div className="rounded border border-red-900 bg-red-950/50 px-4 py-3 text-red-200">
          {error}
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
        <section className="rounded-lg border border-gray-800 bg-gray-900/40 p-4">
          <h2 className="mb-3 text-lg font-semibold text-white">Venues &amp; edges</h2>
          <p className="mb-2 text-xs text-gray-500">Instrument: {snapshot?.instrument ?? '…'}</p>
          <div className="max-h-96 overflow-x-auto overflow-y-auto text-sm">
            <table className="w-full text-left">
              <thead className="border-b border-gray-800 text-gray-400">
                <tr>
                  <th className="py-2 pr-2">From</th>
                  <th className="py-2 pr-2">To</th>
                  <th className="py-2 pr-2">Price</th>
                  <th className="py-2 pr-2">Size</th>
                  <th className="py-2 pr-2">Lat µs</th>
                  <th className="py-2 pr-2">Fill</th>
                  <th className="py-2 pr-2">Tox</th>
                </tr>
              </thead>
              <tbody>
                {edges.map((e, i) => (
                  <tr key={`${e.from}-${e.to}-${i}`} className="border-b border-gray-800/80">
                    <td className="py-1 pr-2 font-mono text-xs">{e.from}</td>
                    <td className="py-1 pr-2 font-mono text-xs">{e.to}</td>
                    <td className="py-1 pr-2">{e.price.toFixed(5)}</td>
                    <td className="py-1 pr-2">{(e.available_size / 1e6).toFixed(2)}M</td>
                    <td className="py-1 pr-2">{e.latency_us.toFixed(0)}</td>
                    <td className="py-1 pr-2">{(e.fill_probability * 100).toFixed(1)}%</td>
                    <td className="py-1 pr-2">{(e.toxicity * 100).toFixed(0)}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="space-y-4 rounded-lg border border-gray-800 bg-gray-900/40 p-4">
          <h2 className="text-lg font-semibold text-white">Execution path &amp; AI</h2>
          {planResult ? (
            <>
              <div className="space-y-1 text-sm text-gray-300">
                <div className="font-mono text-xs text-gray-500">
                  Client: {planResult.client_id}
                </div>
                <div>
                  Risk:{' '}
                  <span className={planResult.risk_ok ? 'text-emerald-400' : 'text-red-400'}>
                    {planResult.risk_ok ? 'PASS' : 'FAIL'}
                  </span>
                </div>
                <div>
                  End-to-end latency:{' '}
                  <span className="text-amber-300">{planResult.total_latency_us} µs</span>
                </div>
                <div className="text-gray-400">{planResult.ai_notes}</div>
              </div>
              <div>
                <h3 className="mb-1 text-sm font-medium text-gray-400">Primary path</h3>
                <div className="flex flex-wrap gap-2">
                  {planResult.plan.primary_path.map((n) => (
                    <span
                      key={n}
                      className="rounded bg-blue-900/50 px-2 py-1 font-mono text-xs text-blue-100"
                    >
                      {n}
                    </span>
                  ))}
                </div>
              </div>
              <div>
                <h3 className="mb-1 text-sm font-medium text-gray-400">Allocations</h3>
                <ul className="space-y-1 text-sm">
                  {planResult.plan.allocations.map((a) => (
                    <li key={a.venue_id} className="flex justify-between gap-4">
                      <span className="font-mono text-blue-200">{a.venue_id}</span>
                      <span>
                        {(a.quantity / 1e6).toFixed(2)}M @ {a.expected_price.toFixed(5)}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
              {planResult.fills.length > 0 && (
                <div>
                  <h3 className="mb-1 text-sm font-medium text-gray-400">
                    Parallel fill latency heatmap
                  </h3>
                  <div className="grid grid-cols-2 gap-2">
                    {planResult.fills.map((f) => {
                      const heat = Math.min(100, f.latency_us / 2);
                      return (
                        <div
                          key={f.venue_id}
                          className="rounded border border-gray-800 p-2 text-xs"
                          style={{
                            backgroundColor: `rgba(248, 113, 113, ${heat / 100})`,
                          }}
                        >
                          <div className="font-mono text-white">{f.venue_id}</div>
                          <div className="text-gray-900">
                            {(f.quantity / 1e6).toFixed(2)}M — {f.latency_us} µs
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </>
          ) : (
            <p className="text-sm text-gray-500">Run a plan to see routing and AI notes.</p>
          )}
        </section>
      </div>
    </div>
  );
}
