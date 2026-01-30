import { useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { MCP_UI_TARGETS } from '@/services/mcpUiService'

// 钱包快照类型
interface WalletSnapshot {
  chain?: string;
}

// 交易类型
interface Transaction {
  hash?: string;
  id?: string;
}

// 消息类型
interface Message {
  id?: string;
  type?: string;
  timestamp?: number;
}

// Hook参数类型
interface UseAgentEventHandlersParams {
  navigate: ReturnType<typeof useNavigate>;
  logout: () => Promise<void>;
  recordInteraction: (action: string, data?: unknown) => void;
  setLanguage: (lang: string) => void;
  currentLanguage: string;
  refreshWallet: () => Promise<void>;
  createSession: () => Promise<void>;
  setSessionReady: (ready: boolean) => void;
  sessionId: string | null;
  walletSnapshot: WalletSnapshot | null;
  fetchAndOpenUiResource: (target: string, params: Record<string, unknown>, options: Record<string, unknown>) => Promise<void>;
}

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
}: UseAgentEventHandlersParams) => {
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
    async (event: CustomEvent<{ address?: string }>) => {
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
    (transaction: Transaction) => {
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
    (message: Message) => {
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

export default useAgentEventHandlers
