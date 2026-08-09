/**
 * Lightweight SVG sparkline for observability series (no chart library).
 */

import type { ChartPoint } from '@/lib/observability';

type Props = {
  points: ChartPoint[];
  className?: string;
  stroke?: string;
};

export function MetricSparkline({
  points,
  className = '',
  stroke = '#34d399',
}: Props) {
  const w = 320;
  const h = 96;
  const pad = 4;

  if (points.length === 0) {
    return (
      <div
        className={`flex h-24 items-center justify-center text-xs text-gray-500 ${className}`}
      >
        No samples
      </div>
    );
  }

  const xs = points.map((p) => p.t);
  const ys = points.map((p) => p.v);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);
  const spanX = Math.max(maxX - minX, 1);
  const spanY = Math.max(maxY - minY, 1e-9);

  const coords = points.map((p) => {
    const x = pad + ((p.t - minX) / spanX) * (w - pad * 2);
    const y = h - pad - ((p.v - minY) / spanY) * (h - pad * 2);
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });

  const last = points[points.length - 1]?.v;
  const lastLabel =
    last == null
      ? '—'
      : Number.isFinite(last)
        ? last >= 100
          ? last.toFixed(0)
          : last.toFixed(3)
        : '—';

  return (
    <div className={`relative ${className}`}>
      <svg viewBox={`0 0 ${w} ${h}`} className="h-24 w-full" role="img" aria-label="metric sparkline">
        <polyline
          fill="none"
          stroke={stroke}
          strokeWidth="2"
          strokeLinejoin="round"
          strokeLinecap="round"
          points={coords.join(' ')}
        />
      </svg>
      <p className="absolute right-1 top-1 font-mono text-xs text-gray-300">{lastLabel}</p>
    </div>
  );
}
