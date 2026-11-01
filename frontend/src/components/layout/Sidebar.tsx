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
        'fixed left-0 top-0 z-40 flex h-screen flex-col border-r border-surface-800 bg-surface-950/95 backdrop-blur-sm transition-all duration-300',
        sidebarOpen ? 'w-64' : 'w-20'
      )}
    >
      {/* Logo / Brand */}
      <div className="flex h-16 items-center justify-between border-b border-surface-800 px-4">
        <div className="flex items-center gap-3">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary-600">
            <Package className="h-5 w-5 text-white" />
          </div>
          {sidebarOpen && (
            <span className="text-lg font-bold text-surface-100">OfflineLauncher</span>
          )}
        </div>
        <button
          onClick={toggleSidebar}
          className={cn(
            'rounded-lg p-1.5 text-surface-500 transition-colors hover:bg-surface-800 hover:text-surface-100',
            sidebarOpen ? 'ml-auto' : 'mx-auto'
          )}
          aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {sidebarOpen ? <ChevronLeft className="h-5 w-5" /> : <ChevronRight className="h-5 w-5" />}
        </button>
      </div>

      {/* Navigation */}
      <nav
        className="scrollbar-thin flex-1 space-y-1 overflow-y-auto px-3 py-4"
        aria-label="Main navigation"
      >
        {navigation.map((item) => {
          const isActive =
            location.pathname === item.path ||
            (item.path !== '/' && location.pathname.startsWith(item.path));
          const Icon = item.icon;

          return (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive: active }) => cn('sidebar-item group', active && 'active')}
              title={sidebarOpen ? undefined : item.label}
              aria-current={isActive ? 'page' : undefined}
            >
              <Icon className="h-5 w-5 flex-shrink-0" aria-hidden="true" />
              {sidebarOpen && <span className="font-medium">{item.label}</span>}
            </NavLink>
          );
        })}
      </nav>

      {/* Bottom section */}
      <div className="border-t border-surface-800 p-3">
        {sidebarOpen && <div className="text-center text-xs text-surface-500">v0.1.0</div>}
      </div>
    </aside>
  );
}
