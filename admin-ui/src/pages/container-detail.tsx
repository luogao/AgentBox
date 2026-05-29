import { useParams, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, RefreshCw, Trash2 } from 'lucide-react'
import { useContainer, useDeleteContainer } from '@/hooks/use-containers'
import { Header } from '@/components/layout/header'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { StatusBadge } from '@/components/containers/status-badge'
import { LogViewer } from '@/components/logs/log-viewer'
import { Skeleton } from '@/components/ui/skeleton'
import { formatDate } from '@/lib/utils'

export function ContainerDetailPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { data: container, isLoading, isError } = useContainer(id)
  const deleteMutation = useDeleteContainer()

  const handleDelete = () => {
    if (id) {
      deleteMutation.mutate(id, {
        onSuccess: () => navigate('/containers'),
      })
    }
  }

  if (isLoading) {
    return (
      <div>
        <Header title={t('Container Detail')} />
        <div className="p-6 space-y-4">
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-64 w-full" />
        </div>
      </div>
    )
  }

  if (isError || !container) {
    return (
      <div>
        <Header title={t('Container Detail')} />
        <div className="flex flex-col items-center gap-4 p-16">
          <p className="text-muted-foreground">{t('Container not found')}</p>
          <Button variant="outline" onClick={() => navigate('/containers')}>
            <ArrowLeft className="h-4 w-4" /> {t('Back to list')}
          </Button>
        </div>
      </div>
    )
  }

  const repos: string[] = (() => {
    try {
      return JSON.parse(container.skill_repos)
    } catch {
      return []
    }
  })()

  return (
    <div>
      <Header
        title={t('Container {{id}}...', { id: container.id.slice(0, 12) })}
        actions={
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={() => navigate('/containers')}>
              <ArrowLeft className="h-4 w-4" /> {t('Back')}
            </Button>
            <Button variant="outline" size="sm" onClick={() => window.location.reload()}>
              <RefreshCw className="h-4 w-4" />
            </Button>
            <Button variant="destructive" size="sm" onClick={handleDelete} disabled={deleteMutation.isPending}>
              <Trash2 className="h-4 w-4" /> {t('Delete')}
            </Button>
          </div>
        }
      />
      <div className="p-6 space-y-6">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">{t('Status')}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <StatusBadge status={container.status} />
              <dl className="space-y-1 text-xs">
                <div className="flex gap-2">
                  <dt className="w-24 text-muted-foreground">{t('Docker ID')}</dt>
                  <dd className="font-mono">{container.docker_id ?? t('N/A')}</dd>
                </div>
                <div className="flex gap-2">
                  <dt className="w-24 text-muted-foreground">{t('Created')}</dt>
                  <dd>{formatDate(container.created_at)}</dd>
                </div>
                <div className="flex gap-2">
                  <dt className="w-24 text-muted-foreground">{t('Last Activity')}</dt>
                  <dd>{formatDate(container.last_activity)}</dd>
                </div>
              </dl>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-sm">{t('Configuration')}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-1 text-xs">
              <div className="flex gap-2">
                <dt className="w-28 text-muted-foreground">{t('Task')}</dt>
                <dd className="max-w-60 truncate">{container.task}</dd>
              </div>
              <div className="flex gap-2">
                <dt className="w-28 text-muted-foreground">{t('CPU / Memory')}</dt>
                <dd>
                  {container.cpu_limit} / {container.memory_limit}
                </dd>
              </div>
              <div className="flex gap-2">
                <dt className="w-28 text-muted-foreground">{t('Idle Timeout')}</dt>
                <dd>{container.idle_timeout}s</dd>
              </div>
              <div className="flex gap-2">
                <dt className="w-28 text-muted-foreground">{t('Max Lifetime')}</dt>
                <dd>{container.max_lifetime}s</dd>
              </div>
              {repos.length > 0 && (
                <div className="flex gap-2">
                  <dt className="w-28 text-muted-foreground">{t('Skill Repos')}</dt>
                  <dd>
                    {repos.map((r, i) => (
                      <div key={i} className="font-mono text-xs truncate">
                        {r}
                      </div>
                    ))}
                  </dd>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <CardTitle className="text-sm">{t('Logs')}</CardTitle>
          </CardHeader>
          <CardContent>
            <LogViewer containerId={container.id} />
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
