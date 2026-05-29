import { useState, type FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useAuth } from '@/context/auth-context'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Container } from 'lucide-react'

export function LoginPage() {
  const { t } = useTranslation()
  const { login } = useAuth()
  const navigate = useNavigate()
  const [key, setKey] = useState('')
  const [remember, setRemember] = useState(false)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    if (!key.trim()) {
      setError(t('Please enter an API key'))
      return
    }
    setLoading(true)
    setError('')

    try {
      const res = await fetch('/health', {
        headers: { Authorization: `Bearer ${key}` },
      })
      if (res.status === 401) {
        setError(t('Invalid API key'))
        return
      }
      login(key, remember)
      navigate('/')
    } catch {
      login(key, remember)
      navigate('/')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30">
      <Card className="w-full max-w-sm">
        <CardHeader className="text-center">
          <Container className="mx-auto h-10 w-10 text-primary" />
          <CardTitle className="mt-2">{t('AgentBox Admin')}</CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="apikey">{t('API Key')}</Label>
              <Input
                id="apikey"
                type="password"
                placeholder={t('Enter your API key')}
                value={key}
                onChange={(e) => setKey(e.target.value)}
                autoFocus
              />
            </div>
            <div className="flex items-center gap-2">
              <Checkbox
                id="remember"
                checked={remember}
                onCheckedChange={setRemember}
              />
              <Label htmlFor="remember" className="text-xs font-normal">
                {t('Remember me')}
              </Label>
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full" disabled={loading}>
              {loading ? t('Connecting...') : t('Connect')}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
