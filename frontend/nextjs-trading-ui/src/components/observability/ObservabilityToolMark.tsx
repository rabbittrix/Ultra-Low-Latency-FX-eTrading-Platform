/**
 * Brand-colored marks for observability cards (literal Tailwind classes so JIT keeps them).
 */

import type { ObservabilityToolId } from '@/lib/observability-links';

type Props = { id: ObservabilityToolId; className?: string };

export function ObservabilityToolMark({ id, className = '' }: Props) {
  const box =
    id === 'grafana'
      ? 'bg-orange-600'
      : id === 'prometheus'
        ? 'bg-red-600'
        : id === 'jaeger'
          ? 'bg-blue-600'
          : 'bg-yellow-500';

  const iconClass =
    id === 'kibana' ? 'h-7 w-7 text-gray-900' : 'h-7 w-7 text-white';

  return (
    <div
      className={`flex h-12 w-12 shrink-0 items-center justify-center rounded-lg shadow-inner ${box} ${className}`}
      aria-hidden
    >
      {id === 'grafana' && <GrafanaGlyph className={iconClass} />}
      {id === 'prometheus' && <PrometheusGlyph className={iconClass} />}
      {id === 'jaeger' && <JaegerGlyph className={iconClass} />}
      {id === 'kibana' && <KibanaGlyph className={iconClass} />}
    </div>
  );
}

/** Abstract flame / lens (orange Grafana-style stack). */
function GrafanaGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="Grafana">
      <path d="M14.3 2.2c-2.8 1.6-4.8 4.5-5.1 7.9-.1 1.1.1 2.2.4 3.2-1.1-.4-2-.9-2.7-1.6C4.7 9.6 4.2 6.5 5.6 4 5.9 3.5 6.4 3.1 7 3c.5-.1 1 0 1.4.3.8.6 1.7 1 2.7 1.1.9-1.1 2-1.9 3.2-2.2z" />
      <path d="M19.5 11.2c-.4 3.1-2.5 5.7-5.4 6.7-.9.3-1.8.5-2.8.5-3.6 0-6.8-2.4-7.8-5.9 1.1 2.6 3.6 4.4 6.5 4.4 1.2 0 2.3-.3 3.3-.8 2.3-1.2 3.9-3.5 4.2-6.2.1-.6-.2-1.2-.8-1.4-.1 0-.2-.1-.3-.1-.5 0-.9.3-1.1.8z" />
    </svg>
  );
}

function PrometheusGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="Prometheus">
      <path d="M12 2L4 6v12l8 4 8-4V6l-8-4zm0 2.2l5.5 2.75L12 10.7 6.5 7.95 12 5.2zM6 9.4l5 2.5v6.05l-5-2.5V9.4zm7 8.55v-6.05l5-2.5v6.05l-5 2.5z" />
    </svg>
  );
}

function JaegerGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="Jaeger">
      <path d="M4 6h16v2H4V6zm0 5h10v2H4v-2zm0 5h16v2H4v-2zM4 4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2H4z" />
    </svg>
  );
}

function KibanaGlyph({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="Kibana">
      <path d="M4 4h4v16H4V4zm6 0l8 8-8 8V4z" />
    </svg>
  );
}
