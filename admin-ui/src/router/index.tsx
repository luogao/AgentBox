import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom'
import { useAuth } from '@/context/auth-context'
import { useSetupStatus } from '@/hooks/use-setup'
import { RootLayout } from '@/components/layout/root-layout'
import { LoginPage } from '@/pages/login'
import { DashboardPage } from '@/pages/dashboard'
import { ContainerListPage } from '@/pages/container-list'
import { CreateContainerPage } from '@/pages/container-create'
import { ContainerDetailPage } from '@/pages/container-detail'
import { SetupPage } from '@/pages/setup'

const SETUP_SKIPPED_KEY = 'setup_skipped'

function ProtectedRoute() {
  const { isAuthenticated } = useAuth()
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  return <Outlet />
}

function SetupGuardRoute() {
  const { isAuthenticated } = useAuth()
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  return <Outlet />
}

function SetupRequiredRoute() {
  const { isAuthenticated } = useAuth()
  const { data: status, isLoading } = useSetupStatus()
  const skipped = localStorage.getItem(SETUP_SKIPPED_KEY) === 'true'

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  if (isLoading) {
    return null
  }

  if (!status?.all_ready && !skipped) {
    return <Navigate to="/setup" replace />
  }

  return <Outlet />
}

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    element: <SetupGuardRoute />,
    children: [
      { path: '/setup', element: <SetupPage /> },
    ],
  },
  {
    element: <SetupRequiredRoute />,
    children: [
      {
        element: <RootLayout />,
        children: [
          { index: true, element: <DashboardPage /> },
          { path: '/containers', element: <ContainerListPage /> },
          { path: '/containers/new', element: <CreateContainerPage /> },
          { path: '/containers/:id', element: <ContainerDetailPage /> },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
])
