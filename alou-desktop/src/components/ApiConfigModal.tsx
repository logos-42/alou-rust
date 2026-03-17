import { useState, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭 0.3.png'
import { saveApiConfig, getActiveApiConfig } from '@/hooks/useApiConfig'
import './ApiConfigModal.css'

// Tauri invoke 安全导入
let invokeCache = null;
async function getInvoke() {
  if (!invokeCache) {
    const isTauri = typeof window !== 'undefined' &&
                   (window.__TAURI__ !== undefined ||
                    window.__TAURI_IPC__ !== undefined ||
                    (typeof import.meta !== 'undefined' && Boolean(import.meta.env?.TAURI_PLATFORM)));

    if (isTauri) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        invokeCache = invoke;
      } catch (error) {
        console.warn('[ApiConfigModal] 无法导入 Tauri invoke:', error);
        invokeCache = createMockInvoke();
      }
    } else {
      invokeCache = createMockInvoke();
    }
  }
  return invokeCache;
}

function createMockInvoke() {
  return async (cmd, args) => {
    console.warn(`[MockInvoke] ${cmd}`, args);
    if (cmd === 'test_api_connection') return { success: true, message: '模拟连接成功' };
    if (cmd === 'get_agent_config') return { user_apis: [], workers_api: { base_url: '', enabled: false }, execution_strategy: 'LocalOnly', default_provider: 'deepseek' };
    if (cmd === 'update_agent_config') return undefined;
    return {};
  };
}

// Provider 定义（包含文本和媒体）
const PROVIDERS = [
  // 文本 LLM
  { value: 'deepseek', label: 'DeepSeek', category: 'text', capabilities: ['text'] },
  { value: 'openai', label: 'OpenAI', category: 'text', capabilities: ['text'] },
  { value: 'claude', label: 'Claude', category: 'text', capabilities: ['text'] },
  { value: 'qwen', label: 'Qwen', category: 'text', capabilities: ['text'] },
  { value: 'kimi', label: 'Kimi', category: 'text', capabilities: ['text'] },
  { value: 'minimax', label: 'MiniMax', category: 'text', capabilities: ['text'] },
  { value: 'glm', label: 'GLM', category: 'text', capabilities: ['text'] },
  { value: 'gemini', label: 'Gemini', category: 'text', capabilities: ['text'] },
  // 媒体生成
  { value: 'seedance', label: 'Seedance', category: 'media', capabilities: ['video'] },
  { value: 'google', label: 'Google Imagen', category: 'media', capabilities: ['image'] },
  { value: 'jimeng', label: '即梦', category: 'media', capabilities: ['image', 'video'] },
  { value: 'seedream', label: 'Seedream', category: 'media', capabilities: ['image'] },
  { value: 'haimian', label: '海绵音乐', category: 'media', capabilities: ['music'] },
  { value: 'suno', label: 'Suno AI', category: 'media', capabilities: ['music'] },
  { value: 'stability', label: 'Stability AI', category: 'media', capabilities: ['image'] },
  { value: 'elevenlabs', label: 'ElevenLabs', category: 'media', capabilities: ['tts'] },
  { value: 'minimax_music', label: 'MiniMax Music', category: 'media', capabilities: ['music'] },
]

const DEFAULT_MODELS = {
  // 文本 LLM
  deepseek: 'deepseek-v3.2',
  openai: 'gpt-5.4',
  claude: 'claude-sonnet-4-6-20260218',
  qwen: 'qwen3.5-plus',
  kimi: 'kimi-k2.5',
  minimax: 'minimax-m2.5',
  glm: 'glm-5',
  gemini: 'gemini-3.1-pro',
  // 媒体生成
  seedance: 'seedance-1.0',
  google: 'imagen-4',
  jimeng: 'seedream-5.0',
  seedream: 'seedream-5.0',
  haimian: 'haimian-v2',
  suno: 'suno-v5',
  stability: 'stable-diffusion-3.5',
  elevenlabs: 'eleven_multilingual_v2',
  minimax_music: 'music-2.5',
}

