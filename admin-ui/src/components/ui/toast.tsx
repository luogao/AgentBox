import { useToast } from '@/context/toast-context'
import { cn } from '@/lib/utils'
import { CheckCircle, XCircle, X } from 'lucide-react'

export function ToastContainer() {
  const { toasts } = useToast()

  if (toasts.length === 0) return null

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={cn(
            'flex items-center gap-2 rounded-md px-4 py-3 text-sm shadow-lg animate-in slide-in-from-bottom-2 fade-in',
            toast.variant === 'success'
              ? 'bg-green-600 text-white'
              : 'bg-destructive text-destructive-foreground',
          )}
        >
          {toast.variant === 'success' ? (
            <CheckCircle className="h-4 w-4 shrink-0" />
          ) : (
            <XCircle className="h-4 w-4 shrink-0" />
          )}
          <span>{toast.message}</span>
        </div>
      ))}
    </div>
  )
}
