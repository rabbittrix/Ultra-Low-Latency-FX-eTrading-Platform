/**
 * Portfolio/Positions page
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { useEffect, useState } from 'react';
import { ExposureSummary, InstrumentExposure, apiClient } from '@/lib/api';
import { TrendingUp, TrendingDown } from 'lucide-react';
import { getPublicApiBaseUrl } from '@/lib/public-config';

export default function PortfolioPage() {
  const [exposure, setExposure] = useState<ExposureSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadExposure();
    const interval = setInterval(loadExposure, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadExposure = async () => {
    try {
      const data = await apiClient.getExposureSummary();
      setExposure(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load exposure');
      setExposure(null);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <p className="text-gray-400">Loading portfolio...</p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <h1 className="text-2xl font-bold text-white">Portfolio</h1>
        <button
          type="button"
          onClick={loadExposure}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-white text-sm"
        >
          Refresh
        </button>
      </div>

      <p className="text-sm text-gray-500">
        API: <span className="font-mono text-gray-400">{getPublicApiBaseUrl()}</span>
      </p>

      {error && (
        <div className="p-4 bg-red-900/50 border border-red-700 rounded text-red-200">
          <p className="font-medium">Could not load exposure</p>
          <p className="text-sm mt-1 opacity-90">{error}</p>
          <p className="text-xs mt-2 text-red-300/80">
            Start the stack (<code className="text-red-200">docker compose up -d</code> in{' '}
            <code className="text-red-200">deploy/</code>) and ensure the gateway is on port 8080, or set{' '}
            <code className="text-red-200">NEXT_PUBLIC_API_URL</code>.
          </p>
        </div>
      )}

      {!error && exposure && (
        <>
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <div className="bg-gray-900 rounded border border-gray-800 p-4">
              <p className="text-sm text-gray-400">Total Instruments</p>
              <p className="text-2xl font-bold text-white">{exposure.total_instruments}</p>
            </div>
            <div className="bg-gray-900 rounded border border-gray-800 p-4">
              <p className="text-sm text-gray-400">Open Orders</p>
              <p className="text-2xl font-bold text-white">{exposure.total_open_orders}</p>
            </div>
            <div className="bg-gray-900 rounded border border-gray-800 p-4">
              <p className="text-sm text-gray-400">Total Exposure</p>
              <p className="text-2xl font-bold text-white">{exposure.total_exposure.toLocaleString()}</p>
            </div>
            <div className="bg-gray-900 rounded border border-gray-800 p-4">
              <p className="text-sm text-gray-400">Max Position Size</p>
              <p className="text-2xl font-bold text-white">
                {exposure.risk_limits.max_position_size.toLocaleString()}
              </p>
            </div>
          </div>

          <div className="bg-gray-900 rounded border border-gray-800 overflow-hidden">
            <div className="p-4 border-b border-gray-800">
              <h2 className="text-lg font-semibold text-white">Positions</h2>
              <p className="text-sm text-gray-500 mt-1">
                Rows appear when the risk engine has positions. With an empty book this table is empty.
              </p>
            </div>
            <table className="w-full">
              <thead className="bg-gray-800">
                <tr>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Instrument</th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Position</th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Abs Position</th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Utilization</th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Open Orders</th>
                </tr>
              </thead>
              <tbody>
                {exposure.instruments.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-gray-400">
                      No positions yet — place and fill orders on the Trading page to populate risk exposure.
                    </td>
                  </tr>
                ) : (
                  exposure.instruments.map((inst: InstrumentExposure) => (
                    <tr key={inst.instrument} className="border-t border-gray-800 hover:bg-gray-800/50">
                      <td className="px-4 py-3 text-sm text-white font-medium">{inst.instrument}</td>
                      <td className="px-4 py-3 text-sm">
                        <div className="flex items-center gap-2">
                          {inst.position > 0 ? (
                            <TrendingUp className="h-4 w-4 text-green-400" />
                          ) : inst.position < 0 ? (
                            <TrendingDown className="h-4 w-4 text-red-400" />
                          ) : null}
                          <span
                            className={
                              inst.position > 0
                                ? 'text-green-400'
                                : inst.position < 0
                                  ? 'text-red-400'
                                  : 'text-gray-400'
                            }
                          >
                            {inst.position.toLocaleString()}
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-300">{inst.position_abs.toLocaleString()}</td>
                      <td className="px-4 py-3 text-sm">
                        <div className="flex items-center gap-2">
                          <div className="flex-1 bg-gray-700 rounded-full h-2">
                            <div
                              className={`h-2 rounded-full ${
                                inst.position_utilization > 80
                                  ? 'bg-red-500'
                                  : inst.position_utilization > 50
                                    ? 'bg-yellow-500'
                                    : 'bg-green-500'
                              }`}
                              style={{
                                width: `${Math.min(inst.position_utilization, 100)}%`,
                              }}
                            />
                          </div>
                          <span className="text-gray-300 text-xs">
                            {inst.position_utilization.toFixed(1)}%
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-300">{inst.open_orders_count}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </>
      )}

      {!error && !exposure && (
        <div className="p-4 bg-gray-900 border border-gray-800 rounded text-gray-400">
          No exposure payload returned. Check the gateway and risk service.
        </div>
      )}
    </div>
  );
}
