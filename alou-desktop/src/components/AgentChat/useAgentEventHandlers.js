import { useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { MCP_UI_TARGETS } from '@/services/mcpUiService'

/**
 * Hook for managing agent event handlers
 * 管理智能体事件处理器
 */
export const useAgentEventHandlers = ({
  navigate,
  logout,
  recordInteraction,
  setLanguage,
  currentLanguage,
  refreshWallet,
  createSession,
  setSessionReady,
  sessionId,
  walletSnapshot,
  fetchAndOpenUiResource,
}) => {
  const goToLogin = useCallback(() => {
    recordInteraction('navigate_login')
    navigate('/login')
  }, [navigate, recordInteraction])

  const goToWallet = useCallback(() => {
    recordInteraction('navigate_wallet')
    navigate('/wallet')
  }, [navigate, recordInteraction])

  const handleLogout = useCallback(async () => {
    recordInteraction('logout')
    await logout()
  }, [logout, recordInteraction])

  const toggleLanguage = useCallback(() => {
    const next = currentLanguage === 'zh' ? 'en' : 'zh'
    setLanguage(next)
    recordInteraction('toggle_language', { language: next })
  }, [currentLanguage, recordInteraction, setLanguage])

  const handleWalletChanged = useCallback(
    async (event) => {
      const detail = event?.detail
      recordInteraction('wallet_event', { address: detail?.address })
      await refreshWallet()
      await createSession()
      setSessionReady(true)
    },
    [createSession, recordInteraction, refreshWallet, setSessionReady],
  )

  const handleInspectWallet = useCallback(() => {
    if (!sessionId) return
    void fetchAndOpenUiResource(
      MCP_UI_TARGETS.walletOverview,
      { session_id: sessionId, chain: walletSnapshot?.chain },
      { source: 'wallet_card' },
    )
  }, [fetchAndOpenUiResource, sessionId, walletSnapshot])

  const handleInspectTransaction = useCallback(
    (transaction) => {
      if (!transaction) return
      const transactionId = transaction.hash || transaction.id
      if (!transactionId) return
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.transactionDetail,
        { session_id: sessionId, transaction_id: transactionId, chain: walletSnapshot?.chain },
        { source: 'transaction_list', transactionId },
      )
    },
    [fetchAndOpenUiResource, sessionId, walletSnapshot],
  )

  const handleInspectMessage = useCallback(
    (message) => {
      if (!message?.id) return
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.conversationDetail,
        { message_id: message.id, role: message.type, timestamp: message.timestamp },
        { source: 'conversation', messageId: message.id },
      )
    },
    [fetchAndOpenUiResource],
  )

  return {
    goToLogin,
    goToWallet,
    handleLogout,
    toggleLanguage,
    handleWalletChanged,
    handleInspectWallet,
    handleInspectTransaction,
    handleInspectMessage,
  }
}

