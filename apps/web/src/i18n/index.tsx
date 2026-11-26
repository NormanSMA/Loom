import { createContext, useContext, useState, type ReactNode } from 'react'
import es from './locales/es.json'
import en from './locales/en.json'
import ptBr from './locales/pt-BR.json'

export type LocaleCode = 'es' | 'en' | 'pt-BR'
export type MessageKey = keyof typeof es

const DICTIONARIES: Record<LocaleCode, Record<MessageKey, string>> = {
  es,
  en,
  'pt-BR': ptBr,
}

interface I18nContextValue {
  locale: LocaleCode
  setLocale: (next: LocaleCode) => void
  t: (key: MessageKey) => string
}

const I18nContext = createContext<I18nContextValue | null>(null)
const STORAGE_KEY = 'loom.locale.v1'

function initialLocale(): LocaleCode {
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY)
    if (saved === 'es' || saved === 'en' || saved === 'pt-BR') return saved
  } catch {
    /* ignore storage unavailability */
  }
  return 'es'
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<LocaleCode>(initialLocale)

  const setLocale = (next: LocaleCode) => {
    setLocaleState(next)
    try {
      window.localStorage.setItem(STORAGE_KEY, next)
    } catch {
      /* ignore */
    }
  }

  const t = (key: MessageKey): string => DICTIONARIES[locale][key] ?? DICTIONARIES.es[key]

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  )
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) {
    throw new Error('useI18n must be used inside I18nProvider')
  }
  return ctx
}
