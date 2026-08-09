/**
 * Real-time order book component
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { useEffect, useState } from 'react';
import { WebSocketClient, GatewayMessage } from '@/lib/websocket';

interface OrderBookEntry {
  price: number;
  quantity: number;
  total: number;
}

interface OrderBookProps {
  instrument: string;
  wsClient: WebSocketClient | null;
}

export default function OrderBook({ instrument, wsClient }: OrderBookProps) {
  const [bids, setBids] = useState<OrderBookEntry[]>([]);
  const [asks, setAsks] = useState<OrderBookEntry[]>([]);

  useEffect(() => {
    if (!wsClient) return;

    const handleMarketData = (message: GatewayMessage) => {
      if (message.type === 'MarketData' && message.instrument === instrument) {
        // Simulate order book levels from market data
        // In production, this would come from a dedicated order book feed
        const bidLevels: OrderBookEntry[] = [];
        const askLevels: OrderBookEntry[] = [];

        for (let i = 0; i < 10; i++) {
          bidLevels.push({
            price: message.bid - i * 0.0001,
            quantity: Math.floor(message.bid_size * (1 - i * 0.1)),
            total: 0,
          });

          askLevels.push({
            price: message.ask + i * 0.0001,
            quantity: Math.floor(message.ask_size * (1 - i * 0.1)),
            total: 0,
          });
        }

        // Calculate cumulative totals
        let bidTotal = 0;
        let askTotal = 0;
        bidLevels.forEach((entry) => {
          bidTotal += entry.quantity;
          entry.total = bidTotal;
        });
        askLevels.forEach((entry) => {
          askTotal += entry.quantity;
          entry.total = askTotal;
        });

        setBids(bidLevels.reverse());
        setAsks(askLevels);
      }
    };

    wsClient.subscribe('MarketData', handleMarketData);

    return () => {
      wsClient.unsubscribe('MarketData', handleMarketData);
    };
  }, [wsClient, instrument]);

  return (
    <div className="rounded border border-gray-800 bg-gray-900">
      <div className="border-b border-gray-800 p-4">
        <h2 className="text-lg font-semibold text-white">Order Book - {instrument}</h2>
      </div>
      <div className="grid grid-cols-2">
        {/* Asks (Sell side) */}
        <div>
          <div className="border-b border-gray-800 bg-red-900/20 p-2">
            <p className="text-sm font-semibold text-red-400">Asks (Sell)</p>
          </div>
          <div className="max-h-96 overflow-y-auto">
            {asks.map((ask, index) => (
              <div
                key={index}
                className="grid grid-cols-3 gap-2 border-b border-gray-800 p-2 hover:bg-gray-800/50"
              >
                <span className="text-right text-red-400">{ask.price.toFixed(4)}</span>
                <span className="text-center text-gray-300">{ask.quantity.toLocaleString()}</span>
                <span className="text-left text-gray-400">{ask.total.toLocaleString()}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Bids (Buy side) */}
        <div>
          <div className="border-b border-gray-800 bg-green-900/20 p-2">
            <p className="text-sm font-semibold text-green-400">Bids (Buy)</p>
          </div>
          <div className="max-h-96 overflow-y-auto">
            {bids.map((bid, index) => (
              <div
                key={index}
                className="grid grid-cols-3 gap-2 border-b border-gray-800 p-2 hover:bg-gray-800/50"
              >
                <span className="text-right text-green-400">{bid.price.toFixed(4)}</span>
                <span className="text-center text-gray-300">{bid.quantity.toLocaleString()}</span>
                <span className="text-left text-gray-400">{bid.total.toLocaleString()}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
