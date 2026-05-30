import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Plus, Search, ChevronLeft, ChevronRight, ArrowUpDown, Trash2 } from 'lucide-react'
import { useContainers, useDeleteContainer } from '@/hooks/use-containers'
import { useToast } from '@/context/toast-context'
import { Header } from '@/components/layout/header'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { StatusBadge } from '@/components/containers/status-badge'
import { Skeleton } from '@/components/ui/skeleton'
import { Dialog, DialogHeader, DialogTitle, DialogClose } from '@/components/ui/dialog'
import { formatDate } from '@/lib/utils'

export function ContainerListPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('')
  const [sortBy, setSortBy] = useState('created_at')
  const [sortOrder, setSortOrder] = useState('desc')
  const [page, setPage] = useState(1)
  const [deleteId, setDeleteId] = useState<string | null>(null)

  const { data, isLoading } = useContainers({
    search: search || undefined,
    status: statusFilter || undefined,
    sort_by: sortBy,
    sort_order: sortOrder,
    page,
    per_page: 20,
  })
  const deleteMutation = useDeleteContainer()
  const toast = useToast()

  const toggleSort = (col: string) => {
    if (sortBy === col) {
      setSortOrder((o) => (o === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortBy(col)
      setSortOrder('desc')
    }
  }

  const totalPages = data?.total_pages ?? 1

  return (
    <div>
      <Header
        title={t('Containers')}
        actions={
          <Button size="sm" onClick={() => navigate('/containers/new')}>
            <Plus className="h-4 w-4" />
            {t('Create')}
          </Button>
        }
      />
      <div className="p-6 space-y-4">
        <div className="flex gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder={t('Search by task or ID...')}
              value={search}
              onChange={(e) => {
                setSearch(e.target.value)
                setPage(1)
              }}
              className="pl-9"
            />
          </div>
          <Select
            value={statusFilter || undefined}
            onValueChange={(value) => {
              setStatusFilter(value === 'all' ? '' : value)
              setPage(1)
            }}
          >
            <SelectTrigger className="w-40">
              <SelectValue placeholder={t('All Status')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('All Status')}</SelectItem>
              <SelectItem value="Running">{t('Running')}</SelectItem>
              <SelectItem value="Idle">{t('Idle')}</SelectItem>
              <SelectItem value="Stopped">{t('Stopped')}</SelectItem>
              <SelectItem value="Failed">{t('Failed')}</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {isLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
        ) : data && data.data.length > 0 ? (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[120px]">{t('ID')}</TableHead>
                  <TableHead className="cursor-pointer select-none" onClick={() => toggleSort('task')}>
                    <span className="flex items-center gap-1">
                      {t('Task')} <ArrowUpDown className="h-3 w-3" />
                    </span>
                  </TableHead>
                  <TableHead className="cursor-pointer select-none" onClick={() => toggleSort('status')}>
                    <span className="flex items-center gap-1">
                      {t('Status')} <ArrowUpDown className="h-3 w-3" />
                    </span>
                  </TableHead>
                  <TableHead className="cursor-pointer select-none" onClick={() => toggleSort('created_at')}>
                    <span className="flex items-center gap-1">
                      {t('Created')} <ArrowUpDown className="h-3 w-3" />
                    </span>
                  </TableHead>
                  <TableHead className="w-[80px]">{t('Actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.data.map((c) => (
                  <TableRow
                    key={c.id}
                    className="cursor-pointer"
                    onClick={() => navigate(`/containers/${c.id}`)}
                  >
                    <TableCell className="font-mono text-xs">{c.id.slice(0, 12)}...</TableCell>
                    <TableCell className="max-w-60 truncate">{c.task}</TableCell>
                    <TableCell>
                      <StatusBadge status={c.status} />
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {formatDate(c.created_at)}
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={(e) => {
                          e.stopPropagation()
                          setDeleteId(c.id)
                        }}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">
                {t('Page {{page}} of {{totalPages}} ({{total}} total)', {
                  page,
                  totalPages,
                  total: data.total,
                })}
              </span>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page <= 1}
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                >
                  <ChevronLeft className="h-4 w-4" /> {t('Prev')}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page >= totalPages}
                  onClick={() => setPage((p) => p + 1)}
                >
                  {t('Next')} <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </>
        ) : (
          <div className="py-16 text-center">
            <p className="text-sm text-muted-foreground">
              {search || statusFilter ? t('No containers match your filters.') : t('No containers yet.')}
            </p>
            <Button variant="link" onClick={() => navigate('/containers/new')}>
              {t('Create your first container')}
            </Button>
          </div>
        )}
      </div>

      <Dialog open={!!deleteId} onOpenChange={(o) => !o && setDeleteId(null)}>
        <DialogClose onClick={() => setDeleteId(null)} />
        <DialogHeader>
          <DialogTitle>{t('Delete Container')}</DialogTitle>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          {t('Are you sure? This will stop and remove the container permanently.')}
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <Button variant="outline" onClick={() => setDeleteId(null)}>
            {t('Cancel')}
          </Button>
          <Button
            variant="destructive"
            onClick={() => {
              if (deleteId) {
                deleteMutation.mutate(deleteId, {
                  onSuccess: () => toast.success(t('Container deleted successfully')),
                  onError: () => toast.error(t('Failed to delete container')),
                })
                setDeleteId(null)
              }
            }}
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? t('Deleting...') : t('Delete')}
          </Button>
        </div>
      </Dialog>
    </div>
  )
}
