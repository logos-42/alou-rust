/**
 * useApiConfig - 本地 API 配置 Hook
 *
 * 通过 Tauri invoke 读取/保存 API 配置，完全不依赖 Workers 后端。
 * 配置加密存储于本地文件：~/Library/Application Support/alou/agent_config.json
 */

import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface UserApiConfig {
  id: string
  provider: string
  api_key: string
  base_url: string | null
  model: string | null
  is_active: boolean
}

export interface ApiConfig {
  user_apis: UserApiConfig[]
  workers_api: {
    base_url: string
    api_key?: string | null
    enabled: boolean
  }
  execution_strategy: string
  default_provider: string
}

const DEFAULT_PROVIDER = 'deepseek'
// 各平台 API 最新默认模型名称 (2025-2026)
const DEFAULT_MODELS: Record<string, string> = {
  deepseek: 'deepseek-chat',              // DeepSeek V3.2
  openai: 'gpt-5.4',                      // OpenAI GPT-5.4 (2026)
  claude: 'claude-sonnet-4-6-20260218',   // Claude Sonnet 4.6 (2026)
  qwen: 'qwen3.5-plus',                   // Qwen3.5 Plus
  kimi: 'kimi-k2.5',                      // Kimi K2.5
  minimax: 'minimax-m2.5',                // MiniMax M2.5
  glm: 'glm-5',                           // GLM-5
  gemini: 'gemini-3.1-pro',               // Gemini 3.1 Pro
}

/**
 * 获取当前激活的 API 配置（Tauri invoke）
 * 如果 Tauri 不可用，回退到 localStorage
 */
export async function getActiveApiConfig(): Promise<UserApiConfig | null> {
  try {
    const config = await invoke<ApiConfig>('get_agent_config')
    const active = config.user_apis.find((api) => api.is_active)
    if (active && active.api_key) {
      return active
    }
    // 有配置但没有激活的，取第一个
    if (config.user_apis.length > 0 && config.user_apis[0].api_key) {
      return config.user_apis[0]
    }
  } catch (e) {
    console.warn('[useApiConfig] Tauri invoke 失败，回退到 localStorage:', e)
  }

  // 回退：从 localStorage 读取
  const key = localStorage.getItem('user_api_key')
  const provider = localStorage.getItem('ai_provider') || DEFAULT_PROVIDER
  const model =
    localStorage.getItem('ai_model') || DEFAULT_MODELS[provider] || 'deepseek-chat'

  if (!key) return null

  return {
    id: 'local',
    provider,
    api_key: key,
    base_url: null,
    model,
    is_active: true,
  }
}

/**
 * 保存 API 配置（Tauri invoke）
 * 同时写入 localStorage 作为备份
 */
export async function saveApiConfig(
  apiKey: string,
  provider: string,
  model: string,
  baseUrl?: string,
): Promise<void> {
  // localStorage 备份
  localStorage.setItem('user_api_key', apiKey.trim())
  localStorage.setItem('ai_provider', provider)
  localStorage.setItem('ai_model', model.trim())

  // 通过 Tauri 保存加密配置
  try {
    const newConfig: ApiConfig = {
      user_apis: [
        {
          id: 'primary',
          provider,
          api_key: apiKey.trim(),
          base_url: baseUrl || null,
          model: model.trim(),
          is_active: true,
        },
      ],
      workers_api: {
        base_url: 'https://alou-edge.yuanjieliu65.workers.dev',
        api_key: null,
        enabled: false, // 禁用 workers，使用本地 API
      },
      execution_strategy: 'LocalOnly',
      default_provider: provider,
    }
    await invoke('update_agent_config', { config: newConfig })
    console.log('[useApiConfig] 配置已保存到 Tauri 加密存储')
  } catch (e) {
    console.warn('[useApiConfig] Tauri 保存失败，仅保存到 localStorage:', e)
  }
}

/**
 * React Hook：读取并监听 API 配置
 */
export function useApiConfig() {
  const [apiConfig, setApiConfig] = useState<UserApiConfig | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [hasConfig, setHasConfig] = useState(false)

  const loadConfig = useCallback(async () => {
    setIsLoading(true)
    try {
      const config = await getActiveApiConfig()
      setApiConfig(config)
      setHasConfig(!!config?.api_key)
    } catch (e) {
      console.error('[useApiConfig] 加载配置失败:', e)
      setApiConfig(null)
      setHasConfig(false)
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    loadConfig()

    // 监听配置变更事件（ApiConfigModal 保存后触发）
    const handleConfigChanged = () => {
      console.log('[useApiConfig] 检测到配置变更，重新加载')
      loadConfig()
    }

    window.addEventListener('api-config-changed', handleConfigChanged)
    return () => window.removeEventListener('api-config-changed', handleConfigChanged)
  }, [loadConfig])

  return {
    apiConfig,
    isLoading,
    hasConfig,
    reloadConfig: loadConfig,
  }
}
