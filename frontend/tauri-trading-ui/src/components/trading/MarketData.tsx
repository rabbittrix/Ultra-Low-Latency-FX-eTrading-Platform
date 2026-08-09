/**
 * Real-time market data display component
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { useEffect, useState } from 'react';
import { WebSocketClient, GatewayMessage } from '@/lib/websocket';

interface MarketDataProps {
  instrument: string;
  wsClient: WebSocketClient | null;
}

export default function MarketData({ instrument, wsClient }: MarketDataProps) {
  const [marketData, setMarketData] = useState<{
    bid: number;
    ask: number;
    spread: number;
    mid_price: number;
    timestamp: number;
  } | null>(null);

  useEffect(() => {
    if (!wsClient) return;

    const handleMarketData = (message: GatewayMessage) => {
      if (message.type === 'MarketData' && message.instrument === instrument) {
        setMarketData({
          bid: message.bid,
          ask: message.ask,
          spread: message.spread,
          mid_price: message.mid_price,
          timestamp: message.timestamp,
        });
      }
    };

    wsClient.subscribe('MarketData', handleMarketData);

    return () => {
      wsClient.unsubscribe('MarketData', handleMarketData);
    };
  }, [wsClient, instrument]);

  if (!marketData) {
    return (
      <div className="rounded border border-gray-800 bg-gray-900 p-4">
        <p className="text-gray-400">Waiting for market data...</p>
      </div>
    );
  }

  return (
    <div className="rounded border border-gray-800 bg-gray-900 p-4">
      <div className="grid grid-cols-4 gap-4">
        <div>
          <p className="text-sm text-gray-400">Bid</p>
          <p className="text-xl font-bold text-red-400">{marketData.bid.toFixed(4)}</p>
        </div>
        <div>
          <p className="text-sm text-gray-400">Ask</p>
          <p className="text-xl font-bold text-green-400">{marketData.ask.toFixed(4)}</p>
        </div>
        <div>
          <p className="text-sm text-gray-400">Spread</p>
          <p className="text-xl font-bold text-gray-300">{marketData.spread}</p>
        </div>
        <div>
          <p className="text-sm text-gray-400">Mid Price</p>
          <p className="text-xl font-bold text-white">{marketData.mid_price.toFixed(4)}</p>
        </div>
      </div>
    </div>
  );
}