// 媒体 Provider 的模型列表
const MEDIA_MODELS = {
  google: [
    { value: 'imagen-4', label: 'Imagen 4 (最新)' },
    { value: 'imagen-4-ultra', label: 'Imagen 4 Ultra (高端版)' },
    { value: 'imagen-3.0-generate-002', label: 'Imagen 3.0 Generate 002' },
    { value: 'imagen-3.0-generate-001', label: 'Imagen 3.0 Generate 001' },
  ],
  jimeng: [
    { value: 'seedream-5.0', label: 'Seedream 5.0 (最新)' },
    { value: 'seedream-5.0-lite', label: 'Seedream 5.0 Lite' },
    { value: 'seedream-4.0', label: 'Seedream 4.0' },
    { value: 'seedance-2.0', label: 'Seedance 2.0 (视频)' },
  ],
  seedance: [
    { value: 'seedance-1.0', label: 'Seedance 1.0 (最新可用)' },
  ],
  seedream: [
    { value: 'seedream-5.0', label: 'Seedream 5.0 (最新)' },
    { value: 'seedream-4.5', label: 'Seedream 4.5' },
    { value: 'seedream-4.0', label: 'Seedream 4.0' },
  ],
  suno: [
    { value: 'suno-v5', label: 'Suno V5 (最新，44.1kHz 立体声)' },
    { value: 'suno-v4.5', label: 'Suno V4.5' },
    { value: 'suno-v4', label: 'Suno V4' },
  ],
  stability: [
    { value: 'stable-diffusion-3.5', label: 'Stable Diffusion 3.5 (最新)' },
    { value: 'stable-diffusion-3', label: 'Stable Diffusion 3' },
    { value: 'stable-diffusion-xl', label: 'Stable Diffusion XL' },
    { value: 'stable-audio-open-1.0', label: 'Stable Audio Open 1.0' },
  ],
  elevenlabs: [
    { value: 'eleven_multilingual_v2', label: 'Eleven Multilingual V2 (28 种语言)' },
    { value: 'eleven_flash_v2.5', label: 'Eleven Flash V2.5' },
    { value: 'eleven_turbo_v2.5', label: 'Eleven Turbo V2.5' },
    { value: 'eleven_monolingual_v1', label: 'Eleven Monolingual V1' },
  ],
  minimax_music: [
    { value: 'music-2.5', label: 'Music 2.5 (最新)' },
    { value: 'music-2.0', label: 'Music 2.0' },
    { value: 'music-01', label: 'Music 01' },
  ],
  haimian: [
    { value: 'haimian-v2', label: '海绵音乐 V2 (最新)' },
    { value: 'haimian-v1', label: '海绵音乐 V1' },
  ],
  minimax: [
    { value: 'video-01', label: 'Video 01' },
  ],
}

