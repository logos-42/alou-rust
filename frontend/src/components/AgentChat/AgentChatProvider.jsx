import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'
import { walletService } from '@/services/walletService'
import agentService from '@/services/agentService'
import apiClient from '@/services/api'
import { MCP_UI_TARGETS, requestMcpUiResource } from '@/services/mcpUiService'
import { useAgentStream } from '@/hooks/useAgentStream'
import {
  ACTION_LABELS,
  NODE_BOUNDARY,
  defaultTransactions,
  formatWeiHexToEth,
  mapChainIdToBackendChain,
  mapChainLabel,
  normalizeTransaction,
  resolveBackendChain,
  useToolCallHandler,
  estimateFiatValue,
} from '@/hooks/useAgentChat'
import {
  buildChannelFromAgent,
  extractAgentTarget,
  extractErrorMessage,
  computeAgentProfile,
} from './agentUtils'
import { AgentChatContext } from './AgentChatContext'

export const AgentChatProvider = ({ children }) => {
  const navigate = useNavigate()
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const logout = useAuthStore((state) => state.logout)
  const userNameGetter = useAuthStore((state) => state.userName)
  const { t, initLanguage, setLanguage, currentLanguage } = useI18n()

  // UI State
  const [isDarkMode, setIsDarkMode] = useState(false)
  const [isSidebarCollapsed, setSidebarCollapsed] = useState(true)
  const [isLeftSidebarCollapsed, setLeftSidebarCollapsed] = useState(false)
  const [isInteractionCollapsed, setInteractionCollapsed] = useState(true)
  const [isConversationVisible, setConversationVisible] = useState(false)
  const [viewportWidth, setViewportWidth] = useState(
    typeof window !== 'undefined' ? window.innerWidth : 1440,
  )

  // Connection State
  const [connectionStatus, setConnectionStatus] = useState('disconnected')

  // Message State
  const [messages, setMessages] = useState([])
  const [currentMessage, setCurrentMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // Session State
  const [sessionId, setSessionId] = useState(
    `frontend_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
  )
  const [isSessionReady, setSessionReady] = useState(false)

  // Channel State
  const [channelKeyword, setChannelKeyword] = useState('')
  const [activeChannelId, setActiveChannelId] = useState(null)
  const [channels, setChannels] = useState([])
  const [showDiapPanel, setShowDiapPanel] = useState(true)
  const [selectedAgent, setSelectedAgent] = useState(null)
  const [isChannelLoading, setChannelLoading] = useState(false)
  const [channelError, setChannelError] = useState(null)
  const channelRequestIdRef = useRef(0)
  const searchDebounceRef = useRef(null)
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)

  // Wallet State
  const [walletSnapshot, setWalletSnapshot] = useState(null)
  const [preferredChain, setPreferredChain] = useState(null)
  const [userWalletInfo, setUserWalletInfo] = useState(null)
  const [transactions, setTransactions] = useState(defaultTransactions)

  // Interaction State
  const [interactionLogs, setInteractionLogs] = useState([])

  // Agent Position State
  const dragStateRef = useRef({ dragging: false, offsetX: 0, offsetY: 0, moved: false })
  const agentPositionRef = useRef({ x: 0, y: 0 })
  const [agentPosition, setAgentPosition] = useState({ x: 0, y: 0 })

  // UI Resource State
  const [uiResource, setUiResource] = useState(null)
  const [isUiModalOpen, setUiModalOpen] = useState(false)
  const finalEventHandledRef = useRef(null)
  const contextEventsRef = useRef([])

  // Refs
  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)

  // Derived values
  const connectionStatusLabel = useMemo(() => {
    if (connectionStatus === 'connected') return '已连接'
    if (connectionStatus === 'connecting') return '连接中'
    if (connectionStatus === 'error') return '服务异常'
    return '未连接'
  }, [connectionStatus])

  const agentProfile = useMemo(() => computeAgentProfile(selectedAgent), [selectedAgent])

  const activeChain = useMemo(
    () =>
      resolveBackendChain({
        chain: preferredChain || walletSnapshot?.chain,
        chainId: userWalletInfo?.chainId,
      }),
    [preferredChain, userWalletInfo?.chainId, walletSnapshot?.chain],
  )

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

  const filteredChannels = useMemo(() => {
    if (!channelKeyword.trim()) {
      return channels
    }
    const lower = channelKeyword.toLowerCase()
    return channels.filter((channel) => channel.name.toLowerCase().includes(lower))
  }, [channelKeyword, channels])

  const agentStyle = useMemo(
    () => ({
      transform: `translate(calc(-50% + ${agentPosition.x}px), calc(-50% + ${agentPosition.y}px))`,
    }),
    [agentPosition],
  )

  const consoleDockStyle = useMemo(() => {
    if (viewportWidth <= 1024) {
      return { margin: '0 1rem 0 1rem' }
    }
    const leftWidth = isLeftSidebarCollapsed ? 84 : viewportWidth <= 1280 ? 240 : 300
    const rightWidth = isSidebarCollapsed ? 80 : 340
    return {
      marginLeft: `${leftWidth + 24}px`,
      marginRight: `${rightWidth + 24}px`,
    }
  }, [viewportWidth, isSidebarCollapsed, isLeftSidebarCollapsed])

  const languageLabel = useMemo(
    () => (currentLanguage === 'zh' ? '中 / EN' : 'EN / 中'),
    [currentLanguage],
  )

  const userName = useMemo(() => userNameGetter?.() ?? 'User', [userNameGetter])

  // Continue with all the callback functions...
  // This is a large component, so we'll keep the logic here
  // and expose it through context

  const value = {
    // State
    isDarkMode,
    setIsDarkMode,
    isSidebarCollapsed,
    setSidebarCollapsed,
    isLeftSidebarCollapsed,
    setLeftSidebarCollapsed,
    isInteractionCollapsed,
    setInteractionCollapsed,
    isConversationVisible,
    setConversationVisible,
    viewportWidth,
    setViewportWidth,
    connectionStatus,
    setConnectionStatus,
    connectionStatusLabel,
    messages,
    setMessages,
    currentMessage,
    setCurrentMessage,
    isLoading,
    setIsLoading,
    sessionId,
    setSessionId,
    isSessionReady,
    setSessionReady,
    channelKeyword,
    setChannelKeyword,
    activeChannelId,
    setActiveChannelId,
    channels,
    setChannels,
    showDiapPanel,
    setShowDiapPanel,
    selectedAgent,
    setSelectedAgent,
    isChannelLoading,
    setChannelLoading,
    channelError,
    setChannelError,
    isCreateAgentModalOpen,
    setCreateAgentModalOpen,
    walletSnapshot,
    setWalletSnapshot,
    preferredChain,
    setPreferredChain,
    userWalletInfo,
    setUserWalletInfo,
    transactions,
    setTransactions,
    interactionLogs,
    setInteractionLogs,
    agentPosition,
    setAgentPosition,
    uiResource,
    setUiResource,
    isUiModalOpen,
    setUiModalOpen,
    agentProfile,
    activeChain,
    sidebarWallet,
    filteredChannels,
    agentStyle,
    consoleDockStyle,
    languageLabel,
    userName,
    isAuthenticated,
    logout,
    navigate,
    setLanguage,
    currentLanguage,
    // Refs
    canvasRef,
    conversationOverlayRef,
    consoleDockRef,
    dragStateRef,
    agentPositionRef,
    channelRequestIdRef,
    searchDebounceRef,
    finalEventHandledRef,
    contextEventsRef,
  }

  return <AgentChatContext.Provider value={value}>{children}</AgentChatContext.Provider>
}

