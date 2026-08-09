/**
 * Admin dashboard page
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { useEffect, useState } from 'react';
import { AuditEvent, apiClient } from '@/lib/api';
import { Activity, AlertCircle, CheckCircle, XCircle } from 'lucide-react';

export default function AdminPage() {
  const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadAuditEvents();
    const interval = setInterval(loadAuditEvents, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadAuditEvents = async () => {
    try {
      const data = await apiClient.getAuditEvents();
      setAuditEvents(data.sort((a, b) => b.timestamp_ns - a.timestamp_ns).slice(0, 100));
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load audit events');
    } finally {
      setLoading(false);
    }
  };

  const getEventIcon = (eventType: string) => {
    switch (eventType) {
      case 'Filled':
        return <CheckCircle className="h-4 w-4 text-green-400" />;
      case 'PartiallyFilled':
        return <Activity className="h-4 w-4 text-yellow-400" />;
      case 'Rejected':
        return <XCircle className="h-4 w-4 text-red-400" />;
      case 'Cancelled':
        return <AlertCircle className="h-4 w-4 text-gray-400" />;
      default:
        return <Activity className="h-4 w-4 text-blue-400" />;
    }
  };

  const getEventColor = (eventType: string) => {
    switch (eventType) {
      case 'Filled':
        return 'text-green-400';
      case 'PartiallyFilled':
        return 'text-yellow-400';
      case 'Rejected':
        return 'text-red-400';
      case 'Cancelled':
        return 'text-gray-400';
      default:
        return 'text-blue-400';
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <p className="text-gray-400">Loading admin data...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <h1 className="text-2xl font-bold text-white">Admin Dashboard</h1>

      {error && (
        <div className="rounded border border-red-700 bg-red-900/50 p-4 text-red-200">{error}</div>
      )}

      {/* Audit Events */}
      <div className="overflow-hidden rounded border border-gray-800 bg-gray-900">
        <div className="flex items-center justify-between border-b border-gray-800 p-4">
          <h2 className="text-lg font-semibold text-white">Recent Audit Events</h2>
          <button
            onClick={loadAuditEvents}
            className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700"
          >
            Refresh
          </button>
        </div>
        <div className="max-h-96 overflow-y-auto">
          <table className="w-full">
            <thead className="sticky top-0 bg-gray-800">
              <tr>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Type</th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">
                  Order ID
                </th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">
                  Instrument
                </th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Side</th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Type</th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">
                  Quantity
                </th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Price</th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">
                  Timestamp
                </th>
                <th className="px-4 py-3 text-left text-sm font-semibold text-gray-300">Message</th>
              </tr>
            </thead>
            <tbody>
              {auditEvents.length === 0 ? (
                <tr>
                  <td colSpan={9} className="px-4 py-8 text-center text-gray-400">
                    No audit events found
                  </td>
                </tr>
              ) : (
                auditEvents.map((event, index) => (
                  <tr key={index} className="border-t border-gray-800 hover:bg-gray-800/50">
                    <td className="px-4 py-3 text-sm">
                      <div className="flex items-center gap-2">
                        {getEventIcon(event.event_type)}
                        <span className={getEventColor(event.event_type)}>{event.event_type}</span>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      {event.order_id.slice(0, 8)}...
                    </td>
                    <td className="px-4 py-3 text-sm font-medium text-white">{event.instrument}</td>
                    <td className="px-4 py-3 text-sm text-gray-300">{event.side}</td>
                    <td className="px-4 py-3 text-sm text-gray-300">{event.order_type}</td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      {event.quantity.toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      {event.price ? event.price.toFixed(4) : 'Market'}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-400">
                      {new Date(event.timestamp_ns / 1_000_000).toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-400">{event.message || '-'}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
