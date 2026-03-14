import React, { useEffect, useState, useCallback } from 'react'
import './Toast.css'

export const TOAST_TYPES = {
  SUCCESS: 'success',
  ERROR: 'error',
  WARNING: 'warning',
  INFO: 'info',
} as const

export type ToastType = typeof TOAST_TYPES[keyof typeof TOAST_TYPES]
export type ToastPosition = 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left'

export interface ToastProps {
  message: string
  type?: ToastType
  duration?: number
  onClose?: () => void
  position?: ToastPosition
}

/**
 * Toast 通知组件
 * 用于显示操作反馈、错误提示等临时消息
 */
export const Toast: React.FC<ToastProps> = ({
  message,
  type = TOAST_TYPES.INFO,
  duration = 3000,
  onClose,
  position = 'top-right'
}) => {
  const [isVisible, setIsVisible] = useState(false)
  const [isLeaving, setIsLeaving] = useState(false)

  useEffect(() => {
    // 进入动画
    const enterTimer = setTimeout(() => setIsVisible(true), 10)

    // 自动关闭
    const closeTimer = setTimeout(() => {
      handleClose()
    }, duration)

    return () => {
      clearTimeout(enterTimer)
      clearTimeout(closeTimer)
    }
  }, [duration])

  const handleClose = useCallback(() => {
    setIsLeaving(true)
    setTimeout(() => {
      setIsVisible(false)
      onClose?.()
    }, 300)
  }, [onClose])

  const getIcon = () => {
    switch (type) {
      case TOAST_TYPES.SUCCESS:
        return '✓'
      case TOAST_TYPES.ERROR:
        return '✕'
      case TOAST_TYPES.WARNING:
        return '⚠'
      case TOAST_TYPES.INFO:
      default:
        return 'ℹ'
    }
  }

  if (!isVisible && isLeaving) return null

  return (
    <div
      className={`toast toast-${type} toast-${position} ${isVisible ? 'visible' : ''} ${isLeaving ? 'leaving' : ''}`}
      role="alert"
      aria-live="polite"
    >
      <div className="toast-icon" aria-hidden="true">
        {getIcon()}
      </div>
      <div className="toast-content">
        <span className="toast-message">{message}</span>
      </div>
      <button
        type="button"
        className="toast-close"
        onClick={handleClose}
        aria-label="关闭通知"
      >
        ×
      </button>
      <div className="toast-progress">
        <div
          className="toast-progress-bar"
          style={{ animationDuration: `${duration}ms` }}
        />
      </div>
    </div>
  )
}

/**
 * Toast 容器组件 - 管理多个 Toast
 */
export interface ToastContainerProps {
  children: React.ReactNode
  position?: ToastPosition
}

export const ToastContainer: React.FC<ToastContainerProps> = ({ children, position = 'top-right' }) => {
  return (
    <div className={`toast-container toast-container-${position}`} role="region" aria-label="通知区域">
      {children}
    </div>
  )
}

export default Toast
