import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Plus, Trash2 } from 'lucide-react'
import { useCreateContainer } from '@/hooks/use-containers'
import { useToast } from '@/context/toast-context'
import { Header } from '@/components/layout/header'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'

export function CreateContainerPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const toast = useToast()
  const mutation = useCreateContainer()

  const [task, setTask] = useState('')
  const [skillRepos, setSkillRepos] = useState<string[]>([])
  const [skillBranch, setSkillBranch] = useState('main')
  const [cpuLimit, setCpuLimit] = useState('2')
  const [memoryLimit, setMemoryLimit] = useState('4Gi')
  const [idleTimeout, setIdleTimeout] = useState('300')
  const [maxLifetime, setMaxLifetime] = useState('3600')
  const [envKeys, setEnvKeys] = useState<string[]>([])
  const [envValues, setEnvValues] = useState<string[]>([])
  const [error, setError] = useState('')

  const addSkillRepo = () => setSkillRepos([...skillRepos, ''])
  const removeSkillRepo = (i: number) => {
    if (skillRepos.length > 1) {
      setSkillRepos(skillRepos.filter((_, idx) => idx !== i))
    }
  }

  const addEnv = () => {
    setEnvKeys([...envKeys, ''])
    setEnvValues([...envValues, ''])
  }
  const removeEnv = (i: number) => {
    setEnvKeys(envKeys.filter((_, idx) => idx !== i))
    setEnvValues(envValues.filter((_, idx) => idx !== i))
  }

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError('')

    if (!task.trim()) {
      setError(t('Task is required'))
      return
    }
    const repos = skillRepos.filter((r) => r.trim())

    const env: Record<string, string> = {}
    for (let i = 0; i < envKeys.length; i++) {
      if (envKeys[i].trim()) {
        env[envKeys[i].trim()] = envValues[i] || ''
      }
    }

    mutation.mutate(
      {
        task: task.trim(),
        skill_repos: repos.length > 0 ? repos : undefined,
        skill_branch: skillBranch || undefined,
        cpu_limit: cpuLimit || undefined,
        memory_limit: memoryLimit || undefined,
        idle_timeout: parseInt(idleTimeout) || undefined,
        max_lifetime: parseInt(maxLifetime) || undefined,
        env: Object.keys(env).length > 0 ? env : undefined,
      },
      {
        onSuccess: (data) => {
          toast.success(t('Container created successfully'))
          navigate(`/containers/${data.id}`)
        },
        onError: (err) => {
          toast.error(err.message || t('Failed to create container'))
          setError(err.message)
        },
      },
    )
  }

  return (
    <div>
      <Header title={t('Create Container')} />
      <div className="mx-auto max-w-2xl p-6">
        <Card>
          <CardContent className="pt-6">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="task">{t('Task *')}</Label>
                <Textarea
                  id="task"
                  placeholder={t('e.g. Review code PR #42')}
                  value={task}
                  onChange={(e) => setTask(e.target.value)}
                  rows={3}
                />
              </div>

              <div className="space-y-2">
                <Label className="block">{t('Skill Repos')}</Label>
                {skillRepos.map((repo, i) => (
                  <div key={i} className="flex gap-2">
                    <Input
                      placeholder={t('https://github.com/org/skills.git')}
                      value={repo}
                      onChange={(e) => {
                        const next = [...skillRepos]
                        next[i] = e.target.value
                        setSkillRepos(next)
                      }}
                    />
                    {skillRepos.length > 1 && (
                      <Button variant="ghost" size="icon" type="button" onClick={() => removeSkillRepo(i)}>
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                ))}
                <Button variant="outline" size="sm" type="button" onClick={addSkillRepo}>
                  <Plus className="h-4 w-4" /> {t('Add Repo')}
                </Button>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="branch">{t('Skill Branch')}</Label>
                  <Input id="branch" value={skillBranch} onChange={(e) => setSkillBranch(e.target.value)} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="cpu">{t('CPU Limit')}</Label>
                  <Input id="cpu" value={cpuLimit} onChange={(e) => setCpuLimit(e.target.value)} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="memory">{t('Memory Limit')}</Label>
                  <Input id="memory" value={memoryLimit} onChange={(e) => setMemoryLimit(e.target.value)} placeholder={t('e.g. 4Gi, 512Mi')} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="idle">{t('Idle Timeout (s)')}</Label>
                  <Input id="idle" type="number" value={idleTimeout} onChange={(e) => setIdleTimeout(e.target.value)} />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="lifetime">{t('Max Lifetime (s)')}</Label>
                  <Input id="lifetime" type="number" value={maxLifetime} onChange={(e) => setMaxLifetime(e.target.value)} />
                </div>
              </div>

              <details className="group">
                <summary className="cursor-pointer text-sm font-medium text-muted-foreground hover:text-foreground">
                  {t('Environment Variables')}
                </summary>
                <div className="mt-3 space-y-2">
                  {envKeys.map((_, i) => (
                    <div key={i} className="flex gap-2">
                      <Input
                        placeholder={t('KEY')}
                        value={envKeys[i]}
                        onChange={(e) => {
                          const next = [...envKeys]
                          next[i] = e.target.value
                          setEnvKeys(next)
                        }}
                        className="w-1/3"
                      />
                      <Input
                        placeholder={t('value')}
                        value={envValues[i]}
                        onChange={(e) => {
                          const next = [...envValues]
                          next[i] = e.target.value
                          setEnvValues(next)
                        }}
                      />
                      <Button variant="ghost" size="icon" type="button" onClick={() => removeEnv(i)}>
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                  <Button variant="outline" size="sm" type="button" onClick={addEnv}>
                    <Plus className="h-4 w-4" /> {t('Add Variable')}
                  </Button>
                </div>
              </details>

              {error && <p className="text-sm text-destructive">{error}</p>}
              {mutation.isError && !error && (
                <p className="text-sm text-destructive">{mutation.error.message}</p>
              )}

              <div className="flex justify-end gap-3 pt-2">
                <Button variant="outline" type="button" onClick={() => navigate('/containers')}>
                  {t('Cancel')}
                </Button>
                <Button type="submit" disabled={mutation.isPending}>
                  {mutation.isPending ? t('Creating...') : t('Create Container')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
