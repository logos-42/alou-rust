/**
 * useMediaConfig - 媒体 API 配置 Hook
 * 
 * 通过 Tauri invoke 读取/保存媒体 Provider 配置
 * 配置加密存储于本地文件：~/Library/Application Support/alou/media_config.json
 */

import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface MediaProviderConfig {
  name: string
  api_key: string
  base_url: string | null
  model: string | null
  enabled: boolean
  capabilities: string[]
  config: Record<string, any> | null
}

export interface MediaApiConfig {
  providers: Record<string, MediaProviderConfig>
}

const DEFAULT_MEDIA_CONFIG: MediaApiConfig = {
  providers: {
    minimax: {
      name: 'minimax',
      api_key: '',
      base_url: null,
      model: null,
      enabled: false,
      capabilities: ['tts', 'video'],
      config: null,
    },
    minimax_music: {
      name: 'minimax_music',
      api_key: '',
      base_url: null,
      model: 'music-2.5+',
      enabled: false,
      capabilities: ['music'],
      config: null,
    },
    google: {
      name: 'google',
      api_key: '',
      base_url: null,
      model: null,
      enabled: false,
      capabilities: ['image'],
      config: null,
    },
    jimeng: {
      name: 'jimeng',
      api_key: '',
      base_url: null,
      model: null,
      enabled: false,
      capabilities: ['image', 'video'],
      config: null,
    },
    seedance: {
      name: 'seedance',
      api_key: '',
      base_url: 'https://api.302.ai/doubao',
      model: 'seedance-1.0-pro',
      enabled: false,
      capabilities: ['video'],
      config: null,
    },
    seedream: {
      name: 'seedream',
      api_key: '',
      base_url: 'https://api.deerapi.com',
      model: 'doubao-seedream-5-0-260128',
      enabled: false,
      capabilities: ['image'],
      config: null,
    },
    haimian: {
      name: 'haimian',
      api_key: '',
      base_url: null,
      model: null,
      enabled: false,
      capabilities: ['music'],
      config: null,
    },
    suno: {
      name: 'suno',
      api_key: '',
      base_url: 'https://api.sunoapi.org',
      model: 'V5',
      enabled: false,
      capabilities: ['music'],
      config: null,
    },
  },
}

/**
 * 获取媒体配置
 */
export async function getMediaConfig(): Promise<MediaApiConfig> {
  try {
    const config = await invoke<MediaApiConfig>('get_media_config')
    return config
  } catch (e) {
    console.warn('[useMediaConfig] 加载配置失败，使用默认配置:', e)
    return DEFAULT_MEDIA_CONFIG
  }
}

/**
 * 保存媒体配置
 */
export async function saveMediaConfig(config: MediaApiConfig): Promise<void> {
  try {
    await invoke('update_media_config', { config })
    console.log('[useMediaConfig] 配置已保存')
  } catch (e) {
    console.error('[useMediaConfig] 保存配置失败:', e)
    throw e
  }
}

/**
 * 测试媒体 Provider 连接
 */
export async function testMediaProvider(
  providerName: string,
  config: MediaProviderConfig,
): Promise<{ success: boolean; message: string }> {
  try {
    const result = await invoke<{ success: boolean; message: string }>(
      'test_media_provider_connection',
      { providerName, config },
    )
    return result
  } catch (e) {
    return { success: false, message: `测试失败：${e}` }
  }
}

/**
 * React Hook：读取并监听媒体配置
 */
export function useMediaConfig() {
  const [mediaConfig, setMediaConfig] = useState<MediaApiConfig>(DEFAULT_MEDIA_CONFIG)
  const [isLoading, setIsLoading] = useState(true)
  const [hasConfig, setHasConfig] = useState(false)

  const loadConfig = useCallback(async () => {
    setIsLoading(true)
    try {
      const config = await getMediaConfig()
      setMediaConfig(config)
      // 检查是否有任何启用的 Provider
      const hasEnabled = Object.values(config.providers).some(p => p.enabled && p.api_key)
      setHasConfig(hasEnabled)
    } catch (e) {
      console.error('[useMediaConfig] 加载配置失败:', e)
      setMediaConfig(DEFAULT_MEDIA_CONFIG)
      setHasConfig(false)
    } finally {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    loadConfig()

    // 监听配置变更事件
    const handleConfigChanged = () => {
      console.log('[useMediaConfig] 检测到配置变更，重新加载')
      loadConfig()
    }

    window.addEventListener('media-config-changed', handleConfigChanged)
    return () => window.removeEventListener('media-config-changed', handleConfigChanged)
  }, [loadConfig])

  return {
    mediaConfig,
    isLoading,
    hasConfig,
    reloadConfig: loadConfig,
  }
}

/**
 * 获取指定 Provider 的配置
 */
export function getProviderConfig(
  mediaConfig: MediaApiConfig,
  providerName: string,
): MediaProviderConfig | undefined {
  return mediaConfig.providers[providerName]
}

/**
 * 更新指定 Provider 的配置
 */
export function updateProviderConfig(
  mediaConfig: MediaApiConfig,
  providerName: string,
  updates: Partial<MediaProviderConfig>,
): MediaApiConfig {
  const provider = mediaConfig.providers[providerName]
  if (!provider) {
    console.warn(`[useMediaConfig] Provider ${providerName} 不存在`)
    return mediaConfig
  }

  return {
    ...mediaConfig,
    providers: {
      ...mediaConfig.providers,
      [providerName]: { ...provider, ...updates },
    },
  }
}
