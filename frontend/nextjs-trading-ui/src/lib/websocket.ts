/**
 * WebSocket client for real-time market data and trading updates
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

export type GatewayMessage =
  | { type: 'MarketData'; instrument: string; bid: number; ask: number; bid_size: number; ask_size: number; spread: number; mid_price: number; timestamp: number }
  | { type: 'Pricing'; instrument: string; bid_price: number; ask_price: number; mid_price: number; spread: number }
  | { type: 'Trade'; trade_id: string; buy_order_id: string; sell_order_id: string; instrument: string; quantity: number; price: number; timestamp_ns: number }
  | { type: 'Exposure'; instrument: string; position: number; position_abs: number; position_utilization: number; open_orders_count: number }
  | { type: 'Error'; message: string };

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;
  private listeners: Map<string, Set<(data: GatewayMessage) => void>> = new Map();
  private isConnecting = false;

  constructor(private url: string) {}

  connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return Promise.resolve();
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        const wsUrl = this.url.replace('http://', 'ws://').replace('https://', 'wss://');
        this.ws = new WebSocket(`${wsUrl}/ws`);

        this.ws.onopen = () => {
          console.log('WebSocket connected');
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: GatewayMessage = JSON.parse(event.data);
            this.notifyListeners(message);
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        };

        this.ws.onerror = (error) => {
          console.error('WebSocket error:', error);
          this.isConnecting = false;
          reject(error);
        };

        this.ws.onclose = () => {
          console.log('WebSocket disconnected');
          this.isConnecting = false;
          this.ws = null;
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
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    
    setTimeout(() => {
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      this.connect().catch(console.error);
    }, delay);
  }

  subscribe(messageType: string, callback: (data: GatewayMessage) => void) {
    if (!this.listeners.has(messageType)) {
      this.listeners.set(messageType, new Set());
    }
    this.listeners.get(messageType)!.add(callback);

    // Send subscription message
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
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
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.listeners.clear();
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

