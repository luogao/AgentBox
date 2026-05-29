import { useTranslation } from 'react-i18next'
import { Globe } from 'lucide-react'
import { Select, SelectTrigger, SelectContent, SelectItem } from '@/components/ui/select'

const languages = [
  { value: 'en', label: 'English' },
  { value: 'zh-CN', label: '中文' },
]

export function LanguageSwitcher() {
  const { i18n } = useTranslation()

  return (
    <Select
      value={i18n.language}
      onValueChange={(lang) => i18n.changeLanguage(lang)}
    >
      <SelectTrigger className="h-8 text-xs">
        <Globe className="mr-1 h-3 w-3" />
        <span className="truncate">
          {languages.find((l) => l.value === i18n.language)?.label ?? 'English'}
        </span>
      </SelectTrigger>
      <SelectContent>
        {languages.map((lang) => (
          <SelectItem key={lang.value} value={lang.value}>
            {lang.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
