import { useState, useCallback, useRef } from 'react'
import { TOAST_TYPES } from '@/components/common/Toast'

/**
 * Toast 类型定义
 */
interface Toast {
  id: string
  message: string
  type: string
  duration: number
}

/**
 * Toast 类型
 */
type ToastType = 'success' | 'error' | 'warning' | 'info'

/**
 * useToast 返回值
 */
interface UseToastReturn {
  toasts: Toast[]
  showToast: (message: string, type?: string, duration?: number) => string
  hideToast: (id: string) => void
  hideAllToasts: () => void
  success: (message: string, duration?: number) => string
  error: (message: string, duration?: number) => string
  warning: (message: string, duration?: number) => string
  info: (message: string, duration?: number) => string
}

/**
 * Toast 状态管理 Hook
 * 用于在组件中管理 Toast 通知
 */
export const useToast = (): UseToastReturn => {
  const [toasts, setToasts] = useState<Toast[]>([])
  const toastIdRef = useRef<number>(0)

  // 生成唯一 ID
  const generateId = useCallback((): string => {
    toastIdRef.current += 1
    return `toast-${Date.now()}-${toastIdRef.current}`
  }, [])

  // 显示 Toast
  const showToast = useCallback((message: string, type: string = TOAST_TYPES.INFO, duration: number = 3000): string => {
    const id = generateId()
    const newToast: Toast = {
      id,
      message,
      type,
      duration,
    }

    setToasts(prev => [...prev, newToast])
    return id
  }, [generateId])

  // 隐藏指定 Toast
  const hideToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== id))
  }, [])

  // 隐藏所有 Toast
  const hideAllToasts = useCallback(() => {
    setToasts([])
  }, [])

  // 快捷方法
  const success = useCallback((message: string, duration?: number): string => {
    return showToast(message, TOAST_TYPES.SUCCESS, duration)
  }, [showToast])

  const error = useCallback((message: string, duration?: number): string => {
    return showToast(message, TOAST_TYPES.ERROR, duration)
  }, [showToast])

  const warning = useCallback((message: string, duration?: number): string => {
    return showToast(message, TOAST_TYPES.WARNING, duration)
  }, [showToast])

  const info = useCallback((message: string, duration?: number): string => {
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
  private listeners: Set<(toasts: Toast[]) => void>
  private toasts: Toast[]
  private idCounter: number

  constructor() {
    this.listeners = new Set()
    this.toasts = []
    this.idCounter = 0
  }

  subscribe(listener: (toasts: Toast[]) => void): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  notify(): void {
    this.listeners.forEach(listener => listener(this.toasts))
  }

  show(message: string, type: string = TOAST_TYPES.INFO, duration: number = 3000): string {
    this.idCounter += 1
    const id = `global-toast-${Date.now()}-${this.idCounter}`
    const toast: Toast = { id, message, type, duration }

    this.toasts = [...this.toasts, toast]
    this.notify()

    // 自动移除
    setTimeout(() => {
      this.hide(id)
    }, duration + 300) // 加上动画时间

    return id
  }

  hide(id: string): void {
    this.toasts = this.toasts.filter(t => t.id !== id)
    this.notify()
  }

  hideAll(): void {
    this.toasts = []
    this.notify()
  }

  // 快捷方法
  success(message: string, duration?: number): string {
    return this.show(message, TOAST_TYPES.SUCCESS, duration)
  }

  error(message: string, duration?: number): string {
    return this.show(message, TOAST_TYPES.ERROR, duration)
  }

  warning(message: string, duration?: number): string {
    return this.show(message, TOAST_TYPES.WARNING, duration)
  }

  info(message: string, duration?: number): string {
    return this.show(message, TOAST_TYPES.INFO, duration)
  }
}

export const toastManager = new ToastManager()

export default useToast
