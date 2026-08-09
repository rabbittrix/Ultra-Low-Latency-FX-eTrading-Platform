/**
 * Order Book page - Full screen order book view
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { useEffect, useState } from 'react';
import { WebSocketClient } from '@/lib/websocket';
import OrderBook from '@/components/orderbook/OrderBook';
import { getPublicApiBaseUrl } from '@/lib/public-config';
const DEFAULT_INSTRUMENT = 'EURUSD';

export default function OrderBookPage() {
  const [instrument, setInstrument] = useState(DEFAULT_INSTRUMENT);
  const [wsClient, setWsClient] = useState<WebSocketClient | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const client = new WebSocketClient(getPublicApiBaseUrl());

    client
      .connect()
      .then(() => {
        setIsConnected(true);
        setWsClient(client);
      })
      .catch(() => {
        setIsConnected(false);
      });

    return () => {
      client.disconnect();
    };
  }, []);

  const instruments = ['EURUSD', 'GBPUSD', 'USDJPY', 'AUDUSD'];

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">Order Book</h1>
        <div className="flex items-center gap-4">
          <div className="flex gap-2">
            {instruments.map((inst) => (
              <button
                key={inst}
                onClick={() => setInstrument(inst)}
                className={`rounded px-4 py-2 font-medium transition-colors ${
                  instrument === inst
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
                }`}
              >
                {inst}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-2">
            <span
              className={`h-2 w-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`}
            />
            <span className="text-sm text-gray-400">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
      </div>

      <OrderBook instrument={instrument} wsClient={wsClient} />
    </div>
  );
}
