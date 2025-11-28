/**
 * Trading screen - Main trading interface
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import { useEffect, useState } from 'react';
import { WebSocketClient } from '@/lib/websocket';
import OrderForm from '@/components/trading/OrderForm';
import MarketData from '@/components/trading/MarketData';
import OrderBook from '@/components/orderbook/OrderBook';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const DEFAULT_INSTRUMENT = 'EURUSD';

export default function TradingScreen() {
  const [instrument, setInstrument] = useState(DEFAULT_INSTRUMENT);
  const [wsClient, setWsClient] = useState<WebSocketClient | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const client = new WebSocketClient(API_URL);
    
    client.connect()
      .then(() => {
        setIsConnected(true);
        setWsClient(client);
      })
      .catch((error) => {
        console.error('Failed to connect WebSocket:', error);
      });

    return () => {
      client.disconnect();
    };
  }, []);

  const instruments = ['EURUSD', 'GBPUSD', 'USDJPY', 'AUDUSD'];

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">Trading Screen</h1>
        <div className="flex items-center gap-2">
          <span className={`h-2 w-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className="text-sm text-gray-400">
            {isConnected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Instrument selector */}
      <div className="flex gap-2">
        {instruments.map((inst) => (
          <button
            key={inst}
            onClick={() => setInstrument(inst)}
            className={`px-4 py-2 rounded font-medium transition-colors ${
              instrument === inst
                ? 'bg-blue-600 text-white'
                : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
            }`}
          >
            {inst}
          </button>
        ))}
      </div>

      {/* Market Data */}
      <MarketData instrument={instrument} wsClient={wsClient} />

      <div className="grid grid-cols-3 gap-6">
        {/* Order Form */}
        <div className="col-span-1">
          <div className="bg-gray-900 rounded border border-gray-800 p-6">
            <h2 className="text-lg font-semibold text-white mb-4">Place Order</h2>
            <OrderForm instrument={instrument} />
          </div>
        </div>

        {/* Order Book */}
        <div className="col-span-2">
          <OrderBook instrument={instrument} wsClient={wsClient} />
        </div>
      </div>
    </div>
  );
}
