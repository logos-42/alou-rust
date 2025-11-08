import { useCallback, useEffect, useMemo, useState } from 'react'
import { messages, walletMessages } from '@/i18n/messages'

const listeners = new Set()
let currentLanguage = 'zh'

const notify = () => {
  listeners.forEach((listener) => listener(currentLanguage))
}

const setLanguageInternal = (lang) => {
  currentLanguage = lang === 'en' ? 'en' : 'zh'
  if (typeof window !== 'undefined') {
    localStorage.setItem('alou-language', currentLanguage)
  }
  notify()
}

const initLanguageInternal = () => {
  if (typeof window === 'undefined') {
    return
  }
  const saved = localStorage.getItem('alou-language')
  if (saved === 'zh' || saved === 'en') {
    currentLanguage = saved
  }
}

const buildDictionary = () => {
  const mergedZh = { ...messages.zh, ...walletMessages.zh }
  const mergedEn = { ...messages.en, ...walletMessages.en }
  return {
    zh: mergedZh,
    en: mergedEn,
  }
}

const dictionaries = buildDictionary()

const translate = (lang, key) => {
  const dict = dictionaries[lang] || dictionaries.zh
  return dict[key] || key
}

export const useI18n = () => {
  const [lang, setLang] = useState(currentLanguage)

  useEffect(() => {
    initLanguageInternal()
    setLang(currentLanguage)

    const listener = (nextLang) => setLang(nextLang)
    listeners.add(listener)
    return () => listeners.delete(listener)
  }, [])

  const t = useCallback((key) => translate(lang, key), [lang])

  const actions = useMemo(
    () => ({
      setLanguage: setLanguageInternal,
      initLanguage: initLanguageInternal,
    }),
    [],
  )

  return {
    t,
    currentLanguage: lang,
    ...actions,
  }
}

export const i18n = {
  setLanguage: setLanguageInternal,
  initLanguage: initLanguageInternal,
  t: (key) => translate(currentLanguage, key),
}

