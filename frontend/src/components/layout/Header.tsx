// Header component
import { useTheme } from '@/hooks/useTheme';
import { cn } from '@/lib/utils';
import { Sun, Moon, Monitor } from 'lucide-react';

interface HeaderProps {
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
}

export function Header({ title, subtitle, actions }: HeaderProps) {
  const { theme, setTheme } = useTheme();

  return (
    <header className="sticky top-0 z-30 h-16 border-b border-surface-800 bg-surface-950/80 backdrop-blur-sm">
      <div className="flex h-full items-center justify-between px-6">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-semibold text-surface-100">{title}</h1>
          {subtitle && <span className="text-sm text-surface-500">{subtitle}</span>}
        </div>

        <div className="flex items-center gap-3">
          {actions}

          {/* Theme Toggle */}
          <div className="flex items-center gap-1 rounded-lg border border-surface-700 bg-surface-800/50 p-1">
            {['light', 'dark', 'system'].map((t) => (
              <button
                key={t}
                onClick={() => setTheme(t as 'light' | 'dark' | 'system')}
                className={cn(
                  'rounded-md p-2 transition-all duration-200',
                  theme === t
                    ? 'bg-primary-600 text-white shadow-lg'
                    : 'text-surface-500 hover:bg-surface-700 hover:text-surface-100'
                )}
                aria-label={`Set theme to ${t}`}
                aria-pressed={theme === t}
              >
                {t === 'light' && <Sun className="h-4 w-4" />}
                {t === 'dark' && <Moon className="h-4 w-4" />}
                {t === 'system' && <Monitor className="h-4 w-4" />}
              </button>
            ))}
          </div>
        </div>
      </div>
    </header>
  );
}
