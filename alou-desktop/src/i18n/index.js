/**
 * 国际化统一入口
 *
 * 文件结构：
 * src/i18n/
 *   ├── index.js           # 统一入口（本文件）
 *   └── namespaces/        # 按功能模块分离的翻译文件
 *       ├── common.js      # 通用翻译（按钮、状态、导航等）
 *       ├── agent.js       # 智能体相关（创建、列表、对话、DIAP）
 *       ├── login.js       # 登录页面
 *       └── wallet.js      # 钱包相关
 *
 * 命名规范：
 * - 使用点分隔的层级结构：namespace.module.item
 * - 例如：agent.create.title, login.wallet.metamask
 *
 * 添加新翻译的步骤：
 * 1. 在对应的命名空间文件中添加 key-value
 * 2. 同时添加中文和英文翻译
 * 3. 组件中使用 t('agent.create.title') 获取翻译
 *
 * 添加新命名空间的步骤：
 * 1. 在 namespaces/ 目录下创建新文件
 * 2. 按照现有格式导出 { zh: {...}, en: {...} }
 * 3. 在本文件中导入并添加到 namespaces 数组
 */

import { common } from './namespaces/common'
import { agent } from './namespaces/agent'
import { login } from './namespaces/login'
import { wallet } from './namespaces/wallet'
import { subscription } from './namespaces/subscription'
import { legacy } from './namespaces/legacy'

// 所有命名空间列表 - 添加新命名空间时在这里注册
// legacy 放在最后，确保新的命名空间 key 优先
const namespaces = [common, agent, login, wallet, subscription, legacy]

// 支持的语言列表
export const SUPPORTED_LANGUAGES = ['zh', 'en']
export const DEFAULT_LANGUAGE = 'zh'

// 语言显示名称
export const LANGUAGE_NAMES = {
  zh: '中文',
  en: 'English',
}

/**
 * 合并所有命名空间的翻译
 * @returns {{ zh: Record<string, string>, en: Record<string, string> }}
 */
const mergeNamespaces = () => {
  const merged = {
    zh: {},
    en: {},
  }

  for (const namespace of namespaces) {
    Object.assign(merged.zh, namespace.zh || {})
    Object.assign(merged.en, namespace.en || {})
  }

  return merged
}

// 构建完整的翻译字典
export const dictionary = mergeNamespaces()

/**
 * 获取翻译文本
 * @param {string} key - 翻译 key，例如 'agent.create.title'
 * @param {string} lang - 语言代码，'zh' 或 'en'
 * @param {Record<string, string>} [params] - 插值参数，例如 { count: 3 }
 * @returns {string} 翻译后的文本，如果找不到则返回 key
 */
export const getTranslation = (key, lang = DEFAULT_LANGUAGE, params) => {
  const langDict = dictionary[lang] || dictionary[DEFAULT_LANGUAGE]
  let text = langDict[key]

  // 如果当前语言没有，尝试默认语言
  if (text === undefined && lang !== DEFAULT_LANGUAGE) {
    text = dictionary[DEFAULT_LANGUAGE][key]
  }

  // 如果还是没有，返回 key
  if (text === undefined) {
    console.warn(`[i18n] Missing translation for key: "${key}" in language: "${lang}"`)
    return key
  }

  // 处理插值参数 {name} -> value
  if (params) {
    Object.entries(params).forEach(([paramKey, value]) => {
      text = text.replace(new RegExp(`\\{${paramKey}\\}`, 'g'), String(value))
    })
  }

  return text
}

/**
 * 创建一个绑定了语言的翻译函数
 * @param {string} lang - 语言代码
 * @returns {(key: string, params?: Record<string, string>) => string}
 */
export const createTranslator = (lang) => {
  return (key, params) => getTranslation(key, lang, params)
}

/**
 * 检查翻译 key 是否存在
 * @param {string} key - 翻译 key
 * @param {string} [lang] - 可选的语言代码
 * @returns {boolean}
 */
export const hasTranslation = (key, lang) => {
  if (lang) {
    return key in (dictionary[lang] || {})
  }
  return SUPPORTED_LANGUAGES.some((l) => key in (dictionary[l] || {}))
}

/**
 * 获取所有翻译 keys（用于调试）
 * @returns {string[]}
 */
export const getAllKeys = () => {
  const keysSet = new Set([
    ...Object.keys(dictionary.zh),
    ...Object.keys(dictionary.en),
  ])
  return Array.from(keysSet).sort()
}

/**
 * 查找缺失的翻译（用于调试）
 * @returns {{ missingInZh: string[], missingInEn: string[] }}
 */
export const findMissingTranslations = () => {
  const allKeys = getAllKeys()
  const missingInZh = allKeys.filter((key) => !(key in dictionary.zh))
  const missingInEn = allKeys.filter((key) => !(key in dictionary.en))

  if (missingInZh.length > 0 || missingInEn.length > 0) {
    console.warn('[i18n] Missing translations found:', { missingInZh, missingInEn })
  }

  return { missingInZh, missingInEn }
}

// 导出默认对象方便引用
export default {
  dictionary,
  getTranslation,
  createTranslator,
  hasTranslation,
  getAllKeys,
  findMissingTranslations,
  SUPPORTED_LANGUAGES,
  DEFAULT_LANGUAGE,
  LANGUAGE_NAMES,
}

