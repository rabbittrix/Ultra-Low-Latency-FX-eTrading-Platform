/**
 * API client for FX eTrading Platform
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { getPublicApiBaseUrl } from '@/lib/public-config';

export interface SubmitOrderRequest {
  instrument: string;
  side: 'Buy' | 'Sell';
  order_type: 'Market' | 'Limit' | 'Stop' | 'IoC' | 'FoK';
  quantity: number;
  price?: number;
}

export interface SubmitOrderResponse {
  success: boolean;
  message: string;
  order_id: string;
  trades: TradeResponse[];
}

export interface TradeResponse {
  trade_id: string;
  buy_order_id: string;
  sell_order_id: string;
  instrument: string;
  quantity: number;
  price: number;
  timestamp_ns: number;
}

export interface CancelOrderRequest {
  order_id: string;
}

export interface CancelOrderResponse {
  success: boolean;
  message: string;
  order_id: string;
}

export interface PositionResponse {
  instrument: string;
  position: number;
}

export interface InstrumentExposure {
  instrument: string;
  position: number;
  position_abs: number;
  position_utilization: number;
  open_orders_count: number;
}

export interface ExposureSummary {
  total_instruments: number;
  total_open_orders: number;
  total_exposure: number;
  instruments: InstrumentExposure[];
  risk_limits: {
    max_position_size: number;
    max_order_size: number;
    max_daily_loss: number;
    max_open_orders: number;
  };
}

export interface AuditEvent {
  event_type: string;
  order_id: string;
  instrument: string;
  side: string;
  order_type: string;
  quantity: number;
  price?: number;
  timestamp_ns: number;
  message?: string;
}

class ApiClient {
  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const base = getPublicApiBaseUrl();
    const url = `${base}${endpoint}`;
    const method = (options.method ?? 'GET').toUpperCase();
    const headers = new Headers(options.headers as HeadersInit | undefined);
    if (method !== 'GET' && method !== 'HEAD' && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json');
    }
    let response: Response;
    try {
      response = await fetch(url, {
        ...options,
        method,
        headers,
      });
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e);
      throw new Error(
        `Cannot reach API at ${url} (${reason}). Start the gateway on ${base} — try \`npm run dev:stack\` from frontend/tauri-trading-ui, or \`docker compose up -d\` in deploy/.`,
      );
    }

    if (!response.ok) {
      const body = (await response.text()).trim() || '(empty body)';
      throw new Error(`HTTP ${response.status} ${response.statusText} on ${endpoint} — ${body}`);
    }

    return response.json();
  }

  async submitOrder(order: SubmitOrderRequest): Promise<SubmitOrderResponse> {
    return this.request<SubmitOrderResponse>('/matching/orders', {
      method: 'POST',
      body: JSON.stringify(order),
    });
  }

  async cancelOrder(orderId: string): Promise<CancelOrderResponse> {
    return this.request<CancelOrderResponse>('/matching/orders/cancel', {
      method: 'POST',
      body: JSON.stringify({ order_id: orderId }),
    });
  }

  async getTrades(): Promise<TradeResponse[]> {
    const response = await this.request<{ trades: TradeResponse[] }>('/matching/trades');
    return response.trades;
  }

  async getAuditEvents(): Promise<AuditEvent[]> {
    const response = await this.request<{ events: AuditEvent[] }>('/matching/audit');
    return response.events;
  }

  async getPosition(instrument: string): Promise<PositionResponse> {
    return this.request<PositionResponse>(`/risk/position/${instrument}`);
  }

  async getExposureSummary(): Promise<ExposureSummary> {
    return this.request<ExposureSummary>('/risk/exposure');
  }

  async getInstrumentExposure(instrument: string): Promise<InstrumentExposure> {
    return this.request<InstrumentExposure>(`/risk/exposure/${instrument}`);
  }

  async checkOrderRisk(order: {
    instrument: string;
    side: string;
    quantity: number;
    order_id: string;
  }): Promise<{ success: boolean; message: string }> {
    return this.request('/risk/check', {
      method: 'POST',
      body: JSON.stringify(order),
    });
  }
}

export const apiClient = new ApiClient();

export type GatewayHealthResult =
  { reachable: true; health: Record<string, unknown> } | { reachable: false; message: string };

/** Lightweight check — does not use ApiClient (no JSON Content-Type on GET). */
export async function fetchGatewayHealth(): Promise<GatewayHealthResult> {
  const base = getPublicApiBaseUrl();
  try {
    const r = await fetch(`${base}/health`);
    const text = await r.text();
    if (!r.ok) {
      return {
        reachable: false,
        message: `HTTP ${r.status} ${r.statusText}: ${text.trim() || '(empty body)'}`,
      };
    }
    try {
      return { reachable: true, health: JSON.parse(text) as Record<string, unknown> };
    } catch {
      return { reachable: true, health: { status: text, raw: true } };
    }
  } catch (e) {
    return { reachable: false, message: e instanceof Error ? e.message : String(e) };
  }
}

