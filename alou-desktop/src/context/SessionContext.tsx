/**
 * Session 上下文 - 提供全应用级别的 Session 管理
 *
 * 通过 React Context 提供当前 session 的 coordinator 实例
 * 确保每个页面/组件都使用正确的 session 数据
 *
 * @module context/SessionContext
 */

import React, { createContext, useContext, useEffect, useState, useCallback, useMemo } from 'react'
import { UnifiedAgentCoordinator, AgentCoordinatorConfig } from '@/services/unifiedAgentCoordinator'
import { sessionManager, getOrCreateSession, SessionInfo } from '@/services/sessionManager'
import { setCurrentSessionId } from '@/stores/agentStore'

/**
 * Session 上下文接口
 */
interface SessionContextValue {
  session: SessionInfo | null
  coordinator: UnifiedAgentCoordinator | null
  sessionId: string | null
  isInitialized: boolean
  initSession: (config?: AgentCoordinatorConfig) => Promise<void>
  destroySession: () => Promise<void>
  switchSession: (sessionId: string) => Promise<void>
  touchSession: () => void
}

const SessionContext = createContext<SessionContextValue | null>(null)

interface SessionProviderProps {
  children: React.ReactNode
  defaultConfig?: AgentCoordinatorConfig
  autoInit?: boolean
}

export const SessionProvider: React.FC<SessionProviderProps> = ({
  children,
  defaultConfig = {},
  autoInit = true,
}) => {
  const [session, setSession] = useState<SessionInfo | null>(null)
  const [coordinator, setCoordinator] = useState<UnifiedAgentCoordinator | null>(null)
  const [isInitialized, setIsInitialized] = useState(false)
  const [isLoading, setIsLoading] = useState(autoInit)

  const initSession = useCallback(async (config?: AgentCoordinatorConfig) => {
    try {
      setIsLoading(true)
      
      const hash = window.location.hash
      const match = hash.match(/session=([^&]+)/)
      const urlSessionId = match ? decodeURIComponent(match[1]) : undefined

      const sessionInfo = getOrCreateSession({
        ...defaultConfig,
        ...config,
        sessionId: urlSessionId || config?.sessionId,
      })

      setCurrentSessionId(sessionInfo.sessionId)

      if (!urlSessionId) {
        const newHash = `session=${sessionInfo.sessionId}`
        const currentHash = window.location.hash
        if (!currentHash.includes('session=')) {
          window.location.hash = currentHash ? `${currentHash}&${newHash}` : `#${newHash}`
        }
      }

      setSession(sessionInfo)
      setCoordinator(sessionInfo.coordinator)
      setIsInitialized(true)

      console.log('[SessionContext] Session 初始化完成:', {
        sessionId: sessionInfo.sessionId,
      })
    } catch (error) {
      console.error('[SessionContext] Session 初始化失败:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [defaultConfig])

  const destroySession = useCallback(async () => {
    if (!session) return

    try {
      await sessionManager.destroySession(session.sessionId)
      setSession(null)
      setCoordinator(null)
      setIsInitialized(false)
      console.log('[SessionContext] Session 已销毁:', session.sessionId)
    } catch (error) {
      console.error('[SessionContext] Session 销毁失败:', error)
    }
  }, [session])

  const switchSession = useCallback(async (newSessionId: string) => {
    if (session) {
      await sessionManager.destroySession(session.sessionId, false)
    }

    const sessionInfo = getOrCreateSession({
      ...defaultConfig,
      sessionId: newSessionId,
    })

    setCurrentSessionId(sessionInfo.sessionId)
    setSession(sessionInfo)
    setCoordinator(sessionInfo.coordinator)

    window.location.hash = `session=${sessionInfo.sessionId}`

    console.log('[SessionContext] Session 已切换:', newSessionId)
  }, [session, defaultConfig])

  const touchSession = useCallback(() => {
    if (session) {
      sessionManager.touchSession(session.sessionId)
    }
  }, [session])

  useEffect(() => {
    if (autoInit && !isInitialized && !isLoading) {
      initSession(defaultConfig).catch(console.error)
    }
  }, [autoInit, isInitialized, isLoading, initSession, defaultConfig])

  useEffect(() => {
    const handleBeforeUnload = () => {
      if (session) {
        sessionManager.destroySession(session.sessionId)
      }
    }

    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload)
    }
  }, [session])

  useEffect(() => {
    const handleSessionChanged = (event: CustomEvent) => {
      const { sessionId: newSessionId } = event.detail
      if (newSessionId && newSessionId !== session?.sessionId) {
        switchSession(newSessionId).catch(console.error)
      }
    }

    window.addEventListener('alou:session-changed', handleSessionChanged as EventListener)
    return () => {
      window.removeEventListener('alou:session-changed', handleSessionChanged as EventListener)
    }
  }, [session, switchSession])

  const value = useMemo<SessionContextValue>(() => ({
    session,
    coordinator,
    sessionId: session?.sessionId || null,
    isInitialized,
    initSession,
    destroySession,
    switchSession,
    touchSession,
  }), [session, coordinator, isInitialized, initSession, destroySession, switchSession, touchSession])

  return (
    <SessionContext.Provider value={value}>
      {children}
    </SessionContext.Provider>
  )
}

export const useSession = (): SessionContextValue => {
  const context = useContext(SessionContext)
  
  if (!context) {
    throw new Error('useSession 必须在 SessionProvider 内部使用')
  }
  
  return context
}

export const useCoordinator = (): UnifiedAgentCoordinator | null => {
  const { coordinator } = useSession()
  return coordinator
}

export const useSessionId = (): string | null => {
  const { sessionId } = useSession()
  return sessionId
}

export default SessionContext
