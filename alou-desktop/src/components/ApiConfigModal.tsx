import { useState, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
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
  // 媒体生成
  { value: 'minimax', label: 'MiniMax', category: 'media', capabilities: ['tts', 'video'] },
  { value: 'google', label: 'Google Imagen', category: 'media', capabilities: ['image'] },
  { value: 'jimeng', label: '即梦', category: 'media', capabilities: ['image', 'video'] },
  { value: 'seedance', label: 'Seedance', category: 'media', capabilities: ['video'] },
  { value: 'seedream', label: 'Seedream', category: 'media', capabilities: ['image'] },
  { value: 'haimian', label: '海绵音乐', category: 'media', capabilities: ['music'] },
  { value: 'stability', label: 'Stability AI', category: 'media', capabilities: ['image'] },
  { value: 'elevenlabs', label: 'ElevenLabs', category: 'media', capabilities: ['tts'] },
]

const DEFAULT_MODELS = {
  deepseek: 'deepseek-chat',
  openai: 'gpt-4o',
  claude: 'claude-3-5-sonnet-20241022',
  qwen: 'qwen-plus',
  kimi: 'moonshot-v1-8k',
}

// 媒体 Provider 的特殊配置字段
const MEDIA_PROVIDER_FIELDS = {
  minimax: { label: 'Group ID', placeholder: '输入 Group ID' },
  google: { label: 'Project ID', placeholder: '输入 Project ID' },
  jimeng: { label: 'API Secret', placeholder: '输入 API Secret' },
  seedance: { label: 'Base URL', placeholder: 'https://api.302.ai/doubao' },
  seedream: { label: 'Base URL', placeholder: 'https://api.deerapi.com' },
  haimian: { label: 'API Secret', placeholder: '输入 API Secret' },
}

