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
          error.message?.includes('ERR_CONNECTION_REFUSED') ||
          error.message?.includes('Failed to fetch') ||
          !error.response

        if (isConnectionError) {
          console.warn(
            '[AgentChat] Backend server unavailable. Please start the backend server or configure VITE_API_BASE_URL.',
          )
        } else {
          console.error('Connection check failed:', error)
        }
      }

      setConnectionStatus('disconnected')
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

      const data = await agentService.createSession(walletAddress || undefined)
      setSessionId(data.session_id)
    } catch (error) {
      const isConnectionError =
        error.code === 'ECONNREFUSED' ||
        error.code === 'ERR_NETWORK' ||
        error.message?.includes('ERR_CONNECTION_REFUSED') ||
        error.message?.includes('Failed to fetch') ||
        !error.response

      const now = Date.now()
      const lastErrorTime = window.__lastCreateSessionError || 0

      if (isConnectionError) {
        if (now - lastErrorTime > 10000) {
          window.__lastCreateSessionError = now
          console.warn(
            '[AgentChat] Cannot create session: backend server unavailable. Please start the backend server or configure VITE_API_BASE_URL.',
          )
        }
      } else {
        if (now - lastErrorTime > 5000) {
          window.__lastCreateSessionError = now
          console.error('Failed to create session:', error)
        }
      }
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

