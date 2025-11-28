/**
 * Executions/Trades history page
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { useEffect, useState } from 'react';
import { TradeResponse, apiClient } from '@/lib/api';

export default function ExecutionsPage() {
  const [trades, setTrades] = useState<TradeResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTrades();
    const interval = setInterval(loadTrades, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const loadTrades = async () => {
    try {
      const data = await apiClient.getTrades();
      setTrades(data.sort((a, b) => b.timestamp_ns - a.timestamp_ns));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load trades');
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
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-white">Executions</h1>
        <button
          onClick={loadTrades}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white"
        >
          Refresh
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-900/50 border border-red-700 rounded text-red-200 mb-4">
          {error}
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
                  No trades found
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

