import { RouterProvider } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AuthProvider } from '@/context/auth-context'
import { ThemeProvider } from '@/context/theme-context'
import { ToastProvider } from '@/context/toast-context'
import { ToastContainer } from '@/components/ui/toast'
import { router } from '@/router'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 10_000,
    },
  },
})

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <ToastProvider>
          <AuthProvider>
            <RouterProvider router={router} />
          </AuthProvider>
          <ToastContainer />
        </ToastProvider>
      </ThemeProvider>
    </QueryClientProvider>
  )
}