/** --- Global liquidity graph + AI execution (gateway proxies) --- */

export interface LiquidityNode {
  id: string;
  class: string;
  label: string;
}

export interface LiquidityEdge {
  from: string;
  to: string;
  price: number;
  available_size: number;
  latency_us: number;
  fill_probability: number;
  toxicity: number;
}

export interface LiquidityGraphSnapshot {
  instrument: string;
  nodes: Record<string, LiquidityNode>;
  adj: Record<string, LiquidityEdge[]>;
}

export interface VenueAllocation {
  venue_id: string;
  quantity: number;
  expected_price: number;
  hop: number;
}

export interface ExecutionPlan {
  instrument: string;
  side: string;
  total_quantity: number;
  allocations: VenueAllocation[];
  slice_strategy:
    { immediate?: null } | { time_weighted?: { slices: number; interval_ms: number } };
  expected_slippage_bps: number;
  primary_path: string[];
  path_cost: number;
}

export interface ExecuteResponse {
  client_id: string;
  risk_ok: boolean;
  plan: ExecutionPlan;
  fills: { venue_id: string; quantity: number; latency_us: number }[];
  total_latency_us: number;
  ai_notes: string;
}

export async function getLiquiditySnapshot(): Promise<LiquidityGraphSnapshot> {
  const url = `${getPublicApiBaseUrl()}/liquidity/v1/graph/snapshot`;
  const response = await fetch(url);
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

export async function getLiquidityPlan(
  instrument: string,
  side: string,
  quantity: number,
): Promise<ExecutionPlan | null> {
  const url = `${getPublicApiBaseUrl()}/liquidity/v1/plan`;
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ instrument, side, quantity }),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

export async function runExecutionPipeline(body: {
  instrument: string;
  side: string;
  quantity: number;
  client_id: string;
}): Promise<ExecuteResponse> {
  const url = `${getPublicApiBaseUrl()}/execution/v1/execute`;
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

/** --- SMC advisory research API (direct, not via gateway) --- */

function getSmcApiBaseUrl(): string {
  const raw = import.meta.env.VITE_SMC_API_URL;
  if (typeof raw === 'string' && raw.length > 0) {
    return raw.replace(/\/+$/, '');
  }
  return 'http://127.0.0.1:8094';
}

export function getSmcApiPublicBase(): string {
  return getSmcApiBaseUrl();
}

export interface SmcAnalyzeResponse {
  disclaimer: string;
  tick_count: number;
  pool_count: number;
  sweep_total?: number;
  plan_total?: number;
  structure_break_count?: number;
  fvg_count?: number;
  conf_signal?: string;
  sweeps: Array<{
    pool_id: string;
    side: string;
    pierce_idx: number;
    confirm_idx: number;
    displacement_ticks: number;
  }>;
  plans: Array<{
    id: string;
    side: string;
    entry_ticks: number;
    stop_ticks: number;
    target_ticks: number;
    risk_ticks: number;
    reward_ticks: number;
    confluence: number;
    invalidation: string;
  }>;
  regime: { label: string };
  window: { score: number; window_ticks: number };
  suitability: { suitable: boolean; reasons: string[] };
  window_color?: string;
  window_raw?: number;
  window_side?: string;
  facts?: string[];
}

export async function getSmcHealth(): Promise<{ status: string; service: string }> {
  const response = await fetch(`${getSmcApiBaseUrl()}/health`);
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

export async function runSmcAnalyze(tickCount = 400): Promise<SmcAnalyzeResponse> {
  const capped = Math.min(Math.max(tickCount, 32), 2500);
  const controller = new AbortController();
  const timer = window.setTimeout(() => controller.abort(), 90_000);
  try {
    const response = await fetch(`${getSmcApiBaseUrl()}/v1/analyze`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ tick_count: capped }),
      signal: controller.signal,
    });
    if (!response.ok) throw new Error(await response.text());
    return response.json();
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') {
      throw new Error('Analyze timed out after 90s — try fewer ticks (e.g. 400) or restart fx-smc-advisory-api');
    }
    throw e;
  } finally {
    window.clearTimeout(timer);
  }
}

export function getSmcDocsUrl(): string {
  return `${getSmcApiBaseUrl()}/docs`;
}
