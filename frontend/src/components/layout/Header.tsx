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
    <header className="sticky top-0 z-30 h-16 bg-surface-950/80 backdrop-blur-sm border-b border-surface-800">
      <div className="flex items-center justify-between h-full px-6">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-semibold text-surface-100">{title}</h1>
          {subtitle && (
            <span className="text-sm text-surface-500">{subtitle}</span>
          )}
        </div>

        <div className="flex items-center gap-3">
          {actions}

          {/* Theme Toggle */}
          <div className="flex items-center gap-1 bg-surface-800/50 rounded-lg p-1 border border-surface-700">
            {['light', 'dark', 'system'].map((t) => (
              <button
                key={t}
                onClick={() => setTheme(t as 'light' | 'dark' | 'system')}
                className={cn(
                  'p-2 rounded-md transition-all duration-200',
                  theme === t
                    ? 'bg-primary-600 text-white shadow-lg'
                    : 'text-surface-500 hover:text-surface-100 hover:bg-surface-700'
                )}
                aria-label={`Set theme to ${t}`}
                aria-pressed={theme === t}
              >
                {t === 'light' && <Sun className="w-4 h-4" />}
                {t === 'dark' && <Moon className="w-4 h-4" />}
                {t === 'system' && <Monitor className="w-4 h-4" />}
              </button>
            ))}
          </div>
        </div>
      </div>
    </header>
  );
}