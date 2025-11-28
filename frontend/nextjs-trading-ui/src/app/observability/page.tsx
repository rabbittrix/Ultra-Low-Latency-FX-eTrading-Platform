/**
 * Observability dashboards page
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { ExternalLink } from 'lucide-react';
import Link from 'next/link';

export default function ObservabilityPage() {
  const dashboards = [
    {
      name: 'Grafana',
      description: 'Metrics and monitoring dashboards',
      url: 'http://localhost:3001',
      color: 'bg-orange-600',
    },
    {
      name: 'Prometheus',
      description: 'Time-series database and metrics',
      url: 'http://localhost:9099',
      color: 'bg-red-600',
    },
    {
      name: 'Jaeger',
      description: 'Distributed tracing',
      url: 'http://localhost:16686',
      color: 'bg-blue-600',
    },
    {
      name: 'Kibana',
      description: 'Log analysis and visualization',
      url: 'http://localhost:5601',
      color: 'bg-yellow-600',
    },
  ];

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-white mb-6">Observability Dashboards</h1>

      <div className="grid grid-cols-2 gap-4">
        {dashboards.map((dashboard) => (
          <Link
            key={dashboard.name}
            href={dashboard.url}
            target="_blank"
            rel="noopener noreferrer"
            className="bg-gray-900 rounded border border-gray-800 p-6 hover:border-gray-700 transition-colors"
          >
            <div className="flex items-center justify-between mb-4">
              <div className={`${dashboard.color} p-3 rounded`}>
                <ExternalLink className="h-6 w-6 text-white" />
              </div>
              <ExternalLink className="h-5 w-5 text-gray-400" />
            </div>
            <h3 className="text-lg font-semibold text-white mb-2">{dashboard.name}</h3>
            <p className="text-sm text-gray-400">{dashboard.description}</p>
            <p className="text-xs text-gray-500 mt-2">{dashboard.url}</p>
          </Link>
        ))}
      </div>

      <div className="mt-8 bg-gray-900 rounded border border-gray-800 p-6">
        <h2 className="text-lg font-semibold text-white mb-4">Service Metrics</h2>
        <div className="grid grid-cols-4 gap-4">
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

