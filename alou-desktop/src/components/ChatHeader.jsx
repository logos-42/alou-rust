import React, { useMemo, useState } from 'react'
import { useI18n } from '@/hooks/useI18n'
import AlouLogo from '@/../src-tauri/icons/Square30x30Logo.png'
import ApiDocIcon from '@/assets/API文档.png'
import SettingsIcon from '@/assets/设置 0.7.png'
import SettingsPanel from './SettingsPanel'
import './ChatHeader.css'

const statusTextMap = {
  connected: 'connected',
  disconnected: 'disconnected',
  error: 'connecting',
}

const ChatHeader = ({
  connectionStatus = 'disconnected',
  isDarkMode,
  isAuthenticated,
  userName,
  onToggleTheme,
  onGoToLogin,
  onGoToWallet,
  onLogout,
  onBackgroundChange,
  activeChannelId = null,
}) => {
  const { t } = useI18n()
  const [showUserMenu, setShowUserMenu] = useState(false)
  const [showSettingsPanel, setShowSettingsPanel] = useState(false)

  const statusText = useMemo(() => {
    const key = statusTextMap[connectionStatus] || 'connecting'
    return t(key)
  }, [connectionStatus, t])

  const handleToggleMenu = () => {
    setShowUserMenu((prev) => !prev)
  }

  const handleWalletClick = () => {
    setShowUserMenu(false)
    onGoToWallet?.()
  }

  const handleLogoutClick = () => {
    setShowUserMenu(false)
    onLogout?.()
  }

  return (
    <nav className="top-nav">
      <div className="nav-content">
        <div className="logo-section">
          <img src={AlouLogo} alt="Alou" className="logo" />
          <h1 className="app-title">Alou</h1>
        </div>

        <div className="nav-controls">
          <button 
            type="button" 
            onClick={() => {
              // 打开API配置页面
              window.open('https://docs.alou.ai/api', '_blank')
            }} 
            className="api-icon-btn" 
            title={statusText}
          >
            <img src={ApiDocIcon} alt="API" className="api-icon" />
          </button>

          <button type="button" onClick={() => setShowSettingsPanel(true)} className="theme-toggle settings-toggle" title="设置">
            <img src={SettingsIcon} alt="设置" />
          </button>

          {!isAuthenticated ? (
            <button type="button" onClick={onGoToLogin} className="login-btn">
              <span>🔐</span>
              <span>{t('login')}</span>
            </button>
          ) : (
            <div className="user-menu">
              <button type="button" onClick={handleToggleMenu} className="user-btn">
                <span>👤</span>
                <span>{userName}</span>
              </button>
              {showUserMenu && (
                <div className="user-dropdown">
                  <button type="button" onClick={handleWalletClick} className="menu-item">
                    <span>💰</span>
                    <span>{t('walletManagement')}</span>
                  </button>
                  <button type="button" onClick={handleLogoutClick} className="menu-item">
                    <span>🚪</span>
                    <span>{t('logout')}</span>
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
      {showSettingsPanel && (
        <SettingsPanel
          isDarkMode={isDarkMode}
          onToggleTheme={onToggleTheme}
          onClose={() => setShowSettingsPanel(false)}
          onBackgroundChange={onBackgroundChange}
          activeChannelId={activeChannelId}
        />
      )}
    </nav>
  )
}

export default ChatHeader
