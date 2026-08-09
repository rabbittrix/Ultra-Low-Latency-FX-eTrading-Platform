/**
 * Main layout component with sidebar
 *
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import BackendStatusPanel from '@/components/system/BackendStatusPanel';
import Sidebar from './Sidebar';

export default function MainLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen bg-gray-950">
      <Sidebar />
      <main className="flex flex-1 flex-col overflow-hidden">
        <div className="shrink-0 border-b border-gray-800 bg-gray-900/90 px-4 py-2">
          <BackendStatusPanel />
        </div>
        <div className="flex-1 overflow-auto">{children}</div>
      </main>
    </div>
  );
}
