import { useCallback, useEffect, useMemo, useState } from 'react'
import agentService from '@/services/agentService'
import { resolveBackendChain } from '@/hooks/useAgentChat'

/**
 * Hook for managing agent connection state and session
 */
export const useAgentConnection = ({ activeChain, preferredChain, setPreferredChain }) => {
  const [connectionStatus, setConnectionStatus] = useState('disconnected')
  const [sessionId, setSessionId] = useState(
    `frontend_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
  )
  const [isSessionReady, setSessionReady] = useState(false)

  const connectionStatusLabel = useMemo(() => {
    if (connectionStatus === 'connected') return '已连接'
    if (connectionStatus === 'connecting') return '连接中'
    if (connectionStatus === 'error') return '服务异常'
    return '未连接'
  }, [connectionStatus])

  const checkConnection = useCallback(async () => {
    try {
      await agentService.healthCheck()
      setConnectionStatus('connected')
    } catch (error) {
      const now = Date.now()
      const lastErrorTime = window.__lastHealthCheckError || 0

      if (now - lastErrorTime > 10000) {
        window.__lastHealthCheckError = now
        const isConnectionError =
          error.code === 'ECONNREFUSED' ||
          error.code === 'ERR_NETWORK' ||
          error.code === 'ERR_BAD_RESPONSE' ||
          error.message?.includes('ERR_CONNECTION_REFUSED') ||
          error.message?.includes('Failed to fetch') ||
          error.message?.includes('ETIMEDOUT') ||
          !error.response

        if (isConnectionError) {
          console.warn(
            '[AgentChat] 后端服务器不可用，应用将在本地模式下运行。',
          )
          // 后端不可用时，设置为本地模式
          setConnectionStatus('local')
        } else {
          console.error('Connection check failed:', error)
          setConnectionStatus('disconnected')
        }
      } else {
        setConnectionStatus('disconnected')
      }
    }
  }, [])

  const createSession = useCallback(async () => {
    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      const chainId =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
      const detectedChain = resolveBackendChain({
        chainId,
        chain: activeChain,
      })

      if (detectedChain && detectedChain !== preferredChain) {
        setPreferredChain(detectedChain)
      }

      console.log('[useAgentConnection] 正在创建会话...')
      try {
        const data = await agentService.createSession(walletAddress || undefined)
        console.log('[useAgentConnection] 会话创建成功:', data.session_id)
        setSessionId(data.session_id)
        return data.session_id
      } catch (error) {
        const isConnectionError =
          error.code === 'ECONNREFUSED' ||
          error.code === 'ERR_NETWORK' ||
          error.code === 'ERR_BAD_RESPONSE' ||
          error.message?.includes('ERR_CONNECTION_REFUSED') ||
          error.message?.includes('Failed to fetch') ||
          error.message?.includes('ETIMEDOUT') ||
          !error.response

        const now = Date.now()
        const lastErrorTime = window.__lastCreateSessionError || 0

        if (isConnectionError) {
          if (now - lastErrorTime > 10000) {
            window.__lastCreateSessionError = now
            console.warn(
              '[AgentChat] 后端服务器不可用，使用本地会话模式。',
            )
          }
          
          // 后端不可用时，生成本地会话ID
          const localSessionId = `local_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
          console.log('[useAgentConnection] 创建本地会话:', localSessionId)
          setSessionId(localSessionId)
          return localSessionId
        } else {
          if (now - lastErrorTime > 5000) {
            window.__lastCreateSessionError = now
            console.error('Failed to create session:', error)
          }
          // 对于非连接错误，仍然抛出
          throw error
        }
      }
    } catch (error) {
      console.error('[useAgentConnection] 创建会话失败:', error)
      throw error
    }
  }, [activeChain, preferredChain, setPreferredChain])

  return {
    connectionStatus,
    setConnectionStatus,
    connectionStatusLabel,
    sessionId,
    setSessionId,
    isSessionReady,
    setSessionReady,
    checkConnection,
    createSession,
  }
}
