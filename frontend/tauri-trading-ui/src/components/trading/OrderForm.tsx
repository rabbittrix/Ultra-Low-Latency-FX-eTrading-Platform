/**
 * Order placement form component
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { useState } from 'react';
import { SubmitOrderRequest } from '@/lib/api';
import { apiClient } from '@/lib/api';

interface OrderFormProps {
  instrument: string;
  onOrderSubmitted?: () => void;
}

export default function OrderForm({ instrument, onOrderSubmitted }: OrderFormProps) {
  const [side, setSide] = useState<'Buy' | 'Sell'>('Buy');
  const [orderType, setOrderType] = useState<'Market' | 'Limit' | 'Stop' | 'IoC' | 'FoK'>('Market');
  const [quantity, setQuantity] = useState<string>('');
  const [price, setPrice] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setIsSubmitting(true);

    try {
      const order: SubmitOrderRequest = {
        instrument,
        side,
        order_type: orderType,
        quantity: parseInt(quantity, 10),
        price: orderType !== 'Market' && price ? parseFloat(price) : undefined,
      };

      const response = await apiClient.submitOrder(order);

      if (response.success) {
        setSuccess(`Order ${response.order_id} submitted successfully`);
        setQuantity('');
        setPrice('');
        onOrderSubmitted?.();
      } else {
        setError(response.message);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit order');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <button
          type="button"
          onClick={() => setSide('Buy')}
          className={`rounded px-4 py-2 font-semibold transition-colors ${
            side === 'Buy'
              ? 'bg-green-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
        >
          Buy
        </button>
        <button
          type="button"
          onClick={() => setSide('Sell')}
          className={`rounded px-4 py-2 font-semibold transition-colors ${
            side === 'Sell'
              ? 'bg-red-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
        >
          Sell
        </button>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-300">Order Type</label>
        <select
          value={orderType}
          onChange={(e) => setOrderType(e.target.value as any)}
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-white"
        >
          <option value="Market">Market</option>
          <option value="Limit">Limit</option>
          <option value="Stop">Stop</option>
          <option value="IoC">IoC (Immediate or Cancel)</option>
          <option value="FoK">FoK (Fill or Kill)</option>
        </select>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-300">Quantity</label>
        <input
          type="number"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
          required
          min="1"
          className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-white"
          placeholder="Enter quantity"
        />
      </div>

      {orderType !== 'Market' && (
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-300">Price</label>
          <input
            type="number"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            required
            step="0.0001"
            className="w-full rounded border border-gray-700 bg-gray-800 px-3 py-2 text-white"
            placeholder="Enter price"
          />
        </div>
      )}

      {error && (
        <div className="rounded border border-red-700 bg-red-900/50 p-3 text-sm text-red-200">
          {error}
        </div>
      )}

      {success && (
        <div className="rounded border border-green-700 bg-green-900/50 p-3 text-sm text-green-200">
          {success}
        </div>
      )}

      <button
        type="submit"
        disabled={isSubmitting}
        className={`w-full rounded px-4 py-3 font-semibold transition-colors ${
          side === 'Buy'
            ? 'bg-green-600 text-white hover:bg-green-700'
            : 'bg-red-600 text-white hover:bg-red-700'
        } disabled:cursor-not-allowed disabled:opacity-50`}
      >
        {isSubmitting ? 'Submitting...' : `${side} ${orderType}`}
      </button>
    </form>
  );
}
