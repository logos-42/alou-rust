import React, { useState, useRef, useEffect } from 'react'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import './SettingsPanel.css'

const SettingsPanel = ({ isDarkMode, onToggleTheme, onClose, onBackgroundChange, isSidebarCollapsed = false }) => {
  const { t } = useI18n()
  const panelRef = useRef(null)
  const fileInputRef = useRef(null)
  const [backgroundImage, setBackgroundImage] = useState(() => {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('alou-chat-background') || ''
    }
    return ''
  })

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
  const handleBackgroundSelect = (event) => {
    const file = event.target.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (e) => {
        const imageUrl = e.target?.result
        setBackgroundImage(imageUrl)
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem('alou-chat-background', imageUrl)
        }
        onBackgroundChange?.(imageUrl)
      }
      reader.readAsDataURL(file)
    }
  }

  // 移除背景图片
  const handleRemoveBackground = () => {
    setBackgroundImage('')
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem('alou-chat-background')
    }
    onBackgroundChange?.('')
  }

  const overlayClassName = `settings-panel-overlay ${isSidebarCollapsed ? 'with-sidebar-collapsed' : 'with-sidebar'}`
  
  return (
    <div className={overlayClassName}>
      <div ref={panelRef} className={`settings-panel ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="settings-panel-header">
          <button type="button" className="close-btn" onClick={onClose}>
            <img src={CloseIcon} alt="关闭" />
          </button>
          <h3>设置</h3>
          <div className="header-spacer"></div>
        </div>

        <div className="settings-panel-content">
          {/* 主题切换 */}
          <div className="settings-section">
            <div className="settings-section-title">主题</div>
            <div className="settings-option">
              <span className="option-label">白天/黑夜模式</span>
              <button
                type="button"
                className="theme-toggle-btn"
                onClick={onToggleTheme}
                aria-label={isDarkMode ? '切换到白天模式' : '切换到黑夜模式'}
              >
                <span className={`toggle-switch ${isDarkMode ? 'dark' : 'light'}`}>
                  <span className="toggle-slider" />
                </span>
                <span className="toggle-label">{isDarkMode ? '黑夜' : '白天'}</span>
              </button>
            </div>
          </div>

          {/* 背景设置 */}
          <div className="settings-section">
            <div className="settings-section-title">聊天背景</div>
            <div className="settings-option">
              <span className="option-label">背景图片</span>
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
                  选择图片
                </button>
                {backgroundImage && (
                  <button
                    type="button"
                    className="btn-remove-background"
                    onClick={handleRemoveBackground}
                  >
                    移除背景
                  </button>
                )}
              </div>
            </div>
            {backgroundImage && (
              <div className="background-preview">
                <img src={backgroundImage} alt="背景预览" />
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default SettingsPanel

