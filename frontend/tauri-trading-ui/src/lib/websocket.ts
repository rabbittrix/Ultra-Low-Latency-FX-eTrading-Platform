/**
 * WebSocket client for real-time market data and trading updates
 *
 * Gateway serializes `type` in snake_case (`market_data`, …). We normalize to
 * PascalCase for listeners to match UI code.
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { getGatewayWebSocketUrl } from '@/lib/public-config';

/** Limits duplicate warnings when several components mount or Strict Mode re-runs effects. */
let lastGatewayUnreachableWarnAt = 0;
const GATEWAY_UNREACHABLE_WARN_THROTTLE_MS = 15_000;

function warnGatewayUnreachableThrottled(message: string) {
  const now = Date.now();
  if (now - lastGatewayUnreachableWarnAt < GATEWAY_UNREACHABLE_WARN_THROTTLE_MS) {
    return;
  }
  lastGatewayUnreachableWarnAt = now;
  console.warn(message);
}

export type GatewayMessage =
  | {
      type: 'MarketData';
      instrument: string;
      bid: number;
      ask: number;
      bid_size: number;
      ask_size: number;
      spread: number;
      mid_price: number;
      timestamp: number;
    }
  | {
      type: 'Pricing';
      instrument: string;
      bid_price: number;
      ask_price: number;
      mid_price: number;
      spread: number;
    }
  | {
      type: 'Trade';
      trade_id: string;
      buy_order_id: string;
      sell_order_id: string;
      instrument: string;
      quantity: number;
      price: number;
      timestamp_ns: number;
    }
  | {
      type: 'Exposure';
      instrument: string;
      position: number;
      position_abs: number;
      position_utilization: number;
      open_orders_count: number;
    }
  | { type: 'Error'; message: string };

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;
  private listeners: Map<string, Set<(data: GatewayMessage) => void>> = new Map();
  private isConnecting = false;
  /** True only after a successful `onopen` (avoids reconnect storm when gateway is down). */
  private openedSuccessfully = false;
  /** When true, `onclose` must not schedule reconnect (e.g. `disconnect()` or unmount). */
  private intentionalClose = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  /**
   * @param _baseHttpUrl — unused; URL comes from `getGatewayWebSocketUrl()`.
   */
  constructor(_baseHttpUrl?: string) {}

  private wsUrl(): string {
    return getGatewayWebSocketUrl();
  }

  connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return Promise.resolve();
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        const url = this.wsUrl();
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
          console.log('WebSocket connected:', url);
          this.isConnecting = false;
          this.openedSuccessfully = true;
          this.reconnectAttempts = 0;
          this.flushSubscriptionMessages();
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const raw = JSON.parse(event.data) as Record<string, unknown>;
            const message = normalizeGatewayMessage(raw);
            if (message) {
              this.notifyListeners(message);
            }
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        };

        this.ws.onerror = () => {
          const failUrl = this.wsUrl();
          if (!this.openedSuccessfully) {
            warnGatewayUnreachableThrottled(
              `[WebSocket] no gateway at ${failUrl} — run \`npm run dev:stack\` from frontend/tauri-trading-ui, or \`cargo run --bin gateway-service\` (8080), or set VITE_API_URL.`,
            );
          }
          this.isConnecting = false;
          reject(new Error(`WebSocket failed to connect: ${failUrl}`));
        };

        this.ws.onclose = () => {
          if (this.openedSuccessfully) {
            console.log('WebSocket disconnected');
          }
          this.isConnecting = false;
          this.ws = null;

          if (this.intentionalClose) {
            this.intentionalClose = false;
            return;
          }
          if (!this.openedSuccessfully) {
            return;
          }
          this.attemptReconnect();
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  private attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.warn('[WebSocket] max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      console.log(`WebSocket reconnect ${this.reconnectAttempts}/${this.maxReconnectAttempts}…`);
      void this.connect().catch(() => {
        /* initial error already logged; unreachable opens handled by onerror */
      });
    }, delay);
  }

  subscribe(messageType: string, callback: (data: GatewayMessage) => void) {
    if (!this.listeners.has(messageType)) {
      this.listeners.set(messageType, new Set());
    }
    this.listeners.get(messageType)!.add(callback);

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'subscribe', messageType }));
    }
  }

  /** Re-send subscribe frames after reconnect (one per distinct messageType). */
  private flushSubscriptionMessages() {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    for (const messageType of this.listeners.keys()) {
      this.ws.send(JSON.stringify({ type: 'subscribe', messageType }));
    }
  }

  unsubscribe(messageType: string, callback: (data: GatewayMessage) => void) {
    const listeners = this.listeners.get(messageType);
    if (listeners) {
      listeners.delete(callback);
      if (listeners.size === 0) {
        this.listeners.delete(messageType);
      }
    }
  }

  private notifyListeners(message: GatewayMessage) {
    const listeners = this.listeners.get(message.type);
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(message);
        } catch (error) {
          console.error('Error in WebSocket listener:', error);
        }
      });
    }
  }

  disconnect() {
    this.intentionalClose = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.openedSuccessfully = false;
    this.reconnectAttempts = 0;
    this.listeners.clear();
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

/** Map gateway snake_case `type` to PascalCase keys used by subscribers */
function normalizeGatewayMessage(raw: Record<string, unknown>): GatewayMessage | null {
  const t = raw.type;
  if (typeof t !== 'string') return null;

  const typeMap: Record<string, GatewayMessage['type']> = {
    market_data: 'MarketData',
    pricing: 'Pricing',
    trade: 'Trade',
    exposure: 'Exposure',
    error: 'Error',
  };

  const normalizedType = typeMap[t];
  if (!normalizedType) return null;

  return { ...raw, type: normalizedType } as GatewayMessage;
}
