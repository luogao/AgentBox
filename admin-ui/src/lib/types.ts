export interface Container {
  id: string
  task: string
  status: string
  docker_id: string | null
  skill_repos: string
  cpu_limit: string
  memory_limit: string
  idle_timeout: number
  max_lifetime: number
  created_at: string
  last_activity: string
}

export type ContainerStatus = 'Creating' | 'Running' | 'Idle' | 'Stopping' | 'Stopped' | 'Failed'

export interface ContainerResponse {
  id: string
  status: ContainerStatus
  created_at: string
  docker_id: string | null
}

export interface CreateContainerRequest {
  task: string
  skill_repos?: string[]
  skill_branch?: string
  cpu_limit?: string
  memory_limit?: string
  idle_timeout?: number
  max_lifetime?: number
  env?: Record<string, string>
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface StatsResponse {
  total: number
  by_status: Record<string, number>
}

export interface SetupStatus {
  docker_connected: boolean
  agent_image_ready: boolean
  agent_image_name: string
  api_key_configured: boolean
  all_ready: boolean
  project_root: string
}

export interface UpdateConfigResponse {
  api_key_updated: boolean
  new_api_key: string | null
}