function ApiConfigModal({ isOpen, onClose, isDarkMode }) {
  const { t } = useI18n()
  const modalRef = useRef(null)

  // 多 API 配置列表
  const [apiConfigs, setApiConfigs] = useState([])
  const [activeTab, setActiveTab] = useState('all') // 'all' | 'text' | 'media'
  const [editingId, setEditingId] = useState(null)
  const [expandedConfig, setExpandedConfig] = useState(null)

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
    setError(null)
    setSuccess(null)
  }

  // 添加新配置
  const handleAddNew = () => {
    setEditingId('new')
    setCurrentConfig({
      id: `new_${Date.now()}`,
      provider: 'deepseek',
      api_key: '',
      base_url: '',
      model: DEFAULT_MODELS.deepseek,
      enabled: true,
      is_active: apiConfigs.length === 0,
      capabilities: ['text'],
    })
    setExpandedConfig(null)
  }

  // 编辑现有配置
  const handleEdit = (config) => {
    setEditingId(config.id)
    setCurrentConfig({
      ...config,
      base_url: config.base_url || '',
    })
    setExpandedConfig(null)
  }

  // 取消编辑
  const handleCancel = () => {
    setEditingId(null)
    setCurrentConfig({
      id: '',
      provider: 'deepseek',
      api_key: '',
      base_url: '',
      model: '',
      enabled: true,
      is_active: false,
      capabilities: [],
    })
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
      setEditingId(null)
      setSuccess('配置已保存')
      
      window.dispatchEvent(new CustomEvent('api-config-changed'))
      
      setTimeout(() => {
        handleClose()
      }, 1000)
    } catch (err) {
      console.error('保存配置失败:', err)
      setError(`保存失败：${err.message || '未知错误'}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 删除配置
  const handleDelete = async (id) => {
    if (!confirm('确定要删除此配置吗？')) return

    try {
      const invoke = await getInvoke()
      const newConfigs = apiConfigs.filter(c => c.id !== id)
      
      await invoke('update_agent_config', {
        config: {
          user_apis: newConfigs,
          workers_api: { base_url: '', enabled: false },
          execution_strategy: 'LocalOnly',
          default_provider: newConfigs[0]?.provider || 'deepseek',
        },
      })

      setApiConfigs(newConfigs)
      if (editingId === id) handleCancel()
    } catch (err) {
      setError(`删除失败：${err.message}`)
    }
  }

  // 设置激活状态
  const handleSetActive = async (id) => {
    try {
      const invoke = await getInvoke()
      const newConfigs = apiConfigs.map(c => ({
        ...c,
        is_active: c.id === id,
      }))

      await invoke('update_agent_config', {
        config: {
          user_apis: newConfigs,
          workers_api: { base_url: '', enabled: false },
          execution_strategy: 'LocalOnly',
          default_provider: newConfigs.find(c => c.is_active)?.provider || 'deepseek',
        },
      })

      setApiConfigs(newConfigs)
      window.dispatchEvent(new CustomEvent('api-config-changed'))
    } catch (err) {
      setError(`设置失败：${err.message}`)
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

  // 切换展开/收起
  const toggleExpand = (id) => {
    setExpandedConfig(expandedConfig === id ? null : id)
  }

  const handleClose = () => {
    setError(null)
    setSuccess(null)
    setEditingId(null)
    setExpandedConfig(null)
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
            <button className="api-config-tab api-config-tab-add" onClick={handleAddNew}>
              + 添加配置
            </button>
          </div>

          {/* 配置列表 */}
          {editingId === null && (
            <div className="api-config-list">
              {filteredConfigs.length === 0 ? (
                <div className="api-config-empty">
                  <p>暂无配置</p>
                  <button onClick={handleAddNew}>添加第一个配置</button>
                </div>
              ) : (
                filteredConfigs.map((config) => {
                  const provider = PROVIDERS.find(p => p.value === config.provider)
                  const isExpanded = expandedConfig === config.id
                  const isActive = config.is_active

                  return (
                    <div key={config.id} className={`api-config-item ${isActive ? 'active' : ''}`}>
                      <div className="api-config-item-header" onClick={() => toggleExpand(config.id)}>
                        <div className="api-config-item-info">
                          <span className="api-config-provider-icon">
                            {provider?.category === 'media' ? '🎨' : '💬'}
                          </span>
                          <div>
                            <div className="api-config-item-name">
                              {provider?.label || config.provider}
                              {isActive && <span className="api-config-active-badge">使用中</span>}
                            </div>
                            <div className="api-config-item-meta">
                              {config.model && <span>模型：{config.model}</span>}
                              {provider?.capabilities && (
                                <span className="api-config-capabilities">
                                  {provider.capabilities.map(cap => (
                                    <span key={cap} className="capability-tag">{cap}</span>
                                  ))}
                                </span>
                              )}
                            </div>
                          </div>
                        </div>
                        <div className="api-config-item-actions">
                          <button
                            className="api-config-btn-icon"
                            onClick={(e) => { e.stopPropagation(); handleSetActive(config.id); }}
                            title={isActive ? '取消激活' : '设为激活'}
                          >
                            {isActive ? '✅' : '⭕'}
                          </button>
                          <button
                            className="api-config-btn-icon"
                            onClick={(e) => { e.stopPropagation(); handleEdit(config); }}
                            title="编辑"
                          >
                            ✏️
                          </button>
                          <button
                            className="api-config-btn-icon"
                            onClick={(e) => { e.stopPropagation(); handleDelete(config.id); }}
                            title="删除"
                          >
                            🗑️
                          </button>
                          <button className="api-config-expand-icon">
                            {isExpanded ? '▲' : '▼'}
                          </button>
                        </div>
                      </div>
                      
                      {isExpanded && (
                        <div className="api-config-item-detail">
                          <div className="api-config-detail-row">
                            <strong>API Key:</strong> {config.api_key ? '••••••' + config.api_key.slice(-4) : '未设置'}
                          </div>
                          {config.base_url && (
                            <div className="api-config-detail-row">
                              <strong>Base URL:</strong> {config.base_url}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  )
                })
              )}
            </div>
          )}

          {/* 编辑/添加配置表单 */}
          {editingId !== null && (
            <div className="api-config-form">
              {/* Provider 选择 */}
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
                    <input
                      type="text"
                      value={currentConfig.model}
                      onChange={(e) => setCurrentConfig({ ...currentConfig, model: e.target.value })}
                      placeholder="输入模型名称"
                      disabled={isLoading}
                    />
                  </label>
                  {DEFAULT_MODELS[currentConfig.provider] && (
                    <p className="api-config-hint">
                      默认：{DEFAULT_MODELS[currentConfig.provider]}
                    </p>
                  )}
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
            </div>
          )}

          {/* 保存/取消按钮 */}
          {editingId !== null && (
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
          )}

          {editingId === null && (
            <div className="api-config-modal__info">
              <p>🔒 所有 API Key 加密存储在本地</p>
              <p>💡 智能体根据任务类型自动选择有对应能力的 Provider</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default ApiConfigModal
