import React, { useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useI18n } from '@/hooks/useI18n'
import AlouLogo from '@/../src-tauri/icons/Square30x30Logo.png'
import SettingsIcon from '@/assets/设置 0.7.png'
import LoginIcon from '@/assets/登录.png'
import SettingsPanel from './SettingsPanel'
import ApiConfigModal from './ApiConfigModal'
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
  const navigate = useNavigate()
  const { t } = useI18n()
  const [showUserMenu, setShowUserMenu] = useState(false)
  const [showSettingsPanel, setShowSettingsPanel] = useState(false)
  const [showApiConfigModal, setShowApiConfigModal] = useState(false)

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
            onClick={() => setShowSettingsPanel(true)} 
            className="api-icon-btn settings-icon-btn" 
            title="设置"
          >
            <img src={SettingsIcon} alt="设置" className="api-icon" />
          </button>

          {!isAuthenticated ? (
            <button type="button" onClick={onGoToLogin} className="login-btn">
              <img src={LoginIcon} alt="登录" className="login-icon" />
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
          onOpenApiConfig={() => {
            setShowSettingsPanel(false)
            setShowApiConfigModal(true)
          }}
        />
      )}
      {showApiConfigModal && (
        <ApiConfigModal
          isOpen={showApiConfigModal}
          onClose={() => setShowApiConfigModal(false)}
          isDarkMode={isDarkMode}
        />
      )}
    </nav>
  )
}

export default ChatHeader
