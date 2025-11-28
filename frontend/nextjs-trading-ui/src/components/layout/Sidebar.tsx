/**
 * Sidebar navigation component
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { 
  LayoutDashboard, 
  TrendingUp, 
  BookOpen, 
  FileText, 
  Briefcase, 
  Settings,
  BarChart3
} from 'lucide-react';
import { clsx } from 'clsx';

const navigation = [
  { name: 'Trading', href: '/', icon: TrendingUp },
  { name: 'Order Book', href: '/orderbook', icon: BookOpen },
  { name: 'Executions', href: '/executions', icon: FileText },
  { name: 'Portfolio', href: '/portfolio', icon: Briefcase },
  { name: 'Admin', href: '/admin', icon: Settings },
  { name: 'Observability', href: '/observability', icon: BarChart3 },
];

export default function Sidebar() {
  const pathname = usePathname();

  return (
    <div className="flex h-screen w-64 flex-col bg-gray-900 border-r border-gray-800">
      <div className="flex h-16 items-center px-6 border-b border-gray-800">
        <h1 className="text-xl font-bold text-white">FX Trading</h1>
      </div>
      <nav className="flex-1 space-y-1 px-3 py-4">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.name}
              href={item.href}
              className={clsx(
                'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                isActive
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-800 hover:text-white'
              )}
            >
              <item.icon className="h-5 w-5" />
              {item.name}
            </Link>
          );
        })}
      </nav>
      <div className="border-t border-gray-800 p-4">
        <p className="text-xs text-gray-500">
          © 2024 Roberto de Souza
        </p>
      </div>
    </div>
  );
}

