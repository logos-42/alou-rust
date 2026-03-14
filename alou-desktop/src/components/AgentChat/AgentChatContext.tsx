import React, { createContext, useContext, useCallback, useMemo } from 'react'
import { resolveBackendChain } from '@/hooks/useAgentChat'
import { computeAgentProfile } from './agentUtils'

const AgentChatContext = createContext(null)

export const useAgentChatContext = () => {
  const context = useContext(AgentChatContext)
  if (!context) {
    throw new Error('useAgentChatContext must be used within AgentChatProvider')
  }
  return context
}

export const AgentChatProvider = ({ children, value }) => {
  // Compute derived values
  const connectionStatusLabel = useMemo(() => {
    if (value.connectionStatus === 'connected') return '已连接'
    if (value.connectionStatus === 'connecting') return '连接中'
    if (value.connectionStatus === 'error') return '服务异常'
    return '未连接'
  }, [value.connectionStatus])

  const agentProfile = useMemo(
    () => computeAgentProfile(value.selectedAgent),
    [value.selectedAgent],
  )

  const activeChain = useMemo(
    () =>
      resolveBackendChain({
        chain: value.preferredChain || value.walletSnapshot?.chain,
        chainId: value.userWalletInfo?.chainId,
      }),
    [value.preferredChain, value.userWalletInfo?.chainId, value.walletSnapshot?.chain],
  )

  const sidebarWallet = useMemo(() => {
    const normalizedActive = resolveBackendChain({ chain: activeChain }) || activeChain

    const normalizedAgentChain = value.walletSnapshot
      ? resolveBackendChain({ chain: value.walletSnapshot.chain })
      : null

    const normalizedUserChain = value.userWalletInfo
      ? resolveBackendChain({
          chain: value.userWalletInfo.backendChain,
          chainId: value.userWalletInfo.chainId,
        })
      : null

    if (normalizedActive) {
      if (value.userWalletInfo && normalizedUserChain === normalizedActive) {
        if (!value.walletSnapshot || normalizedAgentChain !== normalizedActive) {
          return value.userWalletInfo
        }
      }

      if (value.walletSnapshot && normalizedAgentChain === normalizedActive) {
        return value.walletSnapshot
      }

      if (value.userWalletInfo && normalizedUserChain === normalizedActive) {
        return value.userWalletInfo
      }
    }

    return value.walletSnapshot || value.userWalletInfo
  }, [activeChain, value.userWalletInfo, value.walletSnapshot])

  const filteredChannels = useMemo(() => {
    if (!value.channelKeyword.trim()) {
      return value.channels
    }
    const lower = value.channelKeyword.toLowerCase()
    return value.channels.filter((channel) => channel.name.toLowerCase().includes(lower))
  }, [value.channelKeyword, value.channels])

  const agentStyle = useMemo(
    () => ({
      transform: `translate(calc(-50% + ${value.agentPosition.x}px), calc(-50% + ${value.agentPosition.y}px))`,
    }),
    [value.agentPosition],
  )

  const showConversationPanel = value.isConversationVisible

  const contextValue = useMemo(
    () => ({
      ...value,
      connectionStatusLabel,
      agentProfile,
      activeChain,
      sidebarWallet,
      filteredChannels,
      agentStyle,
      showConversationPanel,
    }),
    [
      value,
      connectionStatusLabel,
      agentProfile,
      activeChain,
      sidebarWallet,
      filteredChannels,
      agentStyle,
      showConversationPanel,
    ],
  )

  return <AgentChatContext.Provider value={contextValue}>{children}</AgentChatContext.Provider>
}