// 文本 Provider 的模型列表
const TEXT_MODELS = {
  deepseek: [
    { value: 'deepseek-v3.2', label: 'DeepSeek V3.2 (最新，671B MoE)' },
    { value: 'deepseek-v3.2-speciale', label: 'DeepSeek V3.2 Speciale (长思考增强版)' },
    { value: 'deepseek-v3.1-terminus', label: 'DeepSeek V3.1 Terminus' },
    { value: 'deepseek-v3', label: 'DeepSeek V3' },
    { value: 'deepseek-r1', label: 'DeepSeek R1 (推理模型)' },
  ],
  openai: [
    { value: 'gpt-5.4', label: 'GPT-5.4 (最新)' },
    { value: 'o3', label: 'O3' },
    { value: 'gpt-4o', label: 'GPT-4o' },
    { value: 'gpt-4o-mini', label: 'GPT-4o Mini' },
    { value: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
  ],
  claude: [
    { value: 'claude-sonnet-4-6-20260218', label: 'Claude Sonnet 4.6 (最新)' },
    { value: 'claude-opus-4-6-20260218', label: 'Claude Opus 4.6 (最新)' },
    { value: 'claude-sonnet-4-5-20251101', label: 'Claude Sonnet 4.5' },
    { value: 'claude-opus-4-5-20251101', label: 'Claude Opus 4.5' },
    { value: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4' },
    { value: 'claude-opus-4-20250514', label: 'Claude Opus 4' },
  ],
  qwen: [
    { value: 'qwen3.5-plus', label: 'Qwen3.5 Plus (最新)' },
    { value: 'qwen3.5', label: 'Qwen3.5' },
    { value: 'qwen3-max-thinking', label: 'Qwen3 Max Thinking (推理模型)' },
    { value: 'qwen3-max', label: 'Qwen3 Max' },
    { value: 'qwen3-plus', label: 'Qwen3 Plus' },
    { value: 'qwen-plus', label: 'Qwen Plus' },
    { value: 'qwen-turbo', label: 'Qwen Turbo' },
  ],
  kimi: [
    { value: 'kimi-k2.5', label: 'Kimi K2.5 (最新，万亿参数)' },
    { value: 'kimi-k2-thinking', label: 'Kimi K2 Thinking (推理版)' },
    { value: 'kimi-k2', label: 'Kimi K2' },
    { value: 'kimi-k1.5', label: 'Kimi K1.5' },
  ],
  minimax: [
    { value: 'minimax-m2.5', label: 'MiniMax M2.5 (最新)' },
    { value: 'abab6.5s-chat', label: 'ABAB 6.5s Chat' },
    { value: 'abab6.5t-chat', label: 'ABAB 6.5t Chat' },
    { value: 'abab6.5g-chat', label: 'ABAB 6.5g Chat' },
    { value: 'm2.5', label: 'M2.5 (最新)' },
    { value: 'm2.1', label: 'M2.1 (Coding & Agent)' },
  ],
  glm: [
    { value: 'glm-5', label: 'GLM-5 (最新旗舰，Coding & Agent)' },
    { value: 'glm-4.5-flash', label: 'GLM-4.5 Flash (免费)' },
    { value: 'glm-4-plus', label: 'GLM-4 Plus' },
    { value: 'glm-4-air', label: 'GLM-4 Air' },
    { value: 'glm-4-air-250414', label: 'GLM-4 Air 250414' },
    { value: 'glm-4-flash', label: 'GLM-4 Flash' },
    { value: 'glm-z1-air', label: 'GLM-Z1 Air (推理模型)' },
    { value: 'glm-z1-rumination', label: 'GLM-Z1 Rumination (沉思模型)' },
  ],
  gemini: [
    { value: 'gemini-3.1-pro', label: 'Gemini 3.1 Pro (最新，原生多模态推理)' },
    { value: 'gemini-3-pro', label: 'Gemini 3 Pro' },
    { value: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro' },
    { value: 'gemini-2.0-flash', label: 'Gemini 2.0 Flash (快速响应)' },
    { value: 'gemini-2.0-flash-lite', label: 'Gemini 2.0 Flash Lite (轻量版)' },
  ],
}

// 媒体 Provider 的特殊配置字段
const MEDIA_PROVIDER_FIELDS = {
  minimax: { label: 'Group ID', placeholder: '输入 Group ID' },
  google: { label: 'Project ID', placeholder: '输入 Project ID' },
  jimeng: { label: 'API Secret', placeholder: '输入 API Secret' },
  seedance: { label: 'Base URL', placeholder: 'https://api.302.ai/doubao' },
  seedream: { label: 'Base URL', placeholder: 'https://api.deerapi.com' },
  haimian: { label: 'API Secret', placeholder: '输入 API Secret' },
  suno: { label: 'Base URL', placeholder: 'https://api.sunoapi.org' },
  minimax_music: { label: 'Base URL', placeholder: 'https://api.minimaxi.com' },
}

function ApiConfigModal({ isOpen, onClose, isDarkMode }) {
  const { t } = useI18n()
  const modalRef = useRef(null)

  // 多 API 配置列表
  const [apiConfigs, setApiConfigs] = useState([])
  const [activeTab, setActiveTab] = useState('all') // 'all' | 'text' | 'media'
  const [editingId, setEditingId] = useState('new') // 默认显示编辑表单

  // 当前编辑的配置
  const [currentConfig, setCurrentConfig] = useState({
    id: '',
    provider: 'deepseek',
    api_key: '',
    base_url: '',
    model: '',
    enabled: true,
    is_active: false,
    capabilities: [],
  })

  const [isLoading, setIsLoading] = useState(false)
  const [isVerifying, setIsVerifying] = useState(false)
  const [error, setError] = useState(null)
  const [success, setSuccess] = useState(null)

  // 加载配置
  useEffect(() => {
    if (isOpen) {
      loadConfigs()
    }
  }, [isOpen])
  
  // 当切换标签页时，更新表单的 Provider
  useEffect(() => {
    if (isOpen) {
      const firstProvider = activeTab === 'media' 
        ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
        : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
      const provider = PROVIDERS.find(p => p.value === firstProvider)
      setEditingId('new')
      setCurrentConfig({
        id: `new_${Date.now()}`,
        provider: firstProvider,
        api_key: '',
        base_url: '',
        model: DEFAULT_MODELS[firstProvider],
        enabled: true,
        is_active: apiConfigs.length === 0,
        capabilities: provider?.capabilities || [],
      })
    }
  }, [activeTab, isOpen])

  const loadConfigs = async () => {
    try {
      const invoke = await getInvoke()
      const config = await invoke('get_agent_config')

      if (config && config.user_apis) {
        setApiConfigs(config.user_apis.map(api => ({
          ...api,
          base_url: api.base_url || '',
          enabled: api.is_active !== false,
        })))
      }
    } catch (err) {
      console.warn('加载配置失败:', err)
      // 回退到 localStorage
      const localConfig = await getActiveApiConfig()
      if (localConfig) {
        setApiConfigs([{ ...localConfig, base_url: localConfig.base_url || '', enabled: true }])
      }
    }
    
    // 直接开始编辑（显示 Provider/Model 选择页面）
    const firstProvider = activeTab === 'media' 
      ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
      : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
    const provider = PROVIDERS.find(p => p.value === firstProvider)
    setEditingId('new')
    setCurrentConfig({
      id: `new_${Date.now()}`,
      provider: firstProvider,
      api_key: '',
      base_url: '',
      model: DEFAULT_MODELS[firstProvider],
      enabled: true,
      is_active: apiConfigs.length === 0,
      capabilities: provider?.capabilities || [],
    })
    
    setError(null)
    setSuccess(null)
  }

  // 添加新配置 - 直接开始编辑
  const handleAddNew = () => {
    const firstProvider = activeTab === 'media'
      ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
      : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
    const provider = PROVIDERS.find(p => p.value === firstProvider)

    setEditingId('new')
    setCurrentConfig({
      id: `new_${Date.now()}`,
      provider: firstProvider,
      api_key: '',
      base_url: '',
      model: DEFAULT_MODELS[firstProvider],
      enabled: true,
      is_active: apiConfigs.length === 0,
      capabilities: provider?.capabilities || [],
    })
  }

  // 取消编辑 - 关闭模态框
  const handleCancel = () => {
    setError(null)
    setSuccess(null)
    setEditingId(null)
    onClose()
  }

  // 保存配置
  const handleSave = async () => {
    setError(null)
    setSuccess(null)

    if (!currentConfig.api_key.trim()) {
      setError('API Key 不能为空')
      return
    }

    setIsLoading(true)
    try {
      const invoke = await getInvoke()

      let newConfigs
      if (editingId === 'new') {
        // 添加新配置
        newConfigs = [...apiConfigs, { ...currentConfig, id: `api_${Date.now()}` }]
      } else {
        // 更新现有配置
        newConfigs = apiConfigs.map(c =>
          c.id === editingId ? currentConfig : c
        )
      }

      // 保存到 Tauri
      await invoke('update_agent_config', {
        config: {
          user_apis: newConfigs,
          workers_api: { base_url: '', enabled: false },
          execution_strategy: 'LocalOnly',
          default_provider: newConfigs.find(c => c.is_active)?.provider || newConfigs[0]?.provider || 'deepseek',
        },
      })

      setApiConfigs(newConfigs)
      setSuccess('配置已保存')

      window.dispatchEvent(new CustomEvent('api-config-changed'))

      // 重置为新的编辑状态，继续添加
      const nextProvider = activeTab === 'media'
        ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
        : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
      const provider = PROVIDERS.find(p => p.value === nextProvider)
      setEditingId('new')
      setCurrentConfig({
        id: `new_${Date.now()}`,
        provider: nextProvider,
        api_key: '',
        base_url: '',
        model: DEFAULT_MODELS[nextProvider],
        enabled: true,
        is_active: false,
        capabilities: provider?.capabilities || [],
      })
      
      setTimeout(() => {
        setSuccess(null)
      }, 2000)
    } catch (err) {
      console.error('保存配置失败:', err)
      setError(`保存失败：${err.message || '未知错误'}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 测试连接
  const handleVerify = async () => {
    if (!currentConfig.api_key.trim()) {
      setError('API Key 不能为空')
      return
    }

    setError(null)
    setSuccess(null)
    setIsVerifying(true)

    try {
      const invoke = await getInvoke()
      const result = await invoke('test_api_connection', {
        config: currentConfig,
      })

      if (result?.success) {
        setSuccess('连接测试成功')
      } else {
        setError(result?.message || '连接测试失败')
      }
    } catch (err) {
      setError(`连接测试失败：${err.message}`)
    } finally {
      setIsVerifying(false)
    }
  }

  const handleClose = () => {
    setError(null)
    setSuccess(null)
    setEditingId(null)
    onClose()
  }

  const filteredConfigs = activeTab === 'all'
    ? apiConfigs
    : apiConfigs.filter(c => {
        const provider = PROVIDERS.find(p => p.value === c.provider)
        return provider?.category === activeTab
      })

  const isEditingMedia = currentConfig.provider &&
    PROVIDERS.find(p => p.value === currentConfig.provider)?.category === 'media'

  if (!isOpen) return null

  return (
    <div className="api-config-modal-backdrop">
      <div ref={modalRef} className={`api-config-modal ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="api-config-modal__header">
          <div>
            <h2>{t('apiConfig.title')}</h2>
            <p>支持多 API 配置并存，智能体根据能力自动路由</p>
          </div>
          <button type="button" onClick={handleClose} className="api-config-modal__close">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <div className="api-config-modal__content">
          {/* 标签页切换 */}
          <div className="api-config-tabs">
            <button
              className={`api-config-tab ${activeTab === 'all' ? 'active' : ''}`}
              onClick={() => setActiveTab('all')}
            >
              全部 ({apiConfigs.length})
            </button>
            <button
              className={`api-config-tab ${activeTab === 'text' ? 'active' : ''}`}
              onClick={() => setActiveTab('text')}
            >
              文本 LLM
            </button>
            <button
              className={`api-config-tab ${activeTab === 'media' ? 'active' : ''}`}
              onClick={() => setActiveTab('media')}
            >
              媒体生成
            </button>
          </div>

          {/* Provider/Model 选择表单 */}
          <div className="api-config-form">
              <div className="api-config-field">
                <label>
                  <span>Provider</span>
                  <select
                    value={currentConfig.provider}
                    onChange={(e) => {
                      const newProvider = e.target.value
                      const provider = PROVIDERS.find(p => p.value === newProvider)
                      setCurrentConfig({
                        ...currentConfig,
                        provider: newProvider,
                        model: DEFAULT_MODELS[newProvider] || '',
                        capabilities: provider?.capabilities || [],
                      })
                    }}
                    disabled={isLoading}
                  >
                    {activeTab === 'all' && (
                      <>
                        <optgroup label="文本 LLM">
                          {PROVIDERS.filter(p => p.category === 'text').map(p => (
                            <option key={p.value} value={p.value}>{p.label}</option>
                          ))}
                        </optgroup>
                        <optgroup label="媒体生成">
                          {PROVIDERS.filter(p => p.category === 'media').map(p => (
                            <option key={p.value} value={p.value}>{p.label}</option>
                          ))}
                        </optgroup>
                      </>
                    )}
                    {activeTab === 'text' && (
                      <optgroup label="文本 LLM">
                        {PROVIDERS.filter(p => p.category === 'text').map(p => (
                          <option key={p.value} value={p.value}>{p.label}</option>
                        ))}
                      </optgroup>
                    )}
                    {activeTab === 'media' && (
                      <optgroup label="媒体生成">
                        {PROVIDERS.filter(p => p.category === 'media').map(p => (
                          <option key={p.value} value={p.value}>{p.label}</option>
                        ))}
                      </optgroup>
                    )}
                  </select>
                </label>
              </div>

              {/* API Key */}
              <div className="api-config-field">
                <label>
                  <span>API Key</span>
                  <input
                    type="password"
                    value={currentConfig.api_key}
                    onChange={(e) => setCurrentConfig({ ...currentConfig, api_key: e.target.value })}
                    placeholder="输入 API Key"
                    disabled={isLoading}
                  />
                </label>
              </div>

              {/* Base URL（媒体 Provider 特殊字段） */}
              {isEditingMedia && MEDIA_PROVIDER_FIELDS[currentConfig.provider] && (
                <div className="api-config-field">
                  <label>
                    <span>{MEDIA_PROVIDER_FIELDS[currentConfig.provider].label}</span>
                    <input
                      type="text"
                      value={currentConfig.base_url}
                      onChange={(e) => setCurrentConfig({ ...currentConfig, base_url: e.target.value })}
                      placeholder={MEDIA_PROVIDER_FIELDS[currentConfig.provider].placeholder}
                      disabled={isLoading}
                    />
                  </label>
                </div>
              )}

              {/* Model（仅文本 Provider） */}
              {!isEditingMedia && (
                <div className="api-config-field">
                  <label>
                    <span>Model</span>
                    <div className="api-config-model-selector">
                      <select
                        value={currentConfig.model}
                        onChange={(e) => {
                          if (e.target.value) {
                            setCurrentConfig({ ...currentConfig, model: e.target.value })
                          }
                        }}
                        disabled={isLoading}
                        className="api-config-model-select"
                      >
                        <option value="">选择模型...</option>
                        {TEXT_MODELS[currentConfig.provider]?.map((m) => (
                          <option key={m.value} value={m.value}>{m.label}</option>
                        ))}
                      </select>
                      <input
                        type="text"
                        value={currentConfig.model}
                        onChange={(e) => setCurrentConfig({ ...currentConfig, model: e.target.value })}
                        placeholder="或手动输入模型名称"
                        disabled={isLoading}
                        className="api-config-model-input"
                      />
                    </div>
                  </label>
                  {DEFAULT_MODELS[currentConfig.provider] && (
                    <p className="api-config-hint">
                      默认：{DEFAULT_MODELS[currentConfig.provider]}
                    </p>
                  )}
                </div>
              )}

              {/* 媒体 Provider 的 Model 字段 */}
              {isEditingMedia && (
                <div className="api-config-field">
                  <label>
                    <span>Model</span>
                    <div className="api-config-model-selector">
                      <select
                        value={currentConfig.model}
                        onChange={(e) => {
                          if (e.target.value) {
                            setCurrentConfig({ ...currentConfig, model: e.target.value })
                          }
                        }}
                        disabled={isLoading}
                        className="api-config-model-select"
                      >
                        <option value="">选择模型...</option>
                        {MEDIA_MODELS[currentConfig.provider]?.map((m) => (
                          <option key={m.value} value={m.value}>{m.label}</option>
                        ))}
                      </select>
                      <input
                        type="text"
                        value={currentConfig.model}
                        onChange={(e) => setCurrentConfig({ ...currentConfig, model: e.target.value })}
                        placeholder="或手动输入模型名称"
                        disabled={isLoading}
                        className="api-config-model-input"
                      />
                    </div>
                  </label>
                </div>
              )}

              {/* 能力标签展示 */}
              {currentConfig.capabilities.length > 0 && (
                <div className="api-config-field">
                  <label>
                    <span>支持能力</span>
                    <div className="capability-tags">
                      {currentConfig.capabilities.map(cap => (
                        <span key={cap} className="capability-tag">{cap}</span>
                      ))}
                    </div>
                  </label>
                </div>
              )}

              {/* 测试连接 */}
              <div className="api-config-actions-inline">
                <button
                  type="button"
                  className="api-config-btn-verify"
                  onClick={handleVerify}
                  disabled={isLoading || isVerifying || !currentConfig.api_key.trim()}
                >
                  {isVerifying ? '测试中...' : '测试连接'}
                </button>
              </div>

              {error && <div className="api-config-error">{error}</div>}
              {success && <div className="api-config-success">{success}</div>}

              {/* 保存和取消按钮 */}
              <div className="api-config-modal__actions">
                <button
                  type="button"
                  className="api-config-modal__btn-ghost"
                  onClick={handleCancel}
                  disabled={isLoading}
                >
                  取消
                </button>
                <button
                  type="button"
                  className="api-config-modal__btn-save"
                  onClick={handleSave}
                  disabled={isLoading || isVerifying}
                >
                  {isLoading ? '保存中...' : '保存'}
                </button>
              </div>
            </div>

            <div className="api-config-modal__info">
              <p>🔒 所有 API Key 加密存储在本地</p>
              <p>💡 智能体根据任务类型自动选择有对应能力的 Provider</p>
            </div>
          </div>
        </div>
      </div>
  )
}

export default ApiConfigModal
