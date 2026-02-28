import { useCallback, useEffect, useRef, useState } from 'react'

/**
 * 新消息通知 Hook
 * 处理新消息的音效、视觉提示和浏览器通知
 * 
 * @param {Object} options
 * @param {boolean} options.enabled - 是否启用通知
 * @param {boolean} options.soundEnabled - 是否启用声音
 * @param {string} options.soundUrl - 自定义音效文件URL
 * @param {boolean} options.browserNotification - 是否启用浏览器通知
 * @param {string} options.title - 通知标题
 */
export const useNewMessageNotification = (options = {}) => {
  const {
    enabled = true,
    soundEnabled = true,
    soundUrl = '/notification-sound.mp3',
    browserNotification = false,
    title = '新消息',
  } = options

  const audioRef = useRef(null)
  const [permission, setPermission] = useState('default')

  // 初始化音频和通知权限
  useEffect(() => {
    if (soundEnabled && typeof window !== 'undefined') {
      audioRef.current = new Audio(soundUrl)
      audioRef.current.volume = 0.5
    }

    if (browserNotification && 'Notification' in window) {
      setPermission(Notification.permission)
    }
  }, [soundEnabled, soundUrl, browserNotification])

  // 请求浏览器通知权限
  const requestPermission = useCallback(async () => {
    if (!('Notification' in window)) {
      console.warn('浏览器不支持通知功能')
      return false
    }

    try {
      const result = await Notification.requestPermission()
      setPermission(result)
      return result === 'granted'
    } catch (error) {
      console.error('请求通知权限失败:', error)
      return false
    }
  }, [])

  // 播放通知音效
  const playSound = useCallback(() => {
    if (!enabled || !soundEnabled || !audioRef.current) return

    try {
      // 重置音频到开始位置
      audioRef.current.currentTime = 0
      audioRef.current.play().catch(error => {
        // 自动播放可能被浏览器阻止，这是正常的
        console.log('播放通知音效失败:', error)
      })
    } catch (error) {
      console.error('播放音效出错:', error)
    }
  }, [enabled, soundEnabled])

  // 显示浏览器通知
  const showBrowserNotification = useCallback((message, options = {}) => {
    if (!enabled || !browserNotification || permission !== 'granted') return

    try {
      const notification = new Notification(title, {
        body: message,
        icon: '/favicon.ico',
        badge: '/favicon.ico',
        tag: 'new-message',
        requireInteraction: false,
        ...options,
      })

      notification.onclick = () => {
        window.focus()
        notification.close()
      }

      // 3秒后自动关闭
      setTimeout(() => {
        notification.close()
      }, 3000)
    } catch (error) {
      console.error('显示浏览器通知失败:', error)
    }
  }, [enabled, browserNotification, permission, title])

  // 触发新消息通知（音效 + 浏览器通知）
  const notify = useCallback((message, options = {}) => {
    if (!enabled) return

    playSound()
    
    // 如果页面不可见，显示浏览器通知
    if (document.hidden) {
      showBrowserNotification(message, options)
    }
  }, [enabled, playSound, showBrowserNotification])

  // 震动反馈（移动设备）
  const vibrate = useCallback((pattern = [50, 100, 50]) => {
    if (!enabled || !navigator.vibrate) return

    try {
      navigator.vibrate(pattern)
    } catch (error) {
      console.log('震动反馈不可用')
    }
  }, [enabled])

  return {
    notify,
    playSound,
    showBrowserNotification,
    requestPermission,
    vibrate,
    permission,
    isSupported: 'Notification' in window,
  }
}

export default useNewMessageNotification
