import { useState, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import { apiService } from '@/services/api'
import useAuthStore from '@/stores/authStore'
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
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const [apiKey, setApiKey] = useState('')
  const [showApiKey, setShowApiKey] = useState(false)
  const [provider, setProvider] = useState('deepseek')
  const [model, setModel] = useState(DEFAULT_MODELS.deepseek)
  const [isLoading, setIsLoading] = useState(false)
  const [isVerifying, setIsVerifying] = useState(false)
  const [error, setError] = useState(null)
  const [success, setSuccess] = useState(null)
  const modalRef = useRef(null)

  // 从 localStorage 加载配置，如果已认证则尝试从后端加载
  useEffect(() => {
    if (isOpen) {
      const loadConfig = async () => {
        let storedApiKey = localStorage.getItem('user_api_key') || ''
        let storedProvider = localStorage.getItem('ai_provider') || 'deepseek'
        let storedModel = localStorage.getItem('ai_model') || DEFAULT_MODELS[storedProvider] || 'deepseek-chat'

        // 如果用户已认证，尝试从后端获取配置
        if (isAuthenticated) {
          try {
            const result = await apiService.getApiConfig()
            if (result.success && result.data) {
              // 后端只返回是否有 API key，不返回实际的 key
              // 所以如果后端说有 key，我们保留本地的 key
              if (!result.data.has_api_key) {
                // 后端没有保存 key，清空本地 key
                storedApiKey = ''
                localStorage.removeItem('user_api_key')
              }
              // 使用后端的 provider 和 model
              storedProvider = result.data.provider || storedProvider
              storedModel = result.data.model || storedModel
            }
          } catch (err) {
            console.warn('[ApiConfigModal] 从后端加载配置失败，使用本地配置:', err)
          }
        }

        setApiKey(storedApiKey)
        setProvider(storedProvider)
        setModel(storedModel)
        setError(null)
        setSuccess(null)
      }

      loadConfig()
    }
  }, [isOpen, isAuthenticated])

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
      if (event.key === 'Escape') {
        handleClose()
      }
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
      // 先保存到 localStorage（前端备用）
      localStorage.setItem('user_api_key', apiKey.trim())
      localStorage.setItem('ai_provider', provider)
      localStorage.setItem('ai_model', model.trim())

      // 只有在用户已认证时才尝试保存到后端
      if (isAuthenticated) {
        try {
          const result = await apiService.saveApiConfig(
            apiKey.trim(),
            provider,
            model.trim()
          )
          
          if (!result.success) {
            console.warn('[ApiConfigModal] 后端保存失败，但本地配置已保存:', result.error)
            // 继续显示成功消息，因为本地保存成功了
          }
        } catch (backendErr) {
          console.warn('[ApiConfigModal] 后端保存失败，但本地配置已保存:', backendErr)
          // 继续显示成功消息，因为本地保存成功了
        }
      } else {
        console.log('[ApiConfigModal] 用户未认证，仅保存到本地存储')
      }

      setSuccess(t('apiConfig.success.saved'))
      setTimeout(() => {
        handleClose()
      }, 1000)
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
      // 使用新的 API 服务测试连接
      const result = await apiService.testApiConnection(
        apiKey.trim(),
        provider,
        model.trim()
      )

      if (result.success) {
        if (result.data?.valid) {
          setSuccess(t('apiConfig.success.verified'))
        } else {
          setError(result.data?.error || t('apiConfig.error.verifyFailed'))
        }
      } else {
        // 处理不同的错误情况
        if (result.error?.includes('验证端点未实现')) {
          setSuccess(t('apiConfig.success.verifyNotEnabled'))
        } else if (result.error?.includes('无法连接到后端服务器')) {
          setError(t('apiConfig.error.networkError'))
        } else {
          setError(result.error || t('apiConfig.error.verifyFailed'))
        }
      }
    } catch (err) {
      console.error('[ApiConfigModal] 验证 API Key 失败:', err)
      setError(err.message || t('apiConfig.error.networkError'))
    } finally {
      setIsVerifying(false)
    }
  }

  const handleClose = () => {
    setError(null)
    setSuccess(null)
    onClose()
  }

  if (!isOpen) {
    return null
  }

  return (
    <div className="api-config-modal-backdrop">
      <div ref={modalRef} className={`api-config-modal ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="api-config-modal__header">
          <div>
            <h2>{t('apiConfig.title')}</h2>
            <p>{t('apiConfig.subtitle')}</p>
          </div>
          <button 
            type="button" 
            onClick={handleClose} 
            className="api-config-modal__close"
          >
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <div className="api-config-modal__content">
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

          {error && <div className="api-config-modal__error">{error}</div>}
          {success && <div className="api-config-modal__success">{success}</div>}
          
          {!isAuthenticated && (
            <div className="api-config-modal__info">
              <p>💡 {t('apiConfig.info.connectWallet')}</p>
              <p className="api-config-modal__info-detail">
                {t('apiConfig.info.connectWalletDetail')}
              </p>
            </div>
          )}
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

