import { useState, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import { saveApiConfig, getActiveApiConfig } from '@/hooks/useApiConfig'
import './ApiConfigModal.css'

// Tauri invoke 安全导入
let invokeCache = null;
async function getInvoke() {
  if (!invokeCache) {
    // 尝试直接导入 Tauri，如果失败则使用 Mock
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      // 检查 invoke 是否可用
      if (typeof invoke === 'function') {
        invokeCache = invoke;
        console.log('[ApiConfigModal] ✓ 使用真实 Tauri invoke')
        return invokeCache;
      }
    } catch (error) {
      // 忽略错误，使用 Mock
    }
    
    console.warn('[ApiConfigModal] ⚠️ 使用 MockInvoke (非 Tauri 环境)')
    invokeCache = createMockInvoke();
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
  // 文本 LLM - 使用 API 正确的模型名称
  deepseek: 'deepseek-chat',  // DeepSeek API 使用 deepseek-chat 而不是 deepseek-v3.2
  openai: 'gpt-4o',
  claude: 'claude-3-5-sonnet-20241022',
  qwen: 'qwen-plus',
  kimi: 'moonshot-v1-8k',
  minimax: 'abab6.5s-chat',
  glm: 'glm-4',
  gemini: 'gemini-1.5-pro',
  // 媒体生成
  seedance: 'doubao-seedance-1.0-pro',
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
    { value: 'doubao-seedance-1.0-pro', label: 'Doubao-Seedance 1.0 Pro (标准版)' },
    { value: 'doubao-seedance-1.0-pro-fast', label: 'Doubao-Seedance 1.0 Pro Fast (快速版)' },
    { value: 'doubao-seedance-1.0-lite', label: 'Doubao-Seedance 1.0 Lite (精简版)' },
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
    { value: 'deepseek-chat', label: 'DeepSeek V3 (最新，671B MoE)' },
    { value: 'deepseek-reasoner', label: 'DeepSeek R1 (推理模型)' },
    { value: 'deepseek-coder', label: 'DeepSeek Coder (代码模型)' },
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
  const [isListExpanded, setIsListExpanded] = useState(false) // 列表默认折叠

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
      // 打开模态框时从 localStorage 读取展开状态
      if (typeof localStorage !== 'undefined') {
        const saved = localStorage.getItem('alou-api-config-list-expanded')
        console.log('[ApiConfigModal] 打开模态框，从 localStorage 读取展开状态:', saved)
        setIsListExpanded(saved !== 'false')
      }
    }
  }, [isOpen])

  // 当切换标签页时，更新表单的 Provider（但不重置编辑状态）
  useEffect(() => {
    if (isOpen && editingId === 'new') {
      const firstProvider = activeTab === 'media'
        ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
        : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
      const provider = PROVIDERS.find(p => p.value === firstProvider)
      setCurrentConfig(prev => {
        // 如果当前没有编辑内容或者是新的 Provider 类别，才更新
        if (!prev.provider || prev.api_key === '') {
          return {
            id: prev.id.startsWith('new_') ? prev.id : `new_${Date.now()}`,
            provider: firstProvider,
            api_key: '',
            base_url: '',
            model: DEFAULT_MODELS[firstProvider],
            enabled: true,
            is_active: apiConfigs.length === 0,
            capabilities: provider?.capabilities || [],
          }
        }
        // 否则保持当前编辑状态
        return prev
      })
    }
  }, [activeTab, isOpen])

  // 持久化列表展开状态
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const value = String(isListExpanded)
      localStorage.setItem('alou-api-config-list-expanded', value)
      console.log('[ApiConfigModal] 保存展开状态到 localStorage:', value)
    }
  }, [isListExpanded])

  // 当模态框打开时，降低头像层级
  useEffect(() => {
    if (isOpen) {
      document.body.classList.add('api-config-modal-open')
      // 直接设置头像和标签的隐藏
      const avatars = document.querySelectorAll('.agent-canvas .agent-avatar')
      const labels = document.querySelectorAll('.agent-canvas .agent-label')
      avatars.forEach(avatar => {
        avatar.classList.add('hidden-by-modal')
      })
      labels.forEach(label => {
        label.classList.add('hidden-by-modal')
      })
    } else {
      document.body.classList.remove('api-config-modal-open')
      const avatars = document.querySelectorAll('.agent-canvas .agent-avatar')
      const labels = document.querySelectorAll('.agent-canvas .agent-label')
      avatars.forEach(avatar => {
        avatar.classList.remove('hidden-by-modal')
      })
      labels.forEach(label => {
        label.classList.remove('hidden-by-modal')
      })
    }
    
    return () => {
      document.body.classList.remove('api-config-modal-open')
      const avatars = document.querySelectorAll('.agent-canvas .agent-avatar')
      const labels = document.querySelectorAll('.agent-canvas .agent-label')
      avatars.forEach(avatar => {
        avatar.classList.remove('hidden-by-modal')
      })
      labels.forEach(label => {
        label.classList.remove('hidden-by-modal')
      })
    }
  }, [isOpen])

  const loadConfigs = async () => {
    try {
      const invoke = await getInvoke()
      console.log('[ApiConfigModal] 开始加载配置...')
      
      // 加载文本配置
      const config = await invoke('get_agent_config')
      console.log('[ApiConfigModal] 加载的文本配置:', config)

      if (config && Array.isArray(config.user_apis)) {
        // 过滤掉媒体 Provider（它们应该保存在 media_config.json 中）
        const mediaProviders = ['seedance', 'google', 'jimeng', 'seedream', 'haimian', 'suno', 'stability', 'elevenlabs', 'minimax_music'];
        const textConfigs = config.user_apis.filter(api => !mediaProviders.includes(api.provider));
        const mediaConfigsInWrongPlace = config.user_apis.filter(api => mediaProviders.includes(api.provider));

        // 如果有媒体配置被错误保存到文本配置中，提示用户
        if (mediaConfigsInWrongPlace.length > 0) {
          console.warn('[ApiConfigModal] 发现错误保存的媒体配置:', mediaConfigsInWrongPlace);
          console.log('[ApiConfigModal] 这些配置应该保存在 media_config.json 中，请重新保存');
        }

        const loadedConfigs = textConfigs.map(api => ({
          ...api,
          base_url: api.base_url || '',
          enabled: api.is_active !== false,
        }))
        console.log('[ApiConfigModal] 解析后的文本配置列表:', loadedConfigs)
        setApiConfigs(loadedConfigs)

        // 如果没有配置，显示提示
        if (loadedConfigs.length === 0) {
          console.log('[ApiConfigModal] 暂无文本 API 配置，请添加新的配置')
        }
      } else {
        console.warn('[ApiConfigModal] 没有 user_apis 字段或不是数组', config)
        setApiConfigs([])
      }
      
      // 加载媒体配置
      try {
        const mediaConfig = await invoke('get_media_config')
        console.log('[ApiConfigModal] 加载的媒体配置:', mediaConfig)
        
        if (mediaConfig && mediaConfig.providers) {
          const mediaProvidersList = Object.values(mediaConfig.providers).map((provider: any) => ({
            id: `media_${provider.name}`,
            provider: provider.name,
            api_key: provider.api_key,
            base_url: provider.base_url || '',
            model: provider.model || '',
            enabled: provider.enabled !== false,
            is_active: false,
            capabilities: provider.capabilities || [],
            isMedia: true,
          }))
          console.log('[ApiConfigModal] 解析后的媒体配置列表:', mediaProvidersList)
          
          // 将媒体配置添加到列表中（如果当前在媒体标签页）
          if (activeTab === 'media' || activeTab === 'all') {
            setApiConfigs(prev => {
              const existing = prev.filter(c => !c.isMedia)
              return [...existing, ...mediaProvidersList]
            })
          }
        }
      } catch (err) {
        console.warn('[ApiConfigModal] 加载媒体配置失败:', err)
      }
    } catch (err) {
      console.error('[ApiConfigModal] 加载配置失败:', err)
      // 回退到 localStorage
      const localConfig = await getActiveApiConfig()
      if (localConfig) {
        console.log('[ApiConfigModal] 从 localStorage 回退加载配置:', localConfig)
        setApiConfigs([{ ...localConfig, base_url: localConfig.base_url || '', enabled: true }])
      } else {
        console.log('[ApiConfigModal] 无本地配置，设置为空数组')
        setApiConfigs([])
      }
    }

    // 只在首次打开或没有编辑内容时，初始化表单
    if (editingId === 'new' && (!currentConfig.provider || currentConfig.api_key === '')) {
      const firstProvider = activeTab === 'media'
        ? PROVIDERS.find(p => p.category === 'media')?.value || 'seedance'
        : PROVIDERS.find(p => p.category === 'text')?.value || 'deepseek'
      const provider = PROVIDERS.find(p => p.value === firstProvider)
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

  // 编辑现有配置
  const handleEdit = (config) => {
    setEditingId(config.id)
    setCurrentConfig({
      ...config,
      capabilities: PROVIDERS.find(p => p.value === config.provider)?.capabilities || [],
    })
    setError(null)
    setSuccess(null)
  }

  // 删除配置
  const handleDelete = async (id) => {
    const configToDelete = apiConfigs.find(c => c.id === id)
    if (!configToDelete) return

    if (!window.confirm(`确定要删除 "${configToDelete.provider}" 的配置吗？`)) {
      return
    }

    try {
      const invoke = await getInvoke()
      
      // 判断是否是媒体配置
      if (configToDelete.isMedia) {
        // 删除媒体配置 - 从 media_config.json 中移除
        console.log('[ApiConfigModal] 删除媒体配置:', configToDelete.provider)
        
        // 获取现有媒体配置
        const mediaConfig = await invoke('get_media_config')
        
        // 删除指定的 provider
        delete mediaConfig.providers[configToDelete.provider]
        
        // 保存更新后的配置
        await invoke('update_media_config', { providers: mediaConfig.providers })
        
        // 更新本地状态
        setApiConfigs(prev => prev.filter(c => c.id !== id))
      } else {
        // 删除文本配置
        const newConfigs = apiConfigs.filter(c => c.id !== id && !c.isMedia)
        
        // 修复：将 enabled 映射为 is_active
        const configsForBackend = newConfigs.map(c => ({
          id: c.id,
          provider: c.provider,
          api_key: c.api_key,
          base_url: c.base_url || null,
          model: c.model || null,
          is_active: c.enabled !== false,
        }));

        await invoke('update_agent_config', {
          config: {
            user_apis: configsForBackend,
            workers_api: { base_url: '', enabled: false },
            execution_strategy: 'LocalOnly',
            default_provider: configsForBackend.find(c => c.is_active)?.provider || configsForBackend[0]?.provider || 'deepseek',
          },
        })

        // 重新加载媒体配置并合并
        try {
          const mediaConfig = await invoke('get_media_config')
          const mediaProvidersList = Object.values(mediaConfig.providers || {}).map((provider: any) => ({
            id: `media_${provider.name}`,
            provider: provider.name,
            api_key: provider.api_key,
            base_url: provider.base_url || '',
            model: provider.model || '',
            enabled: provider.enabled !== false,
            is_active: false,
            capabilities: provider.capabilities || [],
            isMedia: true,
          }))
          
          setApiConfigs([...newConfigs, ...mediaProvidersList])
        } catch (err) {
          setApiConfigs(newConfigs)
        }
      }
      
      window.dispatchEvent(new CustomEvent('api-config-changed'))

      // 如果删除的是当前正在编辑的，重置为新增状态
      if (editingId === id) {
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
          is_active: newConfigs.length === 0,
          capabilities: provider?.capabilities || [],
        })
      }
    } catch (err) {
      console.error('删除配置失败:', err)
      alert(`删除失败：${err.message || '未知错误'}`)
    }
  }

  // 设为激活
  const handleSetActive = async (id) => {
    try {
      const invoke = await getInvoke()
      const newConfigs = apiConfigs.map(c => ({
        ...c,
        is_active: c.id === id,
        enabled: c.id === id, // 同步更新 enabled 字段
      }))
      
      // 修复：将 enabled 映射为 is_active
      const configsForBackend = newConfigs.map(c => ({
        id: c.id,
        provider: c.provider,
        api_key: c.api_key,
        base_url: c.base_url || null,
        model: c.model || null,
        is_active: c.id === id,
      }));

      await invoke('update_agent_config', {
        config: {
          user_apis: configsForBackend,
          workers_api: { base_url: '', enabled: false },
          execution_strategy: 'LocalOnly',
          default_provider: configsForBackend.find(c => c.is_active)?.provider || configsForBackend[0]?.provider || 'deepseek',
        },
      })

      setApiConfigs(newConfigs)
      window.dispatchEvent(new CustomEvent('api-config-changed'))
    } catch (err) {
      console.error('设置激活配置失败:', err)
    }
  }

  // 保存配置
  // 判断是否是媒体 Provider
  const isMediaProvider = (provider: string) => {
    const mediaProviders = ['seedance', 'google', 'jimeng', 'seedream', 'haimian', 'suno', 'stability', 'elevenlabs', 'minimax_music'];
    return mediaProviders.includes(provider);
  };

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
      let savedId
      if (editingId === 'new') {
        // 添加新配置
        savedId = `api_${Date.now()}`
        newConfigs = [...apiConfigs, { ...currentConfig, id: savedId }]
      } else {
        // 更新现有配置
        savedId = editingId
        newConfigs = apiConfigs.map(c =>
          c.id === editingId ? currentConfig : c
        )
      }

      // 区分保存：文本 LLM 保存到 agent_config.json，媒体 Provider 保存到 media_config.json
      if (isMediaProvider(currentConfig.provider)) {
        // 媒体 Provider - 保存到 media_config.json
        console.log('[ApiConfigModal] 保存媒体 Provider 配置:', currentConfig.provider);

        // 构建媒体配置
        const capabilityMap = {
          'seedance': ['video'],
          'google': ['image'],
          'jimeng': ['image', 'video'],
          'seedream': ['image'],
          'haimian': ['music'],
          'suno': ['music'],
          'stability': ['image'],
          'elevenlabs': ['tts'],
          'minimax_music': ['music'],
        };

        const providers = {
          [currentConfig.provider]: {
            name: currentConfig.provider,
            api_key: currentConfig.api_key.trim(),
            base_url: currentConfig.base_url || null,
            model: currentConfig.model || null,
            enabled: true,
            capabilities: capabilityMap[currentConfig.provider] || [],
            config: null,
          }
        };

        console.log('[ApiConfigModal] 发送媒体配置到后端:', providers);
        
        try {
          await invoke('update_media_config', { providers });
          console.log('[ApiConfigModal] 媒体配置保存成功');
          setSuccess('媒体配置已保存');
        } catch (err) {
          console.error('[ApiConfigModal] 媒体配置保存失败:', err);
          throw err;
        }
      } else {
        // 文本 LLM - 保存到 agent_config.json
        // 修复：将前端的 enabled 字段映射回后端的 is_active 字段
        const configsForBackend = newConfigs.map(c => ({
          id: c.id,
          provider: c.provider,
          api_key: c.api_key,
          base_url: c.base_url || null,
          model: c.model || null,
          is_active: c.enabled !== false, // 将 enabled 映射为 is_active
        }));
        
        console.log('[ApiConfigModal] 保存文本配置到后端:', configsForBackend);
        
        await invoke('update_agent_config', {
          config: {
            user_apis: configsForBackend,
            workers_api: { base_url: '', enabled: false },
            execution_strategy: 'LocalOnly',
            default_provider: configsForBackend.find(c => c.is_active)?.provider || configsForBackend[0]?.provider || 'deepseek',
          },
        });
        setSuccess('配置已保存');
      }

      setApiConfigs(newConfigs)

      window.dispatchEvent(new CustomEvent('api-config-changed'))

      // 保存后停留在编辑状态，显示刚保存的配置，方便用户继续编辑
      setEditingId(savedId)
      setCurrentConfig(newConfigs.find(c => c.id === savedId))

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
      const isMediaProvider = PROVIDERS.find(p => p.value === currentConfig.provider)?.category === 'media'
      
      let result
      if (isMediaProvider) {
        // 媒体 Provider - 调用专门的测试接口
        result = await invoke('test_media_provider_connection', {
          providerName: currentConfig.provider,
          config: {
            name: currentConfig.provider,
            api_key: currentConfig.api_key.trim(),
            base_url: currentConfig.base_url || null,
            model: currentConfig.model || null,
            enabled: true,
            capabilities: [],
            config: null,
          },
        })
      } else {
        // LLM Provider - 使用通用测试接口
        result = await invoke('test_api_connection', {
          config: currentConfig,
        })
      }

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

          {/* 已添加的 API 列表 */}
          <div className="api-config-list">
            <div
              className="api-config-list-header"
              onClick={() => setIsListExpanded(!isListExpanded)}
            >
              <span>已添加的 API 配置 ({filteredConfigs.length})</span>
              <button
                type="button"
                className="api-config-list-toggle"
                title={isListExpanded ? '折叠' : '展开'}
              >
                <svg
                  viewBox="0 0 16 16"
                  className={`toggle-icon ${isListExpanded ? 'expanded' : ''}`}
                >
                  <path
                    d="M4 6l4 4 4-4"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
              </button>
            </div>
            {isListExpanded && (
              <div className="api-config-list-items">
                {filteredConfigs.length === 0 ? (
                  <div className="api-config-empty">
                    <p>暂无 API 配置</p>
                    <p className="api-config-empty-hint">点击下方标签页切换，添加新的 API 配置</p>
                    <button
                      type="button"
                      className="api-config-test-btn"
                      onClick={async () => {
                        const invoke = await getInvoke()
                        try {
                          const config = await invoke('get_agent_config')
                          console.log('[测试按钮] 配置详情:', config)
                          alert(`配置加载成功！\nuser_apis: ${JSON.stringify(config?.user_apis || 'undefined', null, 2)}`)
                        } catch (err) {
                          console.error('[测试按钮] 失败:', err)
                          alert(`获取配置失败：${err}`)
                        }
                      }}
                    >
                      测试配置加载
                    </button>
                    <button
                      type="button"
                      className="api-config-test-btn"
                      style={{marginTop: '10px', background: '#ff9800'}}
                      onClick={async () => {
                        const invoke = await getInvoke()
                        try {
                          const config = await invoke('get_agent_config')
                          const mediaProviders = ['seedance', 'google', 'jimeng', 'seedream', 'haimian', 'suno', 'stability', 'elevenlabs', 'minimax_music'];
                          const mediaConfigsInWrongPlace = config.user_apis?.filter(api => mediaProviders.includes(api.provider)) || [];
                          
                          if (mediaConfigsInWrongPlace.length === 0) {
                            alert('没有发现需要迁移的配置')
                            return
                          }
                          
                          // 构建媒体配置
                          const capabilityMap = {
                            'seedance': ['video'],
                            'google': ['image'],
                            'jimeng': ['image', 'video'],
                            'seedream': ['image'],
                            'haimian': ['music'],
                            'suno': ['music'],
                            'stability': ['image'],
                            'elevenlabs': ['tts'],
                            'minimax_music': ['music'],
                          };
                          
                          const mediaConfig = {}
                          mediaConfigsInWrongPlace.forEach(api => {
                            mediaConfig[api.provider] = {
                              name: api.provider,
                              api_key: api.api_key,
                              base_url: api.base_url || null,
                              model: api.model || null,
                              enabled: true,
                              capabilities: capabilityMap[api.provider] || [],
                              config: null,
                            }
                          })
                          
                          await invoke('update_media_config', { providers: mediaConfig })
                          
                          // 删除错误配置
                          const textConfigs = config.user_apis.filter(api => !mediaProviders.includes(api.provider))
                          await invoke('update_agent_config', {
                            config: {
                              user_apis: textConfigs,
                              workers_api: { base_url: '', enabled: false },
                              execution_strategy: 'LocalOnly',
                              default_provider: textConfigs.find(c => c.is_active)?.provider || textConfigs[0]?.provider || 'deepseek',
                            }
                          })
                          
                          alert(`已迁移 ${mediaConfigsInWrongPlace.length} 个媒体配置到 media_config.json\n请重新打开模态框查看`)
                          window.location.reload()
                        } catch (err) {
                          console.error('[迁移按钮] 失败:', err)
                          alert(`迁移失败：${err}`)
                        }
                      }}
                    >
                      迁移媒体配置到正确位置
                    </button>
                  </div>
                ) : (
                  filteredConfigs.map((config) => {
                    const provider = PROVIDERS.find(p => p.value === config.provider)
                    const isEditing = editingId === config.id
                    return (
                      <div
                        key={config.id}
                        className={`api-config-list-item ${isEditing ? 'editing' : ''} ${config.is_active ? 'active' : ''}`}
                      >
                        <div className="api-config-item-info">
                          <span className="api-config-item-provider">
                            {provider?.label || config.provider}
                            {config.is_active && <span className="api-config-item-active-tag">（当前使用）</span>}
                          </span>
                          <span className="api-config-item-model">{config.model || '默认模型'}</span>
                          <span className="api-config-item-key">{config.api_key?.slice(0, 8)}...{config.api_key?.slice(-4)}</span>
                        </div>
                        <div className="api-config-item-actions">
                          <button
                            type="button"
                            className="api-config-item-btn"
                            onClick={() => handleEdit(config)}
                            title="编辑"
                          >
                            编辑
                          </button>
                          {!config.is_active && (
                            <button
                              type="button"
                              className="api-config-item-btn"
                              onClick={() => handleSetActive(config.id)}
                              title="设为激活"
                            >
                              激活
                            </button>
                          )}
                          <button
                            type="button"
                            className="api-config-item-btn delete"
                            onClick={() => handleDelete(config.id)}
                            title="删除"
                          >
                            删除
                          </button>
                        </div>
                      </div>
                    )
                  })
                )}
              </div>
            )}
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
                    <div className="api-config-model-combined">
                      <select
                        value={currentConfig.model}
                        onChange={(e) => {
                          if (e.target.value) {
                            setCurrentConfig({ ...currentConfig, model: e.target.value })
                          }
                        }}
                        disabled={isLoading}
                        className="api-config-model-select-combined"
                      >
                        <option value="">选择或输入模型名称...</option>
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
                        className="api-config-model-input-combined"
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
                    <div className="api-config-model-combined">
                      <select
                        value={currentConfig.model}
                        onChange={(e) => {
                          if (e.target.value) {
                            setCurrentConfig({ ...currentConfig, model: e.target.value })
                          }
                        }}
                        disabled={isLoading}
                        className="api-config-model-select-combined"
                      >
                        <option value="">选择或输入模型名称...</option>
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
                        className="api-config-model-input-combined"
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
