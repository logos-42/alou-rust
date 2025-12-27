import { useEffect, useRef, useState } from 'react'
import { API_BASE_URL } from '@/hooks/useAgentChat'

const buildProgressUrl = (sessionId, since) => {
  if (!sessionId) return null
  const base = (API_BASE_URL || '').replace(/\/$/, '')
  try {
    const url = new URL('/api/agent/progress', base || window.location.origin)
    url.searchParams.set('session_id', sessionId)
    if (typeof since === 'number') {
      url.searchParams.set('since', since.toString())
    }
    return url.toString()
  } catch (_error) {
    const origin = base || ''
    const search = new URLSearchParams({ session_id: sessionId })
    if (typeof since === 'number') {
      search.set('since', since.toString())
    }
    return `${origin}/api/agent/progress?${search.toString()}`
  }
}

export const useAgentStream = (sessionId, options = {}) => {
  const {
    enabled = true,
    onEvent,
    intervalMs = 2500,
    idleIntervalMs = 4500,
  } = options
  const [state, setState] = useState('idle') // idle | polling | active | completed | error
  const [events, setEvents] = useState([])
  const lastTimestampRef = useRef(null)
  const timerRef = useRef(null)
  const abortRef = useRef(null)

  useEffect(() => {
    if (!sessionId || !enabled) {
      return undefined
    }

    let stopped = false
    setEvents([])
    setState('polling')
    lastTimestampRef.current = null

    const clearTimer = () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current)
        timerRef.current = null
      }
    }

    const scheduleNext = (delay) => {
      clearTimer()
      timerRef.current = setTimeout(runOnce, delay)
    }

    const runOnce = async () => {
      if (stopped) {
        return
      }

      const url = buildProgressUrl(sessionId, lastTimestampRef.current)
      if (!url) {
        setState('error')
        return
      }

      const controller = new AbortController()
      abortRef.current = controller

      // 设置超时（10秒）
      const timeoutId = setTimeout(() => {
        controller.abort()
      }, 10000)

      try {
        const response = await fetch(url, {
          method: 'GET',
          headers: { Accept: 'application/json' },
          signal: controller.signal,
        })

        clearTimeout(timeoutId)

        if (!response.ok) {
          throw new Error(`Progress fetch failed: ${response.status}`)
        }

        const data = await response.json()
        const newEvents = Array.isArray(data.events) ? data.events : []

        if (newEvents.length > 0) {
          lastTimestampRef.current = newEvents[newEvents.length - 1].timestamp ?? Date.now()

          let finalFound = false
          setEvents((prev) => {
            const merged = [...prev, ...newEvents]
            finalFound = newEvents.some((event) => event.is_final)
            return merged.slice(-200)
          })

          if (typeof onEvent === 'function') {
            newEvents.forEach((event) => {
              onEvent({
                ...event,
                sessionId: event.session_id || sessionId,
              })
            })
          }

          setState(finalFound ? 'completed' : 'active')
          scheduleNext(finalFound ? idleIntervalMs : intervalMs)
        } else {
          scheduleNext(idleIntervalMs)
        }
      } catch (error) {
        clearTimeout(timeoutId)
        
        if (!stopped) {
          // 如果是超时或网络错误，静默处理，不输出错误日志
          const isTimeout = error.name === 'AbortError' && controller.signal.aborted
          const isNetworkError = error.message?.includes('Failed to fetch') || 
                                error.message?.includes('ERR_TIMED_OUT') ||
                                error.message?.includes('NetworkError')
          
          if (!isTimeout && !isNetworkError) {
            console.error('agent progress polling failed:', error)
          }
          
          // 对于超时和网络错误，使用更长的重试间隔
          const retryDelay = (isTimeout || isNetworkError) ? idleIntervalMs * 2 : idleIntervalMs
          setState('error')
          scheduleNext(retryDelay)
        }
      } finally {
        abortRef.current = null
      }
    }

    runOnce()

    return () => {
      stopped = true
      clearTimer()
      if (abortRef.current) {
        abortRef.current.abort()
      }
      setState('idle')
      lastTimestampRef.current = null
      abortRef.current = null
    }
  }, [sessionId, enabled, intervalMs, idleIntervalMs, onEvent])

  return {
    status: state,
    events,
  }
}

