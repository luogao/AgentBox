import * as React from 'react'
import { cn } from '@/lib/utils'

interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  checked?: boolean
  onCheckedChange?: (checked: boolean) => void
}

function Checkbox({ className, checked, onCheckedChange, id, ...props }: CheckboxProps) {
  return (
    <input
      id={id}
      type="checkbox"
      checked={checked}
      onChange={(e) => onCheckedChange?.(e.target.checked)}
      className={cn(
        'h-4 w-4 rounded border-input text-primary focus:ring-2 focus:ring-ring focus:ring-offset-2',
        className,
      )}
      {...props}
    />
  )
}

export { Checkbox }
