import React, { useState, useEffect, useCallback } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './NetworkStatus.css'

/**
 * 网络状态指示器组件
 * 检测网络连接状态并显示相应提示
 */
export const NetworkStatus: React.FC = () => {
  const { t } = useI18n()
  const [isOnline, setIsOnline] = useState(navigator.onLine)
  const [isVisible, setIsVisible] = useState(false)
  const [wasOffline, setWasOffline] = useState(false)

  const handleOnline = useCallback(() => {
    setIsOnline(true)
    if (wasOffline) {
      setIsVisible(true)
      // 3 秒后隐藏
      setTimeout(() => {
        setIsVisible(false)
      }, 3000)
    }
  }, [wasOffline])

  const handleOffline = useCallback(() => {
    setIsOnline(false)
    setWasOffline(true)
    setIsVisible(true)
  }, [])

  useEffect(() => {
    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    // 初始状态检查
    if (!navigator.onLine) {
      // eslint-disable-next-line react-hooks/setState-in-effect
      handleOffline()
    }

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [handleOnline, handleOffline])

  if (!isVisible) return null

  return (
    <div
      className={`network-status ${isOnline ? 'online' : 'offline'}`}
      role="status"
      aria-live="polite"
      aria-label={isOnline ? t('network.online') : t('network.offline')}
    >
      <div className="network-status-content">
        <div className="network-status-icon" aria-hidden="true">
          {isOnline ? '✓' : '!'}
        </div>
        <span className="network-status-text">
          {isOnline ? t('network.online') : t('network.offline')}
        </span>
      </div>
    </div>
  )
}

export interface NetworkStatusBadgeProps {
  showLabel?: boolean
}

/**
 * 网络状态徽章组件
 * 用于显示在网络相关功能旁边的小型状态指示
 */
export const NetworkStatusBadge: React.FC<NetworkStatusBadgeProps> = ({ showLabel = false }) => {
  const [isOnline, setIsOnline] = useState(navigator.onLine)
  const { t } = useI18n()

  useEffect(() => {
    const handleOnline = () => setIsOnline(true)
    const handleOffline = () => setIsOnline(false)

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  return (
    <div
      className={`network-badge ${isOnline ? 'online' : 'offline'}`}
      title={isOnline ? t('network.online') : t('network.offline')}
      aria-label={isOnline ? t('network.online') : t('network.offline')}
    >
      <span className="network-badge-dot" aria-hidden="true" />
      {showLabel && (
        <span className="network-badge-label">
          {isOnline ? t('network.online') : t('network.offline')}
        </span>
      )}
    </div>
  )
}

export default NetworkStatus
