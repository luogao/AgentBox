import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'

const statusVariant: Record<string, 'success' | 'warning' | 'info' | 'destructive' | 'outline' | 'default'> = {
  Running: 'success',
  Idle: 'warning',
  Stopped: 'outline',
  Failed: 'destructive',
  Creating: 'info',
  Stopping: 'warning',
}

export function StatusBadge({ status }: { status: string }) {
  const { t } = useTranslation()
  return <Badge variant={statusVariant[status] ?? 'default'}>{t(status)}</Badge>
}
