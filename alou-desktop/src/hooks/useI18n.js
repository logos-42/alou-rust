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

// ==================== 全局状态管理 ====================

const listeners = new Set()
let currentLanguage = DEFAULT_LANGUAGE

const notify = () => {
  listeners.forEach((listener) => listener(currentLanguage))
}

const setLanguageInternal = (lang) => {
  currentLanguage = SUPPORTED_LANGUAGES.includes(lang) ? lang : DEFAULT_LANGUAGE
  if (typeof window !== 'undefined') {
    localStorage.setItem('alou-language', currentLanguage)
    // 同时设置 HTML lang 属性，有助于 SEO 和无障碍访问
    document.documentElement.lang = currentLanguage
  }
  notify()
}

const initLanguageInternal = () => {
  if (typeof window === 'undefined') {
    return
  }

  const saved = localStorage.getItem('alou-language')
  if (SUPPORTED_LANGUAGES.includes(saved)) {
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

const toggleLanguage = () => {
  const nextLang = currentLanguage === 'zh' ? 'en' : 'zh'
  setLanguageInternal(nextLang)
  return nextLang
}

// ==================== React Hook ====================

/**
 * 国际化 Hook
 * @returns {{
 *   t: (key: string, params?: Record<string, string>) => string,
 *   currentLanguage: string,
 *   setLanguage: (lang: string) => void,
 *   toggleLanguage: () => string,
 *   initLanguage: () => void,
 *   languageNames: Record<string, string>,
 *   supportedLanguages: string[]
 * }}
 */
export const useI18n = () => {
  const [lang, setLang] = useState(currentLanguage)

  useEffect(() => {
    initLanguageInternal()
    setLang(currentLanguage)

    const listener = (nextLang) => setLang(nextLang)
    listeners.add(listener)
    return () => listeners.delete(listener)
  }, [])

  // 翻译函数，支持参数插值
  const t = useCallback(
    (key, params) => getTranslation(key, lang, params),
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
export const i18n = {
  /**
   * 翻译文本
   * @param {string} key - 翻译 key
   * @param {Record<string, string>} [params] - 插值参数
   */
  t: (key, params) => getTranslation(key, currentLanguage, params),

  /**
   * 设置语言
   * @param {string} lang - 语言代码 ('zh' | 'en')
   */
  setLanguage: setLanguageInternal,

  /**
   * 切换语言（中英文切换）
   * @returns {string} 切换后的语言
   */
  toggleLanguage,

  /**
   * 初始化语言（从 localStorage 或浏览器语言推断）
   */
  initLanguage: initLanguageInternal,

  /**
   * 获取当前语言
   * @returns {string}
   */
  getCurrentLanguage: () => currentLanguage,

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
