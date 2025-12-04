import React, { useMemo, useState } from 'react'
import { useI18n } from '@/hooks/useI18n'
import AlouLogo from '@/../src-tauri/icons/Square30x30Logo.png'
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
}) => {
  const { t } = useI18n()
  const [showUserMenu, setShowUserMenu] = useState(false)

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
          <div className={`status-badge ${connectionStatus}`}>
            <div className="status-dot" />
            <span>{statusText}</span>
          </div>

          <button type="button" onClick={onToggleTheme} className="theme-toggle" title={t('theme')}>
            {isDarkMode ? '🌞' : '🌙'}
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
    </nav>
  )
}

export default ChatHeader
