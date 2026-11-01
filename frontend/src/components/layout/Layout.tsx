// Main layout component
import React from 'react';
import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { cn } from '@/lib/utils';
import { useStore } from '@/store/useStore';

export function Layout() {
  const { sidebarOpen } = useStore();

  return (
    <div className="min-h-screen bg-surface-950 flex">
      <Sidebar />
      <div className={cn('flex-1 flex flex-col transition-all duration-300', sidebarOpen ? 'ml-64' : 'ml-20')}>
        <Outlet />
      </div>
    </div>
  );
}

export function PageLayout({
  children,
  className = '',
  padding = 'md'
}: {
  children: React.ReactNode;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg';
}) {
  const paddings = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  };

  return (
    <main className={cn('flex-1 overflow-auto', paddings[padding], className)}>
      {children}
    </main>
  );
}