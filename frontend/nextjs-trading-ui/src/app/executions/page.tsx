/**
 * Executions/Trades history page
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { useEffect, useState } from 'react';
import { TradeResponse, apiClient, fetchGatewayHealth, type GatewayHealthResult } from '@/lib/api';
import { getPublicApiBaseUrl } from '@/lib/public-config';

export default function ExecutionsPage() {
  const [trades, setTrades] = useState<TradeResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [gatewayHealth, setGatewayHealth] = useState<GatewayHealthResult | null>(null);

  useEffect(() => {
    loadTrades();
    const interval = setInterval(loadTrades, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadTrades = async () => {
    const gh = await fetchGatewayHealth();
    setGatewayHealth(gh);
    try {
      const data = await apiClient.getTrades();
      setTrades(data.sort((a, b) => b.timestamp_ns - a.timestamp_ns));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load trades');
      setTrades([]);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <p className="text-gray-400">Loading trades...</p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex flex-wrap items-center justify-between gap-4 mb-6">
        <h1 className="text-2xl font-bold text-white">Executions</h1>
        <button
          type="button"
          onClick={loadTrades}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white text-sm"
        >
          Refresh
        </button>
      </div>

      <p className="text-sm text-gray-500 mb-4">
        API: <span className="font-mono text-gray-400">{getPublicApiBaseUrl()}</span>
      </p>

      {error && (
        <div className="p-4 bg-red-900/50 border border-red-700 rounded text-red-200 mb-4 space-y-2">
          <p className="font-medium">Could not load trades</p>
          <p className="text-sm opacity-90 font-mono break-all">{error}</p>
          {!gatewayHealth?.reachable && (
            <p className="text-xs text-red-200/90">
              The bar above should show gateway status. If the gateway is down, WebSocket and{' '}
              <code className="text-red-100">/matching/*</code> will fail — start{' '}
              <code className="text-red-100">gateway-service</code> on port 8080.
            </p>
          )}
          {gatewayHealth?.reachable && /\b404\b/.test(error) && (
            <p className="text-xs text-amber-200/95">
              Gateway <code className="text-amber-100">/health</code> works but{' '}
              <code className="text-amber-100">/matching/trades</code> returned 404. Rebuild and restart{' '}
              <code className="text-amber-100">gateway-service</code> (proxy uses <code className="text-amber-100">OriginalUri</code>
              ), and run <code className="text-amber-100">matching-engine-service</code> on{' '}
              <code className="text-amber-100">8083</code> (<code className="text-amber-100">npm run dev:stack</code>).
            </p>
          )}
          {gatewayHealth?.reachable && /\b(502|503|504)\b/.test(error) && (
            <p className="text-xs text-amber-200/95">
              Gateway is up but could not reach the matching engine. Start{' '}
              <code className="text-amber-100">matching-engine-service</code> on{' '}
              <code className="text-amber-100">127.0.0.1:8083</code> or fix{' '}
              <code className="text-amber-100">MATCHING_ENGINE_URL</code>.
            </p>
          )}
          <p className="text-xs text-red-300/80">
            From <code className="text-red-200">frontend/nextjs-trading-ui</code>, run{' '}
            <code className="text-red-200">npm run dev:stack</code> (UI + gateway + matching), or{' '}
            <code className="text-red-200">docker compose up -d</code> in <code className="text-red-200">deploy/</code>.
          </p>
        </div>
      )}

      {!error && trades.length === 0 && (
        <div className="mb-4 p-4 bg-gray-900 border border-gray-800 rounded text-gray-400 text-sm">
          No trades in the matching engine yet. Submit opposing buy/sell orders on the{' '}
          <span className="text-gray-300">Trading</span> page to create fills, then refresh.
        </div>
      )}

      <div className="bg-gray-900 rounded border border-gray-800 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-800">
            <tr>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Trade ID</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Instrument</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Quantity</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Price</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Buy Order</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Sell Order</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Timestamp</th>
            </tr>
          </thead>
          <tbody>
            {trades.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-gray-400">
                  {error ? '—' : 'No trades found'}
                </td>
              </tr>
            ) : (
              trades.map((trade) => (
                <tr key={trade.trade_id} className="border-t border-gray-800 hover:bg-gray-800/50">
                  <td className="px-4 py-3 text-sm text-gray-300">{trade.trade_id.slice(0, 8)}...</td>
                  <td className="px-4 py-3 text-sm text-white font-medium">{trade.instrument}</td>
                  <td className="px-4 py-3 text-sm text-gray-300">{trade.quantity.toLocaleString()}</td>
                  <td className="px-4 py-3 text-sm text-white font-semibold">{trade.price.toFixed(4)}</td>
                  <td className="px-4 py-3 text-sm text-green-400">{trade.buy_order_id.slice(0, 8)}...</td>
                  <td className="px-4 py-3 text-sm text-red-400">{trade.sell_order_id.slice(0, 8)}...</td>
                  <td className="px-4 py-3 text-sm text-gray-400">
                    {new Date(trade.timestamp_ns / 1_000_000).toLocaleString()}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
