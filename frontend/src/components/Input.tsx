import { InputHTMLAttributes, forwardRef } from 'react';
import { cn } from '@/lib/utils';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {}

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, ...props }, ref) => {
    return (
      <input
        ref={ref}
        className={cn(
          'bg-surface-0 border border-surface-1 rounded-lg px-4 py-2 text-text placeholder-overlay-0 focus:outline-none focus:ring-2 focus:ring-mauve focus:border-transparent w-full',
          className
        )}
        {...props}
      />
    );
  }
);

Input.displayName = 'Input';

export default Input;
