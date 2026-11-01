// Input component
import React from 'react';
import { cn } from '@/lib/utils';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, helperText, leftIcon, rightIcon, id, ...props }, ref) => {
    const inputId = id || `input-${Math.random().toString(36).substring(7)}`;
    const errorId = `${inputId}-error`;
    const helperId = `${inputId}-helper`;

    return (
      <div className="w-full">
        {label && (
          <label htmlFor={inputId} className="label">
            {label}
          </label>
        )}
        <div className="relative">
          {leftIcon && (
            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-surface-500 pointer-events-none">
              {leftIcon}
            </div>
          )}
          <input
            ref={ref}
            id={inputId}
            className={cn(
              'w-full px-3 py-2 rounded-lg bg-surface-800 border text-surface-100 placeholder-surface-500 focus:outline-none focus:ring-2 focus:border-transparent transition-all duration-200',
              leftIcon ? 'pl-10' : '',
              rightIcon ? 'pr-10' : '',
              error ? 'border-red-500 focus:ring-red-500' : 'border-surface-600 focus:ring-primary-500',
              className
            )}
            aria-invalid={error ? 'true' : 'false'}
            aria-describedby={error ? errorId : helperText ? helperId : undefined}
            {...props}
          />
          {rightIcon && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2 text-surface-500 pointer-events-none">
              {rightIcon}
            </div>
          )}
        </div>
        {error && (
          <p id={errorId} className="mt-1 text-sm text-red-500" role="alert">
            {error}
          </p>
        )}
        {helperText && !error && (
          <p id={helperId} className="mt-1 text-sm text-surface-500">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';