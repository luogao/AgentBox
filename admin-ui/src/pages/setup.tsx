import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { CheckCircle2, XCircle, RefreshCw, AlertCircle, Copy, ArrowRight, Save, Eye, EyeOff } from 'lucide-react'
import { useSetupStatus, useUpdateConfig } from '@/hooks/use-setup'
import { useAuth } from '@/context/auth-context'
import { Header } from '@/components/layout/header'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'

const SETUP_SKIPPED_KEY = 'setup_skipped'

export function SetupPage() {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const { login } = useAuth()
  const { data: status, isLoading, refetch } = useSetupStatus()
  const updateConfig = useUpdateConfig()
  const isZh = i18n.language === 'zh-CN'

  const [apiKeyInput, setApiKeyInput] = useState('')
  const [showApiKey, setShowApiKey] = useState(false)

  const passedCount = status
    ? [status.docker_connected, status.agent_image_ready, status.api_key_configured].filter(Boolean).length
    : 0
  const totalChecks = 3

  const allPassed = passedCount === totalChecks

  const handleSkip = () => {
    localStorage.setItem(SETUP_SKIPPED_KEY, 'true')
    navigate('/')
  }

  const handleContinue = () => {
    localStorage.removeItem(SETUP_SKIPPED_KEY)
    localStorage.setItem(SETUP_SKIPPED_KEY, 'true')
    navigate('/')
  }

  const handleSaveApiKey = async () => {
    if (!apiKeyInput.trim()) return
    const result = await updateConfig.mutateAsync({ api_key: apiKeyInput.trim() })
    if (result.new_api_key) {
      login(result.new_api_key, true)
      setApiKeyInput('')
    }
  }

  const copyCommand = async (command: string, btn: HTMLButtonElement) => {
    await navigator.clipboard.writeText(command)
    const original = btn.textContent
    btn.textContent = isZh ? '已复制' : 'Copied'
    setTimeout(() => { btn.textContent = original }, 2000)
  }

  const checks = [
    {
      key: 'docker_connected' as const,
      passed: status?.docker_connected,
      title: isZh ? 'Docker 已连接' : 'Docker Connected',
      failTitle: isZh ? 'Docker 未连接' : 'Docker Not Connected',
      solution: {
        steps: [
          isZh ? '请确保 Docker 守护进程正在运行' : 'Make sure Docker daemon is running',
          isZh ? '尝试重启 Docker Desktop' : 'Try restarting Docker Desktop',
        ],
      },
    },
    {
      key: 'agent_image_ready' as const,
      passed: status?.agent_image_ready,
      title: isZh ? 'Agent 镜像就绪' : 'Agent Image Ready',
      failTitle: isZh ? 'Agent 镜像未找到' : 'Agent Image Not Found',
      solution: {
        steps: [isZh ? '构建 Agent 容器镜像：' : 'Build the agent container image:'],
        command: `docker build -t ${status?.agent_image_name ?? 'agent-sandbox:latest'} -f ${status?.project_root ?? '.'}/agent-image/Dockerfile ${status?.project_root ?? '.'}`,
      },
    },
    {
      key: 'api_key_configured' as const,
      passed: status?.api_key_configured,
      title: isZh ? 'API 密钥已配置' : 'API Key Configured',
      failTitle: isZh ? 'API 密钥未设置' : 'API Key Not Set',
      hasInput: true,
    },
  ]

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-950 dark:to-slate-900">
      <Header title={isZh ? '系统设置向导' : 'Setup Wizard'} />
      <div className="p-6 max-w-3xl mx-auto space-y-6">
        <div className="text-center space-y-2">
          <h2 className="text-2xl font-bold">
            {isZh ? 'AgentBox 初始化检查' : 'AgentBox Setup Check'}
          </h2>
          <p className="text-muted-foreground">
            {isZh
              ? '请完成以下检查项以确保系统正常运行'
              : 'Complete the following checks to ensure the system is ready'}
          </p>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>{isZh ? '系统状态' : 'System Status'}</CardTitle>
              {!isLoading && status && (
                <div className="flex items-center gap-2">
                  <span className="text-sm text-muted-foreground">
                    {passedCount} / {totalChecks} {isZh ? '项通过' : 'checks passed'}
                  </span>
                  <Button variant="ghost" size="sm" onClick={() => refetch()} className="gap-1.5">
                    <RefreshCw className="h-4 w-4" />
                    {isZh ? '刷新' : 'Refresh'}
                  </Button>
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {isLoading
              ? Array.from({ length: 3 }).map((_, i) => (
                  <div key={i} className="flex items-center gap-3">
                    <Skeleton className="h-8 w-8 rounded-full" />
                    <Skeleton className="h-6 flex-1" />
                  </div>
                ))
              : checks.map((check) => {
                  const Icon = check.passed ? CheckCircle2 : XCircle

                  return (
                    <div key={check.key} className="space-y-2">
                      <div className="flex items-center gap-3">
                        <Icon className={`h-6 w-6 shrink-0 ${check.passed ? 'text-green-600' : 'text-red-600'}`} />
                        <span className={check.passed ? 'font-medium' : ''}>
                          {check.passed ? check.title : check.failTitle}
                        </span>
                      </div>

                      {!check.passed && check.hasInput && (
                        <div className="ml-9 space-y-3">
                          <p className="text-sm text-muted-foreground">
                            {isZh
                              ? '设置 API 密钥以保护管理后台访问：'
                              : 'Set an API key to protect admin access:'}
                          </p>
                          <div className="flex gap-2">
                            <div className="relative flex-1">
                              <Input
                                type={showApiKey ? 'text' : 'password'}
                                placeholder={isZh ? '输入 API 密钥...' : 'Enter API key...'}
                                value={apiKeyInput}
                                onChange={(e) => setApiKeyInput(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && handleSaveApiKey()}
                              />
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                className="absolute right-1 top-1/2 -translate-y-1/2 h-7 w-7 p-0"
                                onClick={() => setShowApiKey(!showApiKey)}
                              >
                                {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                              </Button>
                            </div>
                            <Button
                              onClick={handleSaveApiKey}
                              disabled={!apiKeyInput.trim() || updateConfig.isPending}
                              className="gap-1.5"
                            >
                              <Save className="h-4 w-4" />
                              {updateConfig.isPending
                                ? (isZh ? '保存中...' : 'Saving...')
                                : (isZh ? '保存' : 'Save')}
                            </Button>
                          </div>
                        </div>
                      )}

                      {!check.passed && 'solution' in check && check.solution && (
                        <div className="ml-9 space-y-2 text-sm text-muted-foreground">
                          {check.solution.steps.map((step, i) => (
                            <p key={i}>{step}</p>
                          ))}
                          {'command' in check.solution && check.solution.command && (
                            <div className="mt-2 relative group">
                              <pre className="bg-slate-900 text-slate-100 p-3 rounded-md text-xs overflow-x-auto">
                                <code>{check.solution.command}</code>
                              </pre>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="absolute top-1 right-1 h-7 px-2 opacity-0 group-hover:opacity-100 transition-opacity"
                                onClick={(e) => copyCommand(check.solution.command!, e.currentTarget)}
                              >
                                <Copy className="h-3.5 w-3.5" />
                              </Button>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  )
                })}
          </CardContent>
        </Card>

        <Card className={allPassed ? 'border-green-200 bg-green-50/50 dark:bg-green-950/20' : ''}>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                {allPassed ? (
                  <>
                    <CheckCircle2 className="h-8 w-8 text-green-600" />
                    <div>
                      <p className="font-medium text-green-900 dark:text-green-100">
                        {isZh ? '所有检查项已通过！系统已就绪。' : 'All checks passed! System is ready.'}
                      </p>
                      <p className="text-sm text-green-700 dark:text-green-300">
                        {isZh ? '您可以开始创建和管理 Agent 容器了' : 'You can now create and manage Agent containers'}
                      </p>
                    </div>
                  </>
                ) : (
                  <>
                    <AlertCircle className="h-8 w-8 text-amber-600" />
                    <div>
                      <p className="font-medium">
                        {isZh ? '部分检查项未通过' : 'Some checks are not passing'}
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {isZh ? '请按照上方提示完成配置后刷新状态' : 'Follow the instructions above and refresh the status'}
                      </p>
                    </div>
                  </>
                )}
              </div>
              {allPassed ? (
                <Button onClick={handleContinue} className="gap-2">
                  {isZh ? '继续' : 'Continue'}
                  <ArrowRight className="h-4 w-4" />
                </Button>
              ) : (
                <Button variant="outline" onClick={handleSkip}>
                  {isZh ? '跳过（高级）' : 'Skip (Advanced)'}
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
