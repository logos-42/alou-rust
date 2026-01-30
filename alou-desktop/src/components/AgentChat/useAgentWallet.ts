import { useCallback, useEffect, useMemo, useState } from 'react'
import { walletService } from '@/services/walletService'
import agentService from '@/services/agentService'
import { requestMcpUiResource, MCP_UI_TARGETS } from '@/services/mcpUiService'
import {
  defaultTransactions,
  estimateFiatValue,
  formatWeiHexToEth,
  mapChainIdToBackendChain,
  mapChainLabel,
  normalizeTransaction,
  resolveBackendChain,
} from '@/hooks/useAgentChat'
import { Transaction, WalletSnapshot } from '@/shared/types'

interface UseAgentWalletOptions {
  sessionId: string
  activeChain: string
  preferredChain: string
  setPreferredChain: (chain: string) => void
  recordInteraction: (action: string, data: Record<string, unknown>, label?: string) => void
  appendMessage: (message: Record<string, unknown>) => void
  scrollToBottom: () => void
  setTransactions?: (transactions: Transaction[] | ((prev: Transaction[]) => Transaction[])) => void
}

interface WalletInfo {
  address: string
  chainId: string
  backendChain: string | null
  balance: string
  balanceFiat: string
  networkLabel: string
}

interface UseAgentWalletReturn {
  walletSnapshot: WalletSnapshot | null
  userWalletInfo: WalletInfo | null
  transactions: Transaction[]
  sidebarWallet: WalletSnapshot | WalletInfo | null
  refreshWallet: () => Promise<void>
  loadWalletOverview: (options?: { chain?: string; silent?: boolean }) => Promise<Record<string, unknown> | null>
  handleTransactionBuild: (result: { instruction: { method?: string; params?: unknown[] }; summary?: string; chain?: string }) => Promise<void>
  handleTransactionBroadcast: (result: { tx_hash?: string; txHash?: string; chain?: string; status?: string }) => Promise<void>
  handleWalletChanged: (event: { detail?: { address?: string } }, createSession: () => Promise<void>, setSessionReady: (ready: boolean) => void) => Promise<void>
}

/**
 * Hook for managing wallet state and transactions
 */
