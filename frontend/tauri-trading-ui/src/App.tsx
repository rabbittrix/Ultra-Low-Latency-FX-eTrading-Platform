/**
 * Application routes for the Tauri trading UI.
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import { Navigate, Route, Routes } from 'react-router-dom';
import MainLayout from '@/components/layout/MainLayout';
import TradingPage from '@/pages/Trading';
import LiquidityEnginePage from '@/pages/liquidity-engine';
import OrderBookPage from '@/pages/orderbook';
import ExecutionsPage from '@/pages/executions';
import PortfolioPage from '@/pages/portfolio';
import AdminPage from '@/pages/admin';
import ObservabilityPage from '@/pages/observability';
import SmcAdvisoryPage from '@/pages/smc-advisory';

export default function App() {
  return (
    <MainLayout>
      <Routes>
        <Route path="/" element={<TradingPage />} />
        <Route path="/liquidity-engine" element={<LiquidityEnginePage />} />
        <Route path="/smc-advisory" element={<SmcAdvisoryPage />} />
        <Route path="/orderbook" element={<OrderBookPage />} />
        <Route path="/executions" element={<ExecutionsPage />} />
        <Route path="/portfolio" element={<PortfolioPage />} />
        <Route path="/admin" element={<AdminPage />} />
        <Route path="/observability" element={<ObservabilityPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </MainLayout>
  );
}
