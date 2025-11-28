/**
 * API client for FX eTrading Platform
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

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
  private baseUrl: string;

  constructor() {
    this.baseUrl = API_URL;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`API error: ${error}`);
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