export const useAgentWallet = ({
  sessionId,
  activeChain,
  preferredChain,
  setPreferredChain,
  recordInteraction,
  appendMessage,
  scrollToBottom,
  setTransactions,
}: UseAgentWalletOptions): UseAgentWalletReturn => {
  const [walletSnapshot, setWalletSnapshot] = useState<WalletSnapshot | null>(null)
  const [userWalletInfo, setUserWalletInfo] = useState<WalletInfo | null>(null)
  const [transactions, setLocalTransactions] = useState<Transaction[]>(defaultTransactions)

  // 使用传入的 setTransactions 或本地状态
  const updateTransactions = setTransactions || setLocalTransactions

  const sidebarWallet = useMemo(() => {
    const normalizedActive = resolveBackendChain({ chain: activeChain }) || activeChain

    const normalizedAgentChain = walletSnapshot
      ? resolveBackendChain({ chain: walletSnapshot.chain })
      : null

    const normalizedUserChain = userWalletInfo
      ? resolveBackendChain({
          chain: userWalletInfo.backendChain,
          chainId: userWalletInfo.chainId,
        })
      : null

    if (normalizedActive) {
      if (userWalletInfo && normalizedUserChain === normalizedActive) {
        if (!walletSnapshot || normalizedAgentChain !== normalizedActive) {
          return userWalletInfo
        }
      }

      if (walletSnapshot && normalizedAgentChain === normalizedActive) {
        return walletSnapshot
      }

      if (userWalletInfo && normalizedUserChain === normalizedActive) {
        return userWalletInfo
      }
    }

    return walletSnapshot || userWalletInfo
  }, [activeChain, userWalletInfo, walletSnapshot])

  const refreshWallet = useCallback(async () => {
    try {
      const info = await walletService.getCurrentWalletInfo()
      if (!info) {
        setUserWalletInfo(null)
        recordInteraction('wallet_refresh', { connected: false }, '刷新资产')
        return
      }

      let balance = '0'
      try {
        balance = await walletService.getBalance(info.address)
      } catch (balanceError) {
        console.warn('Failed to get wallet balance:', balanceError)
        balance = '0'
      }

      const backendChain = resolveBackendChain({ chainId: info.chainId }) || preferredChain || null
      if (backendChain && backendChain !== preferredChain) {
        setPreferredChain(backendChain)
      }

      const snapshot: WalletInfo = {
        address: info.address,
        chainId: info.chainId,
        backendChain,
        balance,
        balanceFiat: (parseFloat(balance || '0') * 3400).toFixed(2),
        networkLabel:
          mapChainLabel(backendChain) ||
          ({
            '0x1': 'Ethereum Mainnet',
            '0x14a34': 'Base Sepolia',
            '0x2105': 'Base Mainnet',
            '0xaa36a7': 'Ethereum Sepolia',
          } as Record<string, string>)[info.chainId] ||
          `Chain ${info.chainId}`,
      }

      setUserWalletInfo(snapshot)
      recordInteraction(
        'wallet_refresh',
        {
          connected: true,
          balance,
          token: 'ETH',
          chain: backendChain,
        },
        '刷新资产',
      )
    } catch (error) {
      console.error('Failed to refresh wallet info:', error)
    }
  }, [preferredChain, recordInteraction, setPreferredChain])

  const loadWalletOverview = useCallback(
    async (options: { chain?: string; silent?: boolean } = {}) => {
      if (!sessionId) {
        return null
      }

      try {
        const targetChain = options.chain || activeChain
        const payload = {
          session_id: sessionId,
          ...(targetChain ? { chain: targetChain } : {}),
        }
        const { metadata } = await requestMcpUiResource(MCP_UI_TARGETS.walletOverview, payload)

        if (!metadata) {
          return null
        }

        const metadataChain =
          resolveBackendChain({ chain: metadata.chain }) ||
          (targetChain ? resolveBackendChain({ chain: targetChain }) : null)

        if (
          metadataChain &&
          metadataChain !== preferredChain &&
          (!activeChain || metadataChain === activeChain)
        ) {
          setPreferredChain(metadataChain)
        }

        const wallets = Array.isArray(metadata.wallets) ? metadata.wallets : []
        const requestedChain = payload.chain
        const primaryWallet =
          metadata.wallet ||
          (requestedChain ? wallets.find((item: { chain?: string }) => item.chain === requestedChain) : null) ||
          wallets[0] ||
          null

        if (primaryWallet) {
          const rawBalance = primaryWallet.balance ?? '0'
          const balanceString =
            typeof rawBalance === 'string' ? rawBalance : (rawBalance?.toString?.() ?? '0')
          const balanceNumeric = parseFloat(balanceString)
          const rawChain = primaryWallet.chain || metadata.chain || targetChain || 'ethereum'
          const normalizedChain = resolveBackendChain({ chain: rawChain }) || rawChain || 'ethereum'
          const tokenSymbol = normalizedChain === 'sol' ? 'SOL' : 'ETH'

          setWalletSnapshot({
            address: primaryWallet.address || '0x0000',
            balance: balanceString,
            balanceFiat: estimateFiatValue(balanceNumeric, tokenSymbol),
            networkLabel: mapChainLabel(normalizedChain),
            chain: normalizedChain,
            token: tokenSymbol,
          })

          if (
            normalizedChain &&
            normalizedChain !== preferredChain &&
            (!activeChain || normalizedChain === activeChain)
          ) {
            setPreferredChain(normalizedChain)
          }

          const txSource = Array.isArray(primaryWallet.transactions)
            ? primaryWallet.transactions
            : Array.isArray(metadata.transactions)
              ? metadata.transactions
              : []

          const normalizedTxs = txSource
            .map(
              (item: unknown, index: number) =>
                normalizeTransaction(item as Record<string, unknown>, tokenSymbol) ||
                normalizeTransaction({ ...(item as Record<string, unknown>), id: `${normalizedChain}_${index}` }, tokenSymbol),
            )
            .filter(Boolean) as Transaction[]

          if (normalizedTxs.length > 0) {
            updateTransactions(normalizedTxs)
          }
        }

        return metadata
      } catch (error) {
        if (!options.silent) {
          console.error('Failed to load wallet overview:', error)
        }
        return null
      }
    },
    [activeChain, preferredChain, sessionId, setPreferredChain, updateTransactions],
  )

  const handleTransactionBuild = useCallback(
    async (result: { instruction?: { method?: string; params?: unknown[] }; summary?: string; chain?: string }) => {
      if (!result || !result.instruction) {
        return
      }

      const instruction = result.instruction

      try {
        const txResponse = await walletService.executeInstruction(instruction as { method: string; params: unknown[] })
        const txHash =
          typeof txResponse === 'string'
            ? txResponse
            : (txResponse as { txHash?: string; transactionHash?: string })?.txHash || (txResponse as { transactionHash?: string })?.transactionHash

        const chain =
          resolveBackendChain({
            chain: result.chain,
            chainId: userWalletInfo?.chainId,
          }) ||
          activeChain ||
          'eth'

        if (chain && chain !== preferredChain) {
          setPreferredChain(chain)
        }

        const agentWalletChain = chain.startsWith('base')
          ? 'base'
          : chain.startsWith('polygon')
            ? 'polygon'
            : chain.startsWith('eth')
              ? 'ethereum'
              : null

        const txParams = (instruction.params && (instruction.params as unknown[])[0]) as Record<string, string> || {}

        recordInteraction('wallet_instruction', {
          instruction: instruction.method || 'eth_sendTransaction',
          chain,
          txHash,
        })

        appendMessage({
          id: `tx_success_${Date.now()}`,
          type: 'assistant',
          content: result.summary
            ? `✅ ${result.summary}\n交易哈希：${txHash}`
            : `✅ 交易已提交，哈希：${txHash}`,
          timestamp: Date.now(),
          source: 'system',
        })
        scrollToBottom()

        const tokenSymbol = chain.startsWith('sol') ? 'SOL' : 'ETH'
        const txRecord = {
          hash: txHash as string,
          from: txParams.from,
          to: txParams.to,
          value: formatWeiHexToEth(txParams.value),
          token: tokenSymbol,
          type: 'send' as const,
          status: 'pending' as const,
          timestamp: Date.now(),
          chain,
          counterparty: txParams.to || txParams.from,
        }

        const normalizedTx =
          normalizeTransaction(
            {
              ...txRecord,
              id: txHash,
              direction: 'out',
              amount: txRecord.value,
            },
            tokenSymbol,
          ) || {
            id: txHash as string,
            direction: 'out',
            amount: txRecord.value,
            token: tokenSymbol,
            counterparty: txRecord.counterparty || '未知地址',
            status: txRecord.status,
            statusLabel: '已提交',
            timestamp: txRecord.timestamp,
            hash: txHash as string,
          }

        updateTransactions((prev) => [normalizedTx, ...prev])

        if (agentWalletChain) {
          try {
            await agentService.recordAgentTransaction(sessionId, agentWalletChain, {
              ...txRecord,
              rawValue: txParams.value,
            })
          } catch (error) {
            console.error('Failed to record agent transaction:', error)
          }
        }

        await refreshWallet()
      } catch (error) {
        console.error('Failed to execute transaction instruction:', error)
        appendMessage({
          id: `tx_error_${Date.now()}`,
          type: 'assistant',
          content: `❌ 交易发送失败：${error instanceof Error ? error.message : String(error)}`,
          timestamp: Date.now(),
          source: 'error',
        })
        scrollToBottom()
      }
    },
    [
      activeChain,
      appendMessage,
      preferredChain,
      recordInteraction,
      refreshWallet,
      scrollToBottom,
      sessionId,
      setPreferredChain,
      updateTransactions,
      userWalletInfo?.chainId,
    ],
  )

  const handleTransactionBroadcast = useCallback(
    async (result: { tx_hash?: string; txHash?: string; chain?: string; status?: string }) => {
      if (!result) {
        return
      }
      const txHash = result.tx_hash || result.txHash
      if (!txHash) {
        return
      }

      const chain = result.chain || 'eth'

      updateTransactions((prev) =>
        prev.map((item) =>
          item.hash === txHash
            ? {
                ...item,
                status: result.status || 'submitted',
              }
            : item,
        ),
      )

      recordInteraction('wallet_instruction', {
        instruction: 'broadcast_transaction',
        chain,
        txHash,
      })

      appendMessage({
        id: `tx_broadcast_${Date.now()}`,
        type: 'assistant',
        content: `📡 交易已广播：${txHash}`,
        timestamp: Date.now(),
        source: 'system',
      })
      scrollToBottom()
      await loadWalletOverview({ silent: true })
    },
    [appendMessage, loadWalletOverview, recordInteraction, scrollToBottom, updateTransactions],
  )

  const handleWalletChanged = useCallback(
    async (event: { detail?: { address?: string } }, createSession: () => Promise<void>, setSessionReady: (ready: boolean) => void) => {
      const detail = event?.detail
      recordInteraction('wallet_event', { address: detail?.address })
      await refreshWallet()
      await createSession()
      setSessionReady(true)
    },
    [recordInteraction, refreshWallet],
  )

  // Setup wallet chain listener
  useEffect(() => {
    if (!walletService.isWalletAvailable()) {
      return undefined
    }

    const handleChainChanged = (chainId: string) => {
      const backendChain = mapChainIdToBackendChain(chainId) || null
      recordInteraction('wallet_event', {
        event: 'chain_changed',
        chainId,
        backendChain,
      })
      if (backendChain && backendChain !== preferredChain) {
        setPreferredChain(backendChain)
      }
      void refreshWallet()
      void loadWalletOverview({ chain: backendChain, silent: true })
    }

    const setupChainListener = async () => {
      try {
        await walletService.onChainChanged(handleChainChanged)
      } catch (error) {
        console.warn('Failed to setup chain listener:', error)
      }
    }

    setupChainListener()

    return () => {
      try {
        walletService.removeListener('chainChanged', handleChainChanged)
      } catch (error) {
        console.warn('Failed to remove chain listener:', error)
      }
    }
  }, [loadWalletOverview, preferredChain, recordInteraction, refreshWallet, setPreferredChain])

  return {
    walletSnapshot,
    userWalletInfo,
    transactions,
    sidebarWallet: sidebarWallet as WalletSnapshot | WalletInfo | null,
    refreshWallet,
    loadWalletOverview,
    handleTransactionBuild,
    handleTransactionBroadcast,
    handleWalletChanged,
  }
}

export default useAgentWallet
