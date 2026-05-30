import { useTranslation } from 'react-i18next'
import { Moon, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/context/theme-context'

export function ThemeToggle() {
  const { t } = useTranslation()
  const { theme, toggleTheme } = useTheme()

  return (
    <Button variant="ghost" size="sm" className="w-full justify-start text-sidebar-foreground" onClick={toggleTheme}>
      {theme === 'light' ? (
        <>
          <Moon className="mr-2 h-4 w-4" />
          <span className="text-xs">{t('Dark')}</span>
        </>
      ) : (
        <>
          <Sun className="mr-2 h-4 w-4" />
          <span className="text-xs">{t('Light')}</span>
        </>
      )}
    </Button>
  )
}
