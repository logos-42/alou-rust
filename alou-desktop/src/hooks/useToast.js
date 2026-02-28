import { useState, useCallback, useRef, useEffect } from 'react'
import { TOAST_TYPES } from '@/components/common/Toast'

/**
 * Toast 状态管理 Hook
 * 用于在组件中管理 Toast 通知
 * 
 * @example
 * const { toasts, showToast, hideToast, hideAllToasts } = useToast()
 * 
 * // 显示成功消息
 * showToast('消息发送成功', 'success')
 * 
 * // 显示错误消息
 * showToast('发送失败，请重试', 'error', 5000)
 * 
 * // 在 JSX 中使用
 * <ToastContainer>
 *   {toasts.map(toast => (
 *     <Toast key={toast.id} {...toast} onClose={() => hideToast(toast.id)} />
 *   ))}
 * </ToastContainer>
 */
export const useToast = () => {
  const [toasts, setToasts] = useState([])
  const toastIdRef = useRef(0)

  // 生成唯一 ID
  const generateId = useCallback(() => {
    toastIdRef.current += 1
    return `toast-${Date.now()}-${toastIdRef.current}`
  }, [])

  // 显示 Toast
  const showToast = useCallback((message, type = TOAST_TYPES.INFO, duration = 3000) => {
    const id = generateId()
    const newToast = {
      id,
      message,
      type,
      duration,
    }
    
    setToasts(prev => [...prev, newToast])
    return id
  }, [generateId])

  // 隐藏指定 Toast
  const hideToast = useCallback((id) => {
    setToasts(prev => prev.filter(toast => toast.id !== id))
  }, [])

  // 隐藏所有 Toast
  const hideAllToasts = useCallback(() => {
    setToasts([])
  }, [])

  // 快捷方法
  const success = useCallback((message, duration) => {
    return showToast(message, TOAST_TYPES.SUCCESS, duration)
  }, [showToast])

  const error = useCallback((message, duration) => {
    return showToast(message, TOAST_TYPES.ERROR, duration)
  }, [showToast])

  const warning = useCallback((message, duration) => {
    return showToast(message, TOAST_TYPES.WARNING, duration)
  }, [showToast])

  const info = useCallback((message, duration) => {
    return showToast(message, TOAST_TYPES.INFO, duration)
  }, [showToast])

  return {
    toasts,
    showToast,
    hideToast,
    hideAllToasts,
    success,
    error,
    warning,
    info,
  }
}

/**
 * 全局 Toast 管理器
 * 用于在非 React 组件中显示 Toast
 */
class ToastManager {
  constructor() {
    this.listeners = new Set()
    this.toasts = []
    this.idCounter = 0
  }

  subscribe(listener) {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  notify() {
    this.listeners.forEach(listener => listener(this.toasts))
  }

  show(message, type = TOAST_TYPES.INFO, duration = 3000) {
    this.idCounter += 1
    const id = `global-toast-${Date.now()}-${this.idCounter}`
    const toast = { id, message, type, duration }
    
    this.toasts = [...this.toasts, toast]
    this.notify()
    
    // 自动移除
    setTimeout(() => {
      this.hide(id)
    }, duration + 300) // 加上动画时间
    
    return id
  }

  hide(id) {
    this.toasts = this.toasts.filter(t => t.id !== id)
    this.notify()
  }

  hideAll() {
    this.toasts = []
    this.notify()
  }

  // 快捷方法
  success(message, duration) {
    return this.show(message, TOAST_TYPES.SUCCESS, duration)
  }

  error(message, duration) {
    return this.show(message, TOAST_TYPES.ERROR, duration)
  }

  warning(message, duration) {
    return this.show(message, TOAST_TYPES.WARNING, duration)
  }

  info(message, duration) {
    return this.show(message, TOAST_TYPES.INFO, duration)
  }
}

export const toastManager = new ToastManager()

export default useToast
