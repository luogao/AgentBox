import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { get, post } from '@/lib/api-client'
import type { SetupStatus, UpdateConfigResponse } from '@/lib/types'

export function useSetupStatus() {
  return useQuery<SetupStatus>({
    queryKey: ['setup-status'],
    queryFn: () => get('/api/setup/status'),
    refetchInterval: false,
    staleTime: 5000,
  })
}

export function useUpdateConfig() {
  const qc = useQueryClient()
  return useMutation<UpdateConfigResponse, Error, { api_key?: string }>({
    mutationFn: (body) => post('/api/setup/config', body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['setup-status'] })
    },
  })
}
