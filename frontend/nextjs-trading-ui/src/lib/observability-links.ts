/**
 * Deep links into the local observability stack (Docker Compose defaults).
 * URLs include time ranges and refresh so tools open in a “live” monitoring context.
 */

const trimSlash = (s: string) => s.replace(/\/+$/, '');

export type ObservabilityToolId = 'grafana' | 'prometheus' | 'jaeger' | 'kibana';

export type ObservabilityTool = {
  id: ObservabilityToolId;
  name: string;
  description: string;
  /** Human-readable base (what we show under the card) */
  displayBase: string;
  /** Full URL opened in a new tab */
  href: string;
};

function envUrl(key: string, fallback: string): string {
  if (typeof process.env[key] === 'string' && process.env[key]!.length > 0) {
    return trimSlash(process.env[key]!);
  }
  return trimSlash(fallback);
}

export function getObservabilityTools(): ObservabilityTool[] {
  const grafana = envUrl('NEXT_PUBLIC_GRAFANA_URL', 'http://localhost:3001');
  const prometheus = envUrl('NEXT_PUBLIC_PROMETHEUS_URL', 'http://localhost:9099');
  const jaeger = envUrl('NEXT_PUBLIC_JAEGER_URL', 'http://localhost:16686');
  const kibana = envUrl('NEXT_PUBLIC_KIBANA_URL', 'http://localhost:5601');

  // Grafana: provisioned FX overview (uid in deploy/grafana/dashboard-definitions)
  const grafanaDashboard = new URL(
    '/d/fx-overview/fx-trading-platform-overview',
    `${grafana}/`
  );
  grafanaDashboard.searchParams.set('orgId', '1');
  grafanaDashboard.searchParams.set('from', 'now-15m');
  grafanaDashboard.searchParams.set('to', 'now');
  grafanaDashboard.searchParams.set('refresh', '5s');
  grafanaDashboard.searchParams.set('timezone', 'browser');

  // Prometheus: Graph tab with gateway request rate + 15m window
  const promQ = 'sum(rate(gateway_requests_total[5m]))';
  const prometheusGraph = new URL('/graph', `${prometheus}/`);
  prometheusGraph.searchParams.set('g0.expr', promQ);
  prometheusGraph.searchParams.set('g0.tab', '0');
  prometheusGraph.searchParams.set('g0.range_input', '15m');
  prometheusGraph.searchParams.set('g0.resolution', '15s');
  prometheusGraph.searchParams.set('g0.stacked', '0');

  // Jaeger: search UI, last hour, all services (traces appear when instrumentation is enabled)
  const jaegerSearch = new URL('/search', `${jaeger}/`);
  jaegerSearch.searchParams.set('lookback', '1h');
  jaegerSearch.searchParams.set('limit', '100');

  // Kibana: Discover with 15m window + 5s refresh (Rison in hash; index pattern may be required on first use)
  const kibanaDiscoverHref = `${kibana}/app/discover#/?_g=(filters:!(),refreshInterval:(pause:!f,value:5000),time:(from:now-15m,to:now))`;

  return [
    {
      id: 'grafana',
      name: 'Grafana',
      description:
        'FX Trading overview dashboard — service health, gateway traffic, matching, market data, risk, WebSockets (auto-refresh 5s).',
      displayBase: grafana,
      href: grafanaDashboard.toString(),
    },
    {
      id: 'prometheus',
      name: 'Prometheus',
      description:
        'Graph view: sum(rate(gateway_requests_total[5m])) over the last 15 minutes.',
      displayBase: prometheus,
      href: prometheusGraph.toString(),
    },
    {
      id: 'jaeger',
      name: 'Jaeger',
      description: 'Trace search — last 1 hour (all services). Open a trace for full waterfall.',
      displayBase: jaeger,
      href: jaegerSearch.toString(),
    },
    {
      id: 'kibana',
      name: 'Kibana',
      description:
        'Discover — last 15 minutes with 5s refresh. Ensure logs / index pattern exist in your stack.',
      displayBase: kibana,
      href: kibanaDiscoverHref,
    },
  ];
}
