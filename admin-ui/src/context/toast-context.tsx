import { createContext, useCallback, useContext, useState, type ReactNode } from 'react'

export interface Toast {
  id: number
  message: string
  variant: 'success' | 'error'
}

interface ToastContextValue {
  toasts: Toast[]
  success: (message: string) => void
  error: (message: string) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

let nextId = 0

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])

  const addToast = useCallback((message: string, variant: 'success' | 'error') => {
    const id = nextId++
    setToasts((prev) => [...prev, { id, message, variant }])
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id))
    }, 4000)
  }, [])

  const success = useCallback((message: string) => addToast(message, 'success'), [addToast])
  const error = useCallback((message: string) => addToast(message, 'error'), [addToast])

  return (
    <ToastContext.Provider value={{ toasts, success, error }}>
      {children}
    </ToastContext.Provider>
  )
}

export function useToast() {
  const ctx = useContext(ToastContext)
  if (!ctx) throw new Error('useToast must be used within ToastProvider')
  return ctx
}
