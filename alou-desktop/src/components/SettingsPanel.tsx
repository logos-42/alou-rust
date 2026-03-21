import React, { useState, useRef, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { useI18n } from '@/hooks/useI18n'
import { invoke } from '@tauri-apps/api/core'
import CloseIcon from '@/assets/关闭0.3.png'
import { blurImage } from '@/utils/imageBlur'
import HeartbeatPanel from './Heartbeat/HeartbeatPanel'
import './SettingsPanel.css'

const SettingsPanel = ({ isDarkMode, onToggleTheme, onClose, onBackgroundChange, isSidebarCollapsed = false, activeChannelId = null, onOpenApiConfig }) => {
  const { t } = useI18n()
  const panelRef = useRef(null)
  const fileInputRef = useRef(null)

  // 根据 activeChannelId 获取存储键名
  const getStorageKey = (channelId) => {
    if (!channelId) return 'alou-chat-background'
    return `alou-chat-background-${channelId}`
  }

  const [backgroundImage, setBackgroundImage] = useState(() => {
    if (typeof localStorage !== 'undefined' && activeChannelId) {
      return localStorage.getItem(getStorageKey(activeChannelId)) || ''
    }
    return ''
  })

  // Gateway 相关状态
  const [gatewayConfig, setGatewayConfig] = useState(null)
  const [gatewayStatus, setGatewayStatus] = useState(null)
  const [gatewayLoading, setGatewayLoading] = useState(false)
  const [gatewaySaving, setGatewaySaving] = useState(false)
  const [selectedPlatform, setSelectedPlatform] = useState('telegram')
  const [showGatewaySection, setShowGatewaySection] = useState(false)

  // 平台图标
  const platformIcons = {
    telegram: '📱',
    feishu: '📧',
    discord: '💬',
    qq: '🐧',
  }

  // 心跳相关状态
  const [showHeartbeatPanel, setShowHeartbeatPanel] = useState(false)
  const [heartbeatState, setHeartbeatState] = useState<HeartbeatState | null>(null)
  const [heartbeatConfig, setHeartbeatConfig] = useState<HeartbeatConfig | null>(null)

  // 加载 Gateway 配置
  const loadGatewayConfig = useCallback(async () => {
    try {
      const [configData, statusData] = await Promise.all([
        invoke('get_bot_gateway_config'),
        invoke('get_bot_gateway_status'),
      ])
      setGatewayConfig(configData || null)
      setGatewayStatus(statusData || null)
    } catch (error) {
      console.error('加载 Gateway 配置失败:', error)
      setGatewayConfig(null)
      setGatewayStatus(null)
    }
  }, [])

  // 当打开 Gateway 部分时加载配置
  useEffect(() => {
    if (showGatewaySection && !gatewayConfig && !gatewayLoading) {
      setGatewayLoading(true)
      loadGatewayConfig().finally(() => {
        setGatewayLoading(false)
      })
    }
  }, [showGatewaySection])

  // 当 activeChannelId 变化时，重新加载对应的背景
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const storageKey = getStorageKey(activeChannelId)
      const stored = localStorage.getItem(storageKey) || ''
      // eslint-disable-next-line react-hooks/setState-in-effect
      setBackgroundImage(stored)
    }
  }, [activeChannelId, getStorageKey])

  // 点击外部关闭面板
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (panelRef.current && !panelRef.current.contains(event.target)) {
        onClose()
      }
    }

    const handleEscape = (event) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleEscape)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [onClose])

  // 处理背景图片选择
  const handleBackgroundSelect = async (event) => {
    const file = event.target.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = async (e) => {
        const imageUrl = e.target?.result
        try {
          // 使用模糊处理工具处理图片
          const blurredImageUrl = await blurImage(imageUrl, 10)
          setBackgroundImage(blurredImageUrl)
          const storageKey = getStorageKey(activeChannelId)
          if (typeof localStorage !== 'undefined') {
            localStorage.setItem(storageKey, blurredImageUrl)
          }
          onBackgroundChange?.(blurredImageUrl)
        } catch (error) {
          console.error('图片模糊处理失败:', error)
          // 如果处理失败，使用原图
          setBackgroundImage(imageUrl)
          const storageKey = getStorageKey(activeChannelId)
          if (typeof localStorage !== 'undefined') {
            localStorage.setItem(storageKey, imageUrl)
          }
          onBackgroundChange?.(imageUrl)
        }
      }
      reader.readAsDataURL(file)
    }
  }

  // 移除背景图片
  const handleRemoveBackground = () => {
    setBackgroundImage('')
    const storageKey = getStorageKey(activeChannelId)
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(storageKey)
    }
    onBackgroundChange?.('')
  }

  // 切换 Gateway 服务状态
  const toggleGatewayService = async () => {
    if (!gatewayConfig) return
    try {
      await invoke('toggle_bot_gateway', { enabled: !gatewayConfig.enabled })
      await loadGatewayConfig()
    } catch (error) {
      alert(`操作失败：${error}`)
    }
  }

  // 保存 Gateway 配置
  const handleSaveGateway = async () => {
    if (!gatewayConfig) return
    setGatewaySaving(true)
    try {
      await invoke('update_bot_gateway_config', { config: gatewayConfig })
      await loadGatewayConfig()
      alert(t('common.gateway.success.saved') || '配置已保存')
    } catch (error) {
      alert(`保存失败：${error}`)
    } finally {
      setGatewaySaving(false)
    }
  }

  // 测试平台连接
  const testConnection = async (platform) => {
    try {
      const result = await invoke('test_platform_connection', { platform })
      alert(result.message)
    } catch (error) {
      alert(`测试失败：${error}`)
    }
  }

  // 更新 Gateway 配置
  const updateGatewayConfig = (path, value) => {
    setGatewayConfig((prev) => {
      if (!prev) return prev
      
      const newConfig = JSON.parse(JSON.stringify(prev)) // Deep clone
      
      // 处理平台特定的路径
      if (path.startsWith('platforms.')) {
        const platformPath = path.substring('platforms.'.length)
        const [platform, ...restKeys] = platformPath.split('.')
        const field = restKeys.join('.')
        
        // 确保 platforms 对象存在
        if (!newConfig.platforms) {
          newConfig.platforms = {}
        }
        
        // 确保平台对象存在
        if (!newConfig.platforms[platform]) {
          newConfig.platforms[platform] = { enabled: false }
        }
        
        // 设置值
        if (field) {
          newConfig.platforms[platform][field] = value
        } else {
          // 如果直接设置整个平台对象
          Object.assign(newConfig.platforms[platform], value)
        }
      } else {
        // 处理顶层路径
        const keys = path.split('.')
        let current = newConfig
        
        for (let i = 0; i < keys.length - 1; i++) {
          const key = keys[i]
          if (current[key] === null || current[key] === undefined) {
            current[key] = {}
          }
          current = current[key]
        }
        
        current[keys[keys.length - 1]] = value
      }
      
      return newConfig
    })
  }

  const overlayClassName = `settings-panel-overlay ${isSidebarCollapsed ? 'with-sidebar-collapsed' : 'with-sidebar'}`

  return (
    <div className={overlayClassName}>
      <div ref={panelRef} className={`settings-panel ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="settings-panel-header">
          <button type="button" className="close-btn" onClick={onClose}>
            <img src={CloseIcon} alt={t('common.close')} />
          </button>
          <h3>{t('common.settings.title')}</h3>
          <div className="header-spacer"></div>
        </div>

        <div className="settings-panel-content">
          {/* 主题切换 */}
          <div className="settings-section">
            <div className="settings-section-title">{t('common.theme.title')}</div>
            <div className="settings-option">
              <span className="option-label">{t('common.theme.dayNight')}</span>
              <button
                type="button"
                className="theme-toggle-btn"
                onClick={onToggleTheme}
                aria-label={isDarkMode ? t('common.theme.switchToDay') : t('common.theme.switchToNight')}
              >
                <span className={`toggle-switch ${isDarkMode ? 'dark' : 'light'}`}>
                  <span className="toggle-slider" />
                </span>
                <span className="toggle-label">{isDarkMode ? t('common.theme.night') : t('common.theme.day')}</span>
              </button>
            </div>
          </div>

          {/* API 配置 */}
          <div className="settings-section">
            <div className="settings-section-title">{t('common.settings.apiConfig.title')}</div>
            <div className="settings-option">
              <span className="option-label">{t('common.settings.apiConfig.label')}</span>
              <button
                type="button"
                className="btn-select-image"
                onClick={() => {
                  onOpenApiConfig?.()
                }}
              >
                {t('common.settings.apiConfig.open')}
              </button>
            </div>
          </div>

          {/* Bot Gateway 设置 */}
          <div className="settings-section">
            <div className="settings-section-title">{t('common.gateway.title')}</div>
            <div className="settings-option">
              <span className="option-label">{t('common.gateway.subtitle')}</span>
              <button
                type="button"
                className="btn-select-image"
                onClick={() => setShowGatewaySection(!showGatewaySection)}
              >
                {showGatewaySection ? '收起' : t('common.gateway.configure')}
              </button>
            </div>

            {showGatewaySection && gatewayConfig && (
              <div className="gateway-config-section">
                {/* 状态栏 */}
                <div className={`gateway-status-bar ${gatewayStatus?.running ? 'running' : 'stopped'}`}>
                  <div>
                    <strong>{t('common.gateway.status')}:</strong>{' '}
                    {gatewayStatus?.running ? t('common.gateway.running') : t('common.gateway.stopped')}
                  </div>
                  {gatewayStatus?.running && (
                    <>
                      <div><strong>{t('common.gateway.port')}:</strong> {gatewayStatus.port}</div>
                      <div><strong>{t('common.gateway.platforms')}:</strong> {gatewayStatus.platforms?.join(', ') || t('common.gateway.none')}</div>
                    </>
                  )}
                </div>

                {/* 控制按钮 */}
                <div className="gateway-controls">
                  <button
                    onClick={toggleGatewayService}
                    className={`btn-gateway-toggle ${gatewayConfig.enabled ? 'btn-stop' : 'btn-start'}`}
                  >
                    {gatewayConfig.enabled ? t('common.gateway.stopService') : t('common.gateway.startService')}
                  </button>
                  <button
                    onClick={handleSaveGateway}
                    disabled={gatewaySaving}
                    className="btn-gateway-save"
                  >
                    {gatewaySaving ? t('common.gateway.saving') : t('common.gateway.save')}
                  </button>
                </div>

                {/* 平台选择 */}
                <div className="gateway-platforms">
                  <div className="platform-tabs">
                    {['telegram', 'feishu', 'discord', 'qq'].map((platform) => (
                      <button
                        key={platform}
                        className={`platform-tab ${selectedPlatform === platform ? 'active' : ''}`}
                        onClick={() => setSelectedPlatform(platform)}
                      >
                        {platformIcons[platform]} {t(`common.gateway.platform.${platform}`)}
                      </button>
                    ))}
                  </div>

                  {/* 平台配置表单 */}
                  <div className="platform-config-form">
                    {/* Telegram 配置 */}
                    {selectedPlatform === 'telegram' && (
                      <div className="platform-config-content">
                        <div className="config-item">
                          <label className="config-checkbox">
                            <input
                              type="checkbox"
                              checked={gatewayConfig.platforms?.telegram?.enabled || false}
                              onChange={(e) => updateGatewayConfig('platforms.telegram.enabled', e.target.checked)}
                            />
                            {t('common.gateway.telegram.enabled')}
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.telegram.botToken')}:
                            <input
                              type="password"
                              value={gatewayConfig.platforms?.telegram?.bot_token || ''}
                              onChange={(e) => updateGatewayConfig('platforms.telegram.bot_token', e.target.value)}
                              placeholder={t('common.gateway.telegram.tokenPlaceholder')}
                            />
                          </label>
                        </div>
                        <div className="config-item">
                          <label className="config-checkbox">
                            <input
                              type="checkbox"
                              checked={gatewayConfig.platforms?.telegram?.use_polling || false}
                              onChange={(e) => updateGatewayConfig('platforms.telegram.use_polling', e.target.checked)}
                            />
                            {t('common.gateway.telegram.usePolling')}
                          </label>
                        </div>
                        <button
                          onClick={() => testConnection('telegram')}
                          className="btn-test-connection"
                        >
                          {t('common.gateway.testConnection')}
                        </button>
                        <p className="config-hint">{t('common.gateway.telegram.hint')}</p>
                      </div>
                    )}

                    {/* 飞书配置 */}
                    {selectedPlatform === 'feishu' && (
                      <div className="platform-config-content">
                        <div className="config-item">
                          <label className="config-checkbox">
                            <input
                              type="checkbox"
                              checked={gatewayConfig.platforms?.feishu?.enabled || false}
                              onChange={(e) => updateGatewayConfig('platforms.feishu.enabled', e.target.checked)}
                            />
                            {t('common.gateway.feishu.enabled')}
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.feishu.appId')}:
                            <input
                              type="text"
                              value={gatewayConfig.platforms?.feishu?.app_id || ''}
                              onChange={(e) => updateGatewayConfig('platforms.feishu.app_id', e.target.value)}
                              placeholder={t('common.gateway.feishu.appIdPlaceholder')}
                            />
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.feishu.appSecret')}:
                            <input
                              type="password"
                              value={gatewayConfig.platforms?.feishu?.app_secret || ''}
                              onChange={(e) => updateGatewayConfig('platforms.feishu.app_secret', e.target.value)}
                            />
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.feishu.verifyToken')}:
                            <input
                              type="text"
                              value={gatewayConfig.platforms?.feishu?.verify_token || ''}
                              onChange={(e) => updateGatewayConfig('platforms.feishu.verify_token', e.target.value)}
                            />
                          </label>
                        </div>
                        <button
                          onClick={() => testConnection('feishu')}
                          className="btn-test-connection"
                        >
                          {t('common.gateway.testConnection')}
                        </button>
                        <p className="config-hint">{t('common.gateway.feishu.hint')}</p>
                      </div>
                    )}

                    {/* Discord 配置 */}
                    {selectedPlatform === 'discord' && (
                      <div className="platform-config-content">
                        <div className="config-item">
                          <label className="config-checkbox">
                            <input
                              type="checkbox"
                              checked={gatewayConfig.platforms?.discord?.enabled || false}
                              onChange={(e) => updateGatewayConfig('platforms.discord.enabled', e.target.checked)}
                            />
                            {t('common.gateway.discord.enabled')}
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.discord.botToken')}:
                            <input
                              type="password"
                              value={gatewayConfig.platforms?.discord?.bot_token || ''}
                              onChange={(e) => updateGatewayConfig('platforms.discord.bot_token', e.target.value)}
                              placeholder={t('common.gateway.discord.tokenPlaceholder')}
                            />
                          </label>
                        </div>
                        <button
                          onClick={() => testConnection('discord')}
                          className="btn-test-connection"
                        >
                          {t('common.gateway.testConnection')}
                        </button>
                        <p className="config-hint">{t('common.gateway.discord.hint')}</p>
                      </div>
                    )}

                    {/* QQ 配置 */}
                    {selectedPlatform === 'qq' && (
                      <div className="platform-config-content">
                        <div className="config-item">
                          <label className="config-checkbox">
                            <input
                              type="checkbox"
                              checked={gatewayConfig.platforms?.qq?.enabled || false}
                              onChange={(e) => updateGatewayConfig('platforms.qq.enabled', e.target.checked)}
                            />
                            {t('common.gateway.qq.enabled')}
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.qq.wsUrl')}:
                            <input
                              type="text"
                              value={gatewayConfig.platforms?.qq?.ws_url || ''}
                              onChange={(e) => updateGatewayConfig('platforms.qq.ws_url', e.target.value)}
                              placeholder={t('common.gateway.qq.wsUrlPlaceholder')}
                            />
                          </label>
                        </div>
                        <div className="config-item">
                          <label>
                            {t('common.gateway.qq.accessToken')}:
                            <input
                              type="password"
                              value={gatewayConfig.platforms?.qq?.access_token || ''}
                              onChange={(e) => updateGatewayConfig('platforms.qq.access_token', e.target.value)}
                            />
                          </label>
                        </div>
                        <button
                          onClick={() => testConnection('qq')}
                          className="btn-test-connection"
                        >
                          {t('common.gateway.testConnection')}
                        </button>
                        <p className="config-hint">{t('common.gateway.qq.hint')}</p>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

            {showGatewaySection && !gatewayConfig && gatewayLoading && (
              <div className="gateway-loading">{t('common.loading')}</div>
            )}
          </div>

          {/* 心跳设置 */}
          <div className="settings-section">
            <div className="settings-section-title">心跳设置</div>
            <div className="settings-option">
              <span className="option-label">配置系统心跳机制，保持 7x24 小时无值守运行</span>
              <button
                type="button"
                className="btn-select-image"
                onClick={() => setShowHeartbeatPanel(true)}
              >
                配置心跳
              </button>
            </div>
          </div>

          {/* 背景设置 */}
          <div className="settings-section">
            <div className="settings-section-title">{t('common.settings.background.title')}</div>
            <div className="settings-option">
              <span className="option-label">{t('common.settings.background.label')}</span>
              <div className="background-controls">
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  onChange={handleBackgroundSelect}
                  style={{ display: 'none' }}
                />
                <button
                  type="button"
                  className="btn-select-image"
                  onClick={() => fileInputRef.current?.click()}
                >
                  {t('common.settings.background.select')}
                </button>
                {backgroundImage && (
                  <button
                    type="button"
                    className="btn-remove-background"
                    onClick={handleRemoveBackground}
                  >
                    {t('common.settings.background.remove')}
                  </button>
                )}
              </div>
            </div>
            {backgroundImage && (
              <div className="background-preview">
                <img src={backgroundImage} alt={t('common.settings.background.preview')} />
              </div>
            )}
          </div>
        </div>

        {/* 心跳管理对话框 */}
        {showHeartbeatPanel && (
          <HeartbeatPanel
            isDarkMode={isDarkMode}
            onClose={() => setShowHeartbeatPanel(false)}
          />
        )}
      </div>
    </div>
  )
}

export default SettingsPanel
