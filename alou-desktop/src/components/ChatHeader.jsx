import React, { useMemo, useState, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useI18n } from '@/hooks/useI18n'
import { useAuthStore } from '@/stores/authStore'
import avatarService from '@/services/avatarService'
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
  const { user, updateProfile } = useAuthStore()
  const fileInputRef = useRef(null)

  const [showUserMenu, setShowUserMenu] = useState(false)
  const [showSettingsPanel, setShowSettingsPanel] = useState(false)
  const [showApiConfigModal, setShowApiConfigModal] = useState(false)
  const [showAvatarMenu, setShowAvatarMenu] = useState(false)
  const [isUploadingAvatar, setIsUploadingAvatar] = useState(false)

  const userAvatar = user?.avatar_url || user?.avatar

  const statusText = useMemo(() => {
    const key = statusTextMap[connectionStatus] || 'connecting'
    return t(key)
  }, [connectionStatus, t])

  const handleToggleMenu = () => {
    setShowUserMenu((prev) => !prev)
    setShowAvatarMenu(false)
  }

  const handleWalletClick = () => {
    setShowUserMenu(false)
    onGoToWallet?.()
  }

  const handleLogoutClick = () => {
    setShowUserMenu(false)
    onLogout?.()
  }

  /**
   * 处理头像点击
   */
  const handleAvatarClick = () => {
    setShowAvatarMenu((prev) => !prev)
  }

  /**
   * 切换头像风格
   */
  const handleSwitchAvatarStyle = async (style) => {
    try {
      const newAvatar = avatarService.generateDicebearAvatar(user?.id || userName, style)
      await updateProfile({ avatar_url: newAvatar, avatar: newAvatar })
      setShowAvatarMenu(false)
    } catch (error) {
      console.error('[ChatHeader] 切换头像风格失败:', error)
      alert('切换头像失败：' + error.message)
    }
  }

  /**
   * 使用默认头像
   */
  const handleUseDefaultAvatar = async () => {
    try {
      const newAvatar = avatarService.getFallbackAvatar()
      await updateProfile({ avatar_url: newAvatar, avatar: newAvatar })
      setShowAvatarMenu(false)
    } catch (error) {
      console.error('[ChatHeader] 使用默认头像失败:', error)
    }
  }

  /**
   * 上传头像文件
   */
  const handleUploadAvatar = async (event) => {
    const file = event.target.files?.[0]
    if (!file) return

    setIsUploadingAvatar(true)
    try {
      const result = await avatarService.uploadAvatar(file)
      if (result.success) {
        await updateProfile({
          avatar_url: result.avatarUrl,
          avatar: result.avatarUrl,
        })
        setShowAvatarMenu(false)
      } else {
        alert('上传失败：' + (result.error || '未知错误'))
      }
    } catch (error) {
      console.error('[ChatHeader] 上传头像失败:', error)
      alert('上传失败：' + error.message)
    } finally {
      setIsUploadingAvatar(false)
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
    }
  }

  /**
   * 打开文件选择
   */
  const handleOpenFileSelect = () => {
    fileInputRef.current?.click()
    setShowAvatarMenu(false)
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
              <div className="user-avatar-wrapper">
                <button
                  type="button"
                  onClick={handleAvatarClick}
                  className="user-avatar-btn"
                  title="点击切换头像"
                >
                  <img
                    src={userAvatar || avatarService.getFallbackAvatar()}
                    alt={userName}
                    className="user-avatar"
                    onError={(e) => {
                      e.target.src = avatarService.getFallbackAvatar()
                    }}
                  />
                </button>
                {showAvatarMenu && (
                  <div className="avatar-dropdown">
                    <div className="avatar-dropdown-header">
                      <span>选择头像</span>
                    </div>
                    <div className="avatar-dropdown-section">
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={handleOpenFileSelect}
                        disabled={isUploadingAvatar}
                      >
                        📁 上传头像
                        {isUploadingAvatar && <span className="avatar-uploading">上传中...</span>}
                      </button>
                      <input
                        ref={fileInputRef}
                        type="file"
                        accept="image/*"
                        onChange={handleUploadAvatar}
                        style={{ display: 'none' }}
                      />
                    </div>
                    <div className="avatar-dropdown-divider"></div>
                    <div className="avatar-dropdown-section">
                      <div className="avatar-style-label">Dicebear 风格</div>
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={() => handleSwitchAvatarStyle('identicon')}
                      >
                        🔷 Identicon
                      </button>
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={() => handleSwitchAvatarStyle('avataaars')}
                      >
                        😊 Avataaars
                      </button>
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={() => handleSwitchAvatarStyle('bottts')}
                      >
                        🤖 Bottts
                      </button>
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={() => handleSwitchAvatarStyle('lorelei')}
                      >
                        🧚 Lorelei
                      </button>
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={() => handleSwitchAvatarStyle('notionists')}
                      >
                        📝 Notionists
                      </button>
                    </div>
                    <div className="avatar-dropdown-divider"></div>
                    <div className="avatar-dropdown-section">
                      <button
                        type="button"
                        className="avatar-menu-item"
                        onClick={handleUseDefaultAvatar}
                      >
                        🔄 使用默认头像
                      </button>
                    </div>
                  </div>
                )}
              </div>
              <button type="button" onClick={handleToggleMenu} className="user-btn">
                <span>{userName}</span>
                <span className="user-menu-arrow">▼</span>
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
