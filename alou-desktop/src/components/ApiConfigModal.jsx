import { useState, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import { invoke } from '@tauri-apps/api/core'
import { saveApiConfig, getActiveApiConfig } from '@/hooks/useApiConfig'
import './ApiConfigModal.css'

const PROVIDERS = [
  { value: 'deepseek', label: 'DeepSeek' },
  { value: 'openai', label: 'OpenAI' },
  { value: 'claude', label: 'Claude (Anthropic)' },
  { value: 'qwen', label: 'Qwen' },
  { value: 'kimi', label: 'Kimi' },
]

const DEFAULT_MODELS = {
  deepseek: 'deepseek-chat',
  openai: 'gpt-4o',
  claude: 'claude-3-5-sonnet-20241022',
  qwen: 'qwen-plus',
  kimi: 'moonshot-v1-8k',
}

function ApiConfigModal({ isOpen, onClose, isDarkMode }) {
  const { t } = useI18n()
  const [apiKey, setApiKey] = useState('')
  const [showApiKey, setShowApiKey] = useState(false)
  const [provider, setProvider] = useState('deepseek')
  const [model, setModel] = useState(DEFAULT_MODELS.deepseek)
  const [baseUrl, setBaseUrl] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [isVerifying, setIsVerifying] = useState(false)
  const [error, setError] = useState(null)
  const [success, setSuccess] = useState(null)
  const modalRef = useRef(null)

  // 从 Tauri 本地存储加载配置（不需要 Workers 后端）
  useEffect(() => {
    if (isOpen) {
      const loadConfig = async () => {
        try {
          const config = await getActiveApiConfig()
          if (config) {
            setApiKey(config.api_key || '')
            setProvider(config.provider || 'deepseek')
            setModel(config.model || DEFAULT_MODELS[config.provider] || 'deepseek-chat')
            setBaseUrl(config.base_url || '')
          } else {
            // 回退到 localStorage
            setApiKey(localStorage.getItem('user_api_key') || '')
            const p = localStorage.getItem('ai_provider') || 'deepseek'
            setProvider(p)
            setModel(localStorage.getItem('ai_model') || DEFAULT_MODELS[p] || 'deepseek-chat')
            setBaseUrl('')
          }
        } catch (err) {
          console.warn('[ApiConfigModal] 加载配置失败，使用 localStorage:', err)
          setApiKey(localStorage.getItem('user_api_key') || '')
          const p = localStorage.getItem('ai_provider') || 'deepseek'
          setProvider(p)
          setModel(localStorage.getItem('ai_model') || DEFAULT_MODELS[p] || 'deepseek-chat')
        }
        setError(null)
        setSuccess(null)
      }
      loadConfig()
    }
  }, [isOpen])

  // 当 provider 改变时，更新默认 model
  useEffect(() => {
    if (DEFAULT_MODELS[provider]) {
      setModel(DEFAULT_MODELS[provider])
    }
  }, [provider])

  // 点击外部关闭
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (modalRef.current && !modalRef.current.contains(event.target)) {
        handleClose()
      }
    }
    const handleEscape = (event) => {
      if (event.key === 'Escape') handleClose()
    }
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
      document.addEventListener('keydown', handleEscape)
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [isOpen])

  const handleSave = async () => {
    setError(null)
    setSuccess(null)

    if (!apiKey.trim()) {
      setError(t('apiConfig.error.apiKeyRequired'))
      return
    }
    if (!model.trim()) {
      setError(t('apiConfig.error.modelRequired'))
      return
    }

    setIsLoading(true)
    try {
      // 通过 Tauri 加密保存到本地文件（同时写入 localStorage 作为备份）
      await saveApiConfig(apiKey.trim(), provider, model.trim(), baseUrl.trim() || undefined)

      // 通知其他组件配置已更新
      window.dispatchEvent(new CustomEvent('api-config-changed'))

      setSuccess(t('apiConfig.success.saved'))
      setTimeout(() => handleClose(), 1000)
    } catch (err) {
      console.error('[ApiConfigModal] 保存配置失败:', err)
      setError(t('apiConfig.error.saveFailed') + (err?.message ? `: ${err.message}` : ''))
    } finally {
      setIsLoading(false)
    }
  }

  const handleVerify = async () => {
    if (!apiKey.trim()) {
      setError(t('apiConfig.error.apiKeyRequired'))
      return
    }

    setError(null)
    setSuccess(null)
    setIsVerifying(true)

    try {
      // 通过 Tauri invoke 直接测试 API 连接，无需 Workers 后端
      const result = await invoke('test_api_connection', {
        config: {
          id: 'test',
          provider,
          api_key: apiKey.trim(),
          base_url: baseUrl.trim() || null,
          model: model.trim(),
          is_active: true,
        },
      })

      if (result?.success) {
        setSuccess(t('apiConfig.success.verified'))
      } else {
        setError(result?.message || t('apiConfig.error.verifyFailed'))
      }
    } catch (err) {
      console.error('[ApiConfigModal] 验证 API Key 失败:', err)
      setError(err?.message || t('apiConfig.error.networkError'))
    } finally {
      setIsVerifying(false)
    }
  }

  const handleClose = () => {
    setError(null)
    setSuccess(null)
    onClose()
  }

  if (!isOpen) return null

  return (
    <div className="api-config-modal-backdrop">
      <div ref={modalRef} className={`api-config-modal ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="api-config-modal__header">
          <div>
            <h2>{t('apiConfig.title')}</h2>
            <p>{t('apiConfig.subtitle')}</p>
          </div>
          <button type="button" onClick={handleClose} className="api-config-modal__close">
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <div className="api-config-modal__content">
          {/* API Key */}
          <div className="api-config-modal__field">
            <label>
              <span>{t('apiConfig.apiKey.label')}</span>
              <div className="api-config-modal__input-group">
                <input
                  type={showApiKey ? 'text' : 'password'}
                  placeholder={t('apiConfig.apiKey.placeholder')}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  disabled={isLoading}
                />
                <button
                  type="button"
                  className="api-config-modal__toggle-visibility"
                  onClick={() => setShowApiKey(!showApiKey)}
                  title={showApiKey ? t('apiConfig.apiKey.hide') : t('apiConfig.apiKey.show')}
                >
                  {showApiKey ? '👁️' : '👁️‍🗨️'}
                </button>
              </div>
            </label>
            <p className="api-config-modal__hint">
              ⚠️ {t('apiConfig.apiKey.warning')}
            </p>
          </div>

          {/* Provider */}
          <div className="api-config-modal__field">
            <label>
              <span>{t('apiConfig.provider.label')}</span>
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
                disabled={isLoading}
              >
                {PROVIDERS.map((p) => (
                  <option key={p.value} value={p.value}>
                    {p.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          {/* Model */}
          <div className="api-config-modal__field">
            <label>
              <span>{t('apiConfig.model.label')}</span>
              <input
                type="text"
                placeholder={t('apiConfig.model.placeholder')}
                value={model}
                onChange={(e) => setModel(e.target.value)}
                disabled={isLoading}
              />
            </label>
            <p className="api-config-modal__hint">
              {t('apiConfig.model.default')}: {DEFAULT_MODELS[provider] || t('apiConfig.model.notSet')}
            </p>
          </div>

          {/* Base URL（可选，用于自定义 API 端点） */}
          <div className="api-config-modal__field">
            <label>
              <span>Base URL <span style={{ fontWeight: 'normal', opacity: 0.6 }}>(可选)</span></span>
              <input
                type="text"
                placeholder="https://api.example.com/v1 (留空使用官方端点)"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                disabled={isLoading}
              />
            </label>
            <p className="api-config-modal__hint">
              💡 配置后可直接使用工具和自主循环，无需启动 Workers 后端
            </p>
          </div>

          {error && <div className="api-config-modal__error">{error}</div>}
          {success && <div className="api-config-modal__success">{success}</div>}

          <div className="api-config-modal__info">
            <p>🔒 API Key 加密存储在本地，不上传到任何服务器</p>
          </div>
        </div>

        <div className="api-config-modal__actions">
          <button
            type="button"
            className="api-config-modal__btn-ghost"
            onClick={handleClose}
            disabled={isLoading}
          >
            {t('apiConfig.cancel')}
          </button>
          <button
            type="button"
            className="api-config-modal__btn-verify"
            onClick={handleVerify}
            disabled={isLoading || isVerifying || !apiKey.trim()}
          >
            {isVerifying ? t('apiConfig.verifying') : t('apiConfig.testConnection')}
          </button>
          <button
            type="button"
            className="api-config-modal__btn-save"
            onClick={handleSave}
            disabled={isLoading || isVerifying}
          >
            {isLoading ? t('apiConfig.saving') : t('apiConfig.save')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default ApiConfigModal
