import React, { useState, useRef, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import { blurImage } from '@/utils/imageBlur'
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
  
  // 当 activeChannelId 变化时，重新加载对应的背景
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const storageKey = getStorageKey(activeChannelId)
      const stored = localStorage.getItem(storageKey) || ''
      setBackgroundImage(stored)
    }
  }, [activeChannelId])

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

          {/* SDK 功能 */}

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
      </div>
    </div>
  )
}

export default SettingsPanel

