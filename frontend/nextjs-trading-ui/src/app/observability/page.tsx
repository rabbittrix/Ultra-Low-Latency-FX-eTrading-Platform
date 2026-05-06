/**
 * Observability dashboards page
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { ExternalLink } from 'lucide-react';
import { getObservabilityTools } from '@/lib/observability-links';
import { ObservabilityToolMark } from '@/components/observability/ObservabilityToolMark';

export default function ObservabilityPage() {
  const dashboards = getObservabilityTools();

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-white mb-2">Observability Dashboards</h1>
      <p className="text-sm text-gray-400 mb-6 max-w-3xl">
        Each card opens the tool with a <span className="text-gray-300">live window</span> (last 15m–1h)
        and <span className="text-gray-300">auto-refresh</span> where supported. Override bases with{' '}
        <code className="text-xs text-gray-500">NEXT_PUBLIC_GRAFANA_URL</code>,{' '}
        <code className="text-xs text-gray-500">NEXT_PUBLIC_PROMETHEUS_URL</code>,{' '}
        <code className="text-xs text-gray-500">NEXT_PUBLIC_JAEGER_URL</code>,{' '}
        <code className="text-xs text-gray-500">NEXT_PUBLIC_KIBANA_URL</code>.
      </p>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {dashboards.map((dashboard) => (
          <a
            key={dashboard.id}
            href={dashboard.href}
            target="_blank"
            rel="noopener noreferrer"
            className="bg-gray-900 rounded border border-gray-800 p-6 hover:border-gray-600 transition-colors block"
          >
            <div className="flex items-center justify-between mb-4">
              <ObservabilityToolMark id={dashboard.id} />
              <ExternalLink className="h-5 w-5 text-gray-400 shrink-0" aria-hidden />
            </div>
            <h3 className="text-lg font-semibold text-white mb-2">{dashboard.name}</h3>
            <p className="text-sm text-gray-400 mb-2">{dashboard.description}</p>
            <p className="text-xs text-gray-500 font-mono break-all">{dashboard.displayBase}</p>
          </a>
        ))}
      </div>

      <div className="mt-8 bg-gray-900 rounded border border-gray-800 p-6">
        <h2 className="text-lg font-semibold text-white mb-4">Service Metrics</h2>
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          <div>
            <p className="text-sm text-gray-400">Market Data Service</p>
            <p className="text-2xl font-bold text-white">Port 8081</p>
            <p className="text-xs text-gray-500">Metrics: 9091</p>
          </div>
          <div>
            <p className="text-sm text-gray-400">Pricing Service</p>
            <p className="text-2xl font-bold text-white">Port 8082</p>
            <p className="text-xs text-gray-500">Metrics: 9092</p>
          </div>
          <div>
            <p className="text-sm text-gray-400">Matching Engine</p>
            <p className="text-2xl font-bold text-white">Port 8083</p>
            <p className="text-xs text-gray-500">Metrics: 9093</p>
          </div>
          <div>
            <p className="text-sm text-gray-400">Risk Service</p>
            <p className="text-2xl font-bold text-white">Port 8084</p>
            <p className="text-xs text-gray-500">Metrics: 9094</p>
          </div>
        </div>
      </div>
    </div>
  );
}
