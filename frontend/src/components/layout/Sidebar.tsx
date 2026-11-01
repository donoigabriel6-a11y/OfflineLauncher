// Sidebar navigation component
import { NavLink, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import {
  Home,
  Users,
  Package,
  FolderKanban,
  Settings,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react';
import { useStore } from '@/store/useStore';

const navigation = [
  { path: '/', label: 'Home', icon: Home },
  { path: '/accounts', label: 'Accounts', icon: Users },
  { path: '/versions', label: 'Versions', icon: Package },
  { path: '/instances', label: 'Instances', icon: FolderKanban },
  { path: '/settings', label: 'Settings', icon: Settings },
];

export function Sidebar() {
  const { sidebarOpen, toggleSidebar } = useStore();
  const location = useLocation();

  return (
    <aside
      className={cn(
        'fixed left-0 top-0 z-40 h-screen bg-surface-950/95 border-r border-surface-800 backdrop-blur-sm transition-all duration-300 flex flex-col',
        sidebarOpen ? 'w-64' : 'w-20'
      )}
    >
      {/* Logo / Brand */}
      <div className="flex items-center justify-between h-16 px-4 border-b border-surface-800">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-primary-600 flex items-center justify-center">
            <Package className="w-5 h-5 text-white" />
          </div>
          {sidebarOpen && (
            <span className="font-bold text-lg text-surface-100">OfflineLauncher</span>
          )}
        </div>
        <button
          onClick={toggleSidebar}
          className={cn(
            'p-1.5 rounded-lg text-surface-500 hover:text-surface-100 hover:bg-surface-800 transition-colors',
            sidebarOpen ? 'ml-auto' : 'mx-auto'
          )}
          aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {sidebarOpen ? (
            <ChevronLeft className="w-5 h-5" />
          ) : (
            <ChevronRight className="w-5 h-5" />
          )}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto scrollbar-thin" aria-label="Main navigation">
        {navigation.map((item) => {
          const isActive = location.pathname === item.path ||
            (item.path !== '/' && location.pathname.startsWith(item.path));
          const Icon = item.icon;

          return (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive: active }) => cn(
                'sidebar-item group',
                active && 'active'
              )}
              title={sidebarOpen ? undefined : item.label}
              aria-current={isActive ? 'page' : undefined}
            >
              <Icon className="w-5 h-5 flex-shrink-0" aria-hidden="true" />
              {sidebarOpen && <span className="font-medium">{item.label}</span>}
            </NavLink>
          );
        })}
      </nav>

      {/* Bottom section */}
      <div className="p-3 border-t border-surface-800">
        {sidebarOpen && (
          <div className="text-xs text-surface-500 text-center">
            v0.1.0
          </div>
        )}
      </div>
    </aside>
  );
}