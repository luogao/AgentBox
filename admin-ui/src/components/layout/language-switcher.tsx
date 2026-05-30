import { useTranslation } from 'react-i18next'

export function LanguageSwitcher() {
  const { i18n } = useTranslation()
  const isZh = i18n.language === 'zh-CN'

  return (
    <div className="inline-flex h-8 rounded-md bg-muted p-0.5 text-xs">
      <button
        className={`rounded-sm px-2.5 font-medium transition-colors ${
          !isZh ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
        }`}
        onClick={() => i18n.changeLanguage('en')}
      >
        EN
      </button>
      <button
        className={`rounded-sm px-2.5 font-medium transition-colors ${
          isZh ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
        }`}
        onClick={() => i18n.changeLanguage('zh-CN')}
      >
        中文
      </button>
    </div>
  )
}
