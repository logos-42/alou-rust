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
}) => {
  const [walletSnapshot, setWalletSnapshot] = useState(null)
  const [userWalletInfo, setUserWalletInfo] = useState(null)
  const [transactions, setLocalTransactions] = useState(defaultTransactions)

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

      const snapshot = {
        address: info.address,
        chainId: info.chainId,
        backendChain,
        balance,
        balanceFiat: (parseFloat(balance || '0') * 3400).toFixed(2),
        networkLabel:
          mapChainLabel(backendChain) ||
          {
            '0x1': 'Ethereum Mainnet',
            '0x14a34': 'Base Sepolia',
            '0x2105': 'Base Mainnet',
            '0xaa36a7': 'Ethereum Sepolia',
          }[info.chainId] ||
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
    async (options = {}) => {
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
          (requestedChain ? wallets.find((item) => item.chain === requestedChain) : null) ||
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
              (item, index) =>
                normalizeTransaction(item, tokenSymbol) ||
                normalizeTransaction({ ...item, id: `${normalizedChain}_${index}` }, tokenSymbol),
            )
            .filter(Boolean)

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
    async (result) => {
      if (!result || !result.instruction) {
        return
      }

      const instruction = result.instruction

      try {
        const txResponse = await walletService.executeInstruction(instruction)
        const txHash =
          typeof txResponse === 'string'
            ? txResponse
            : txResponse?.txHash || txResponse?.transactionHash

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

        const txParams = (instruction.params && instruction.params[0]) || {}

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
          hash: txHash,
          from: txParams.from,
          to: txParams.to,
          value: formatWeiHexToEth(txParams.value),
          token: tokenSymbol,
          type: 'send',
          status: 'pending',
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
            id: txHash,
            direction: 'out',
            amount: txRecord.value,
            token: tokenSymbol,
            counterparty: txRecord.counterparty || '未知地址',
            status: txRecord.status,
            statusLabel: '已提交',
            timestamp: txRecord.timestamp,
            hash: txHash,
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
    async (result) => {
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
    async (event, createSession, setSessionReady) => {
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

    const handleChainChanged = (chainId) => {
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
    sidebarWallet,
    refreshWallet,
    loadWalletOverview,
    handleTransactionBuild,
    handleTransactionBroadcast,
    handleWalletChanged,
  }
}

