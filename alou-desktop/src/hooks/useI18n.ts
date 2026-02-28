/**
 * 国际化 React Hook
 *
 * 使用方法：
 * ```jsx
 * import { useI18n } from '@/hooks/useI18n'
 *
 * function MyComponent() {
 *   const { t, currentLanguage, setLanguage } = useI18n()
 *
 *   return (
 *     <div>
 *       <h1>{t('agent.create.title')}</h1>
 *       <p>{t('agent.invite.selected', { count: 3 })}</p>
 *       <button onClick={() => setLanguage(currentLanguage === 'zh' ? 'en' : 'zh')}>
 *         切换语言
 *       </button>
 *     </div>
 *   )
 * }
 * ```
 *
 * 静态使用（非 React 组件中）：
 * ```js
 * import { i18n } from '@/hooks/useI18n'
 *
 * console.log(i18n.t('common.confirm'))
 * i18n.setLanguage('en')
 * ```
 */

import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  dictionary,
  getTranslation,
  SUPPORTED_LANGUAGES,
  DEFAULT_LANGUAGE,
  LANGUAGE_NAMES,
  findMissingTranslations,
} from '@/i18n'

// ==================== 类型定义 ====================

/**
 * 翻译参数类型
 */
export type TranslationParams = Record<string, string | number>

/**
 * 语言代码类型
 */
export type LanguageCode = 'zh' | 'en'

/**
 * 语言监听器类型
 */
type LanguageListener = (lang: string) => void

/**
 * I18n Hook 返回类型
 */
export interface UseI18nReturn {
  t: (key: string, params?: TranslationParams) => string
  currentLanguage: string
  setLanguage: (lang: string) => void
  toggleLanguage: () => string
  initLanguage: () => void
  languageNames: Record<string, string>
  supportedLanguages: string[]
}

/**
 * 静态 I18n API 类型
 */
export interface I18nStaticAPI {
  t: (key: string, params?: TranslationParams) => string
  setLanguage: (lang: string) => void
  toggleLanguage: () => string
  initLanguage: () => void
  getCurrentLanguage: () => string
  languageNames: Record<string, string>
  supportedLanguages: string[]
  dictionary: typeof dictionary
  findMissingTranslations: typeof findMissingTranslations
}

// ==================== 全局状态管理 ====================

const listeners = new Set<LanguageListener>()
let currentLanguage: string = DEFAULT_LANGUAGE

const notify = (): void => {
  listeners.forEach((listener) => listener(currentLanguage))
}

const setLanguageInternal = (lang: string): void => {
  currentLanguage = SUPPORTED_LANGUAGES.includes(lang) ? lang : DEFAULT_LANGUAGE
  if (typeof window !== 'undefined') {
    localStorage.setItem('alou-language', currentLanguage)
    // 同时设置 HTML lang 属性，有助于 SEO 和无障碍访问
    document.documentElement.lang = currentLanguage
  }
  notify()
}

const initLanguageInternal = (): void => {
  if (typeof window === 'undefined') {
    return
  }

  const saved = localStorage.getItem('alou-language')
  if (saved && SUPPORTED_LANGUAGES.includes(saved)) {
    currentLanguage = saved
  } else {
    // 尝试从浏览器语言推断
    const browserLang = navigator.language?.toLowerCase()
    if (browserLang?.startsWith('zh')) {
      currentLanguage = 'zh'
    } else if (browserLang?.startsWith('en')) {
      currentLanguage = 'en'
    }
    // 保存推断的语言
    localStorage.setItem('alou-language', currentLanguage)
  }

  document.documentElement.lang = currentLanguage
}

const toggleLanguage = (): string => {
  const nextLang = currentLanguage === 'zh' ? 'en' : 'zh'
  setLanguageInternal(nextLang)
  return nextLang
}

// ==================== React Hook ====================

/**
 * 国际化 Hook
 * @returns 包含翻译函数和语言管理方法的对象
 */
export const useI18n = (): UseI18nReturn => {
  const [lang, setLang] = useState<string>(currentLanguage)

  useEffect(() => {
    initLanguageInternal()
    // eslint-disable-next-line react-hooks/setState-in-effect
    setLang(currentLanguage)

    const listener: LanguageListener = (nextLang) => setLang(nextLang)
    listeners.add(listener)
    return () => {
      listeners.delete(listener)
    }
  }, [])

  // 翻译函数，支持参数插值
  const t = useCallback(
    (key: string, params?: TranslationParams): string => getTranslation(key, lang, params),
    [lang],
  )

  const actions = useMemo(
    () => ({
      setLanguage: setLanguageInternal,
      toggleLanguage,
      initLanguage: initLanguageInternal,
      languageNames: LANGUAGE_NAMES,
      supportedLanguages: SUPPORTED_LANGUAGES,
    }),
    [],
  )

  return {
    t,
    currentLanguage: lang,
    ...actions,
  }
}

// ==================== 静态 API（非 React 组件使用）====================

/**
 * 静态国际化对象，用于非 React 组件中
 * @example
 * import { i18n } from '@/hooks/useI18n'
 * console.log(i18n.t('common.confirm'))
 */
export const i18n: I18nStaticAPI = {
  /**
   * 翻译文本
   * @param key - 翻译 key
   * @param params - 插值参数
   */
  t: (key: string, params?: TranslationParams): string => getTranslation(key, currentLanguage, params),

  /**
   * 设置语言
   * @param lang - 语言代码 ('zh' | 'en')
   */
  setLanguage: setLanguageInternal,

  /**
   * 切换语言（中英文切换）
   * @returns 切换后的语言
   */
  toggleLanguage,

  /**
   * 初始化语言（从 localStorage 或浏览器语言推断）
   */
  initLanguage: initLanguageInternal,

  /**
   * 获取当前语言
   * @returns 当前语言代码
   */
  getCurrentLanguage: (): string => currentLanguage,

  /**
   * 语言显示名称映射
   */
  languageNames: LANGUAGE_NAMES,

  /**
   * 支持的语言列表
   */
  supportedLanguages: SUPPORTED_LANGUAGES,

  /**
   * 完整的翻译字典（用于调试）
   */
  dictionary,

  /**
   * 查找缺失的翻译（用于调试）
   */
  findMissingTranslations,
}

export default useI18n
