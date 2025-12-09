import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'
import agentService from '@/services/agentService'
import { MCP_UI_TARGETS } from '@/services/mcpUiService'
import { useAgentStream } from '@/hooks/useAgentStream'

// Components
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import DiapPanelToggle from '@/components/agent/DiapPanelToggle'
import McpModal from '@/components/mcp/McpModal'
import CreateAgentModal from '@/components/CreateAgentModal'
import InviteAgentModal from '@/components/InviteAgentModal'
import AgentProfilePanel from '@/components/AgentProfilePanel'
import TranslationIcon from '@/assets/icon_翻译.png'

// Hooks
import { useAgentUI } from './AgentChat/useAgentUI'
import { useAgentConnection } from './AgentChat/useAgentConnection'
import { useAgentDrag } from './AgentChat/useAgentDrag'
import { useAgentMessages } from './AgentChat/useAgentMessages'
import { useAgentWallet } from './AgentChat/useAgentWallet'
import { useChannelManager } from './AgentChat/useChannelManager'
import { useAgentInvite } from './AgentChat/useAgentInvite'
import { useMultiAgentChat } from '@/hooks/useMultiAgentChat'

// Utils & Constants
import { resolveBackendChain, useToolCallHandler } from '@/hooks/useAgentChat'
import { computeAgentProfile, buildChannelFromAgent } from './AgentChat/agentUtils'
import './AgentChat/index.css'

/**
 * AgentChat - 主智能体聊天组件
 * 
 * 通过组合多个专用 hooks 来管理不同的功能域：
 * - useAgentUI: UI 状态（暗黑模式、侧边栏、交互日志等）
 * - useAgentConnection: 连接状态和会话管理
 * - useAgentDrag: 智能体节点拖拽
 * - useAgentMessages: 消息和对话管理
 * - useAgentWallet: 钱包状态和交易
 * - useChannelManager: 频道列表和智能体选择
 */
const AgentChat = () => {
  const navigate = useNavigate()

  // ==================== Auth Store ====================
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const logout = useAuthStore((state) => state.logout)
  const userNameGetter = useAuthStore((state) => state.userName)
  const userName = useMemo(() => userNameGetter?.() ?? 'User', [userNameGetter])

  // ==================== i18n ====================
  const { t, initLanguage, setLanguage, currentLanguage } = useI18n()
  const languageLabel = useMemo(
    () => (currentLanguage === 'zh' ? '中 / EN' : 'EN / 中'),
    [currentLanguage],
  )

  // ==================== Refs ====================
  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)

  // ==================== Shared State ====================
  // 这些状态需要在多个 hooks 之间共享，所以保留在父组件
  const [preferredChain, setPreferredChain] = useState(null)
  const [channelKeyword, setChannelKeyword] = useState('')
  const [channels, setChannels] = useState([])
  const [activeChannelId, setActiveChannelId] = useState(null)
  const [selectedAgent, setSelectedAgent] = useState(null)
  const [isChannelLoading, setChannelLoading] = useState(false)
  const [channelError, setChannelError] = useState(null)
  const [selectedModelType, setSelectedModelType] = useState(null)
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)

  // ==================== 1. UI State Hook ====================
  const uiState = useAgentUI({
    sessionId: null, // 会在 connection 建立后更新
    viewportWidth: typeof window !== 'undefined' ? window.innerWidth : 1440,
  })

  const {
    isDarkMode,
    isSidebarCollapsed,
    isLeftSidebarCollapsed,
    isInteractionCollapsed,
    isConversationVisible,
    setConversationVisible,
    viewportWidth,
    interactionLogs,
    uiResource,
    isUiModalOpen,
    showDiapPanel,
    setShowDiapPanel,
    contextEventsRef,
    recordInteraction,
    openUiResource,
    fetchAndOpenUiResource,
    closeUiResource,
    handleUiAction,
    toggleSidebar,
    toggleInteractionPanel,
    toggleDarkMode,
    toggleLeftSidebar,
    openConversationPanel,
    closeConversationPanel,
    handleResize,
    consoleDockStyle,
  } = uiState

  // ==================== 2. Connection State Hook ====================
  const connectionState = useAgentConnection({
    activeChain: preferredChain,
    preferredChain,
    setPreferredChain,
  })

  const {
    connectionStatus,
    setConnectionStatus,
    connectionStatusLabel,
      sessionId,
    setSessionId,
    isSessionReady,
    setSessionReady,
    checkConnection,
    createSession,
  } = connectionState

  // ==================== 3. Drag State Hook ====================
  const dragState = useAgentDrag({
    canvasRef,
    recordInteraction,
    openConversationPanel,
  })

  const {
    agentPosition,
    clampPosition,
    startDrag,
    onDrag,
    stopDrag,
    handleGlobalPointerUp,
    handleAgentActivate,
  } = dragState

  // ==================== 4. Wallet State Hook ====================
  // 注意：需要先定义 walletState，因为 messageState 的 handleToolCalls 需要它
  const walletState = useAgentWallet({
    sessionId,
    activeChain: preferredChain,
    preferredChain,
    setPreferredChain,
    recordInteraction,
    appendMessage: null, // 会通过 handleToolCalls 传递
    scrollToBottom: null, // 会通过 handleToolCalls 传递
    setTransactions: undefined,
        })

  const {
    walletSnapshot,
    userWalletInfo,
    transactions,
    sidebarWallet,
    refreshWallet,
    loadWalletOverview,
    handleTransactionBuild,
    handleTransactionBroadcast,
  } = walletState

  // ==================== 5. Message State Hook ====================
  const messageState = useAgentMessages({
    sessionId,
    setSessionId,
    activeChain: preferredChain,
    activeChannelId,
    selectedAgent,
    isSessionReady,
    createSession,
    setSessionReady,
    recordInteraction,
    handleToolCalls: useCallback((toolCalls) => {
      // 工具调用处理会在下面定义
    }, []),
    conversationOverlayRef,
    consoleDockRef,
    contextEventsRef,
  })

  const {
    messages,
    messagesByChannel,
    setMessagesForChannel,
    currentMessage,
    setCurrentMessage,
    isLoading,
    setIsLoading,
    // 多智能体独立执行空间
    loadingByAgent,
    isAgentLoading,
    sendMessageToAgent,
    appendMessage,
    scrollToBottom,
    sendMessage: baseSendMessage,
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
  } = messageState

  // ==================== Tool Call Handler ====================
  const handleToolCalls = useToolCallHandler({
    handleTransactionBuild,
    handleTransactionBroadcast,
    refreshWallet,
    recordInteraction,
    appendMessage,
    scrollToBottom,
    openUiResource,
  })

  // ==================== 6. Channel Manager Hook ====================
  const channelManager = useChannelManager({
    sessionId,
    isSessionReady,
    channelKeyword,
    setChannelKeyword,
    channels,
    setChannels,
    activeChannelId,
    setActiveChannelId,
    selectedAgent,
    setSelectedAgent,
    isChannelLoading,
    setChannelLoading,
    channelError,
    setChannelError,
    recordInteraction,
    openConversationPanel,
    loadMessagesFromIpfs,
  })

  const {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    saveAgentToStorage,
    deleteChannel,
  } = channelManager

  // ==================== 7. Invite State Hook ====================
  const inviteState = useAgentInvite({
    deleteChannel,
    resolveExistingAgentTarget,
    recordInteraction,
  })

  const {
    isInviteModalOpen,
    inviteTargetChannel,
    handleInviteToChannel,
    closeInviteModal,
    handleDeleteChannel,
    handleInviteSubmit,
  } = inviteState

  // ==================== 8. Multi-Agent Coordinator ====================
  // 获取本地 DIAP 身份用于多智能体通信
  const localIdentity = useMemo(() => {
    if (selectedAgent?.diapIdentity) {
      return selectedAgent.diapIdentity
    }
    // 尝试从 localStorage 获取
    if (sessionId && typeof window !== 'undefined') {
      const stored = localStorage.getItem(`diap_identity_${sessionId}`)
      if (stored) {
        try {
          return JSON.parse(stored)
        } catch {
          return null
        }
      }
    }
    return null
  }, [selectedAgent, sessionId])

  // 将所有频道中的智能体注册到协调器
  const registeredAgentsForCoordinator = useMemo(() => {
    return channels
      .filter(ch => ch.meta)
      .map(ch => ({
        id: ch.id,
        did: ch.meta.did,
        ipns: ch.meta.ipns,
        name: ch.meta.display_name || ch.meta.name,
        display_name: ch.meta.display_name || ch.meta.name,
        role_description: ch.meta.role_description,
        pubsub_topics: ch.meta.pubsub_topics || [],
      }))
  }, [channels])

  const multiAgentChat = useMultiAgentChat({
    localIdentity,
    registeredAgents: registeredAgentsForCoordinator,
    onAgentMessage: useCallback((agentId, message) => {
      console.log('[AgentChat] 收到智能体消息:', agentId, message)
      // 将智能体间消息添加到对应频道
      if (message.content) {
        appendMessage({
          id: message.id || `agent_${Date.now()}`,
          type: 'assistant',
          content: message.content,
          timestamp: message.timestamp || Date.now(),
          source: 'agent-coordinator',
          fromAgent: message.from,
        }, agentId)
      }
    }, [appendMessage]),
    onGroupMessage: useCallback((groupId, message) => {
      console.log('[AgentChat] 收到群聊消息:', groupId, message)
    }, []),
  })

  const {
    routeMessageToAgent,
    sendToAgent,
    analyzeIntent,
    isCoordinatorReady,
  } = multiAgentChat

  // ==================== Stream Events ====================
  const handleStreamEvent = useCallback(
    (event) => {
      if (!event) return

      recordInteraction(
        'stream_event',
        { event: event.event, payload: event.payload },
        event.label,
      )

      if (event.isFinal && event.timestamp) {
        if (Array.isArray(event.payload?.tool_calls)) {
          const uiCall = event.payload.tool_calls.find((call) => {
            const result = call?.result
            if (!result) return false
            if (Array.isArray(result.resources) && result.resources.length > 0) return true
            return Boolean(result.resource || result.uri)
          })

          if (uiCall?.result) {
            const result = uiCall.result
            if (Array.isArray(result.resources) && result.resources.length > 0) {
              openUiResource(result.resources[0], { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.resource) {
              openUiResource(result.resource, { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.uri) {
              openUiResource(result, { source: 'stream_final', toolCall: uiCall.name })
            }
          }
        }
      }
    },
    [openUiResource, recordInteraction],
  )

  const { status: streamStatus, events: streamEvents } = useAgentStream(sessionId, {
    enabled: isSessionReady,
    onEvent: handleStreamEvent,
  })

  // 同步 stream 状态到连接状态
  useEffect(() => {
    if (streamStatus === 'error') {
      setConnectionStatus('error')
    } else if (streamStatus === 'active' || streamStatus === 'completed') {
      setConnectionStatus('connected')
    } else if (streamStatus === 'polling') {
      setConnectionStatus('connecting')
    } else {
      setConnectionStatus('disconnected')
    }
  }, [streamStatus, setConnectionStatus])

  // ==================== Computed Values ====================
  const agentProfile = useMemo(() => computeAgentProfile(selectedAgent), [selectedAgent])

  const agentStyle = useMemo(
    () => ({
      transform: `translate(calc(-50% + ${agentPosition.x}px), calc(-50% + ${agentPosition.y}px))`,
    }),
    [agentPosition],
  )

  const filteredChannels = useMemo(() => {
    let filtered = channels

    if (channelKeyword.trim()) {
      const lower = channelKeyword.toLowerCase()
      filtered = filtered.filter((channel) => channel.name.toLowerCase().includes(lower))
    }

    if (selectedModelType) {
      filtered = filtered.filter((channel) => {
        const agentType = channel.meta?.agent_type || channel.meta?.model_type
        if (selectedModelType === 'claude') return agentType === 'claude_agent_sdk'
        if (selectedModelType === 'alou') return agentType === 'alou_agent' || agentType === 'alou'
        return true
      })
    }

    return filtered
  }, [channelKeyword, channels, selectedModelType])

  const showConversationPanel = isConversationVisible

  // ==================== Event Handlers ====================
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

  const createChannel = useCallback(() => {
    setCreateAgentModalOpen(true)
    recordInteraction('open_create_agent_modal')
  }, [recordInteraction])

  const closeCreateAgentModal = useCallback(() => {
        setCreateAgentModalOpen(false)
  }, [])

  const handleEarlyChannel = useCallback(
    (earlyMetadata) => {
      const channel = buildChannelFromAgent(earlyMetadata)
      if (channel) {
        // 确保 tempId 保存在 channel.meta 中用于后续匹配
        if (earlyMetadata.cid && earlyMetadata.cid.startsWith('temp_')) {
          channel.tempId = earlyMetadata.cid
          if (channel.meta) {
            channel.meta.tempId = earlyMetadata.cid
          }
        }
        
        setChannels((prev) => {
          // 移除可能重复的频道
          const others = prev.filter((item) => item.id !== channel.id)
          return [channel, ...others]
        })
        setActiveChannelId(channel.id)
        setSelectedAgent(earlyMetadata)
        console.log('[AgentChat] 早期频道已显示，tempId:', channel.tempId, '等待完整创建...')
      }
    },
    [setChannels, setActiveChannelId, setSelectedAgent],
  )

  const handleCreateAgentSubmit = useCallback(
    async ({ name, roleDescription, avatarCid, mcpConfigCid, mcpPorts, diapIdentity, tempId }) => {
      // 如果有 tempId，说明是后台更新，不需要显示 loading
      const isBackgroundUpdate = !!tempId
      if (!isBackgroundUpdate) {
        setChannelLoading(true)
      }
      setChannelError(null)
      recordInteraction('create_claude_agent', { name, isBackgroundUpdate })

      try {
        const walletAddress =
          typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
        const chainId =
          typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
        const detectedChain = resolveBackendChain({ chainId, chain: preferredChain })

        const result = await agentService.createClaudeAgent({
          sessionId,
          walletAddress,
          chain: detectedChain || preferredChain,
          name,
          roleDescription,
          avatarCid,
          mcpConfigCid,
          mcpPorts,
          diapIdentity,
        })

        const metadata = result.agent_metadata || {
          did: result.diap_identity?.did,
          cid: result.diap_identity?.cid,
          ipns: result.diap_identity?.ipns,
          agent_type: 'claude_agent_sdk',
          display_name: name,
          role_description: roleDescription,
          avatar_cid: avatarCid,
          mcp_config_cid: mcpConfigCid,
          mcp_ports: mcpPorts,
          diap_identity: diapIdentity,
          sessionId,
        }

        if (diapIdentity) {
          metadata.diapIdentity = diapIdentity
          metadata.did = metadata.did || diapIdentity.did
          metadata.cid = metadata.cid || diapIdentity.cid
          metadata.ipns = metadata.ipns || diapIdentity.ipns
        }

        const channel = buildChannelFromAgent(metadata)
        if (channel) {
          setChannels((prev) => {
            // 查找临时频道 - 使用多种方式匹配
            const tempIndex = prev.findIndex((item) => {
              // 1. 通过 tempId 属性匹配
              if (tempId && item.tempId === tempId) return true
              if (tempId && item.meta?.tempId === tempId) return true
              // 2. 通过 id 匹配（临时频道的 id 就是 tempId）
              if (tempId && item.id === tempId) return true
              // 3. 通过 meta.cid 匹配
              if (tempId && item.meta?.cid === tempId) return true
              // 4. 检查是否是任何临时频道（以 temp_ 开头）
              if (item.id && item.id.startsWith('temp_')) return true
              if (item.meta?.cid && item.meta.cid.startsWith('temp_')) return true
              return false
            })

            if (tempIndex >= 0) {
              // 更新临时频道为完整频道
              const updated = [...prev]
              const oldChannel = updated[tempIndex]
              console.log('[AgentChat] 找到临时频道:', oldChannel.id, '-> 更新为:', channel.id)
              updated[tempIndex] = channel
              return updated
            }

            // 没有找到临时频道，检查是否已存在相同 ID 的频道
            const existingIndex = prev.findIndex((item) => item.id === channel.id)
            if (existingIndex >= 0) {
              const updated = [...prev]
              updated[existingIndex] = channel
              console.log('[AgentChat] 更新已存在的频道:', channel.id)
              return updated
            }

            // 添加新频道
            console.log('[AgentChat] 添加新频道:', channel.id)
            return [channel, ...prev]
          })
          setActiveChannelId(channel.id)
        }

        setSelectedAgent(metadata)

        // 保存到本地存储
        saveAgentToStorage(metadata)

        console.log('[AgentChat] 智能体创建/更新完成:', metadata.did || metadata.cid)
        return result
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        if (!isBackgroundUpdate) {
          setChannelError(message)
        }
        recordInteraction('create_claude_agent_failed', { error: message })
        console.error('[AgentChat] 创建智能体失败:', message)
        throw new Error(message)
      } finally {
        if (!isBackgroundUpdate) {
          setChannelLoading(false)
        }
      }
    },
    [preferredChain, recordInteraction, saveAgentToStorage, sessionId, setChannels, setActiveChannelId, setSelectedAgent],
  )

  // ==================== Send Message with Tool Calls ====================
  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) return

    if (!isSessionReady) {
      await createSession()
      setSessionReady(true)
    }

    const userMessage = {
      id: `user_${Date.now()}`,
      type: 'user',
      content: text,
      timestamp: Date.now(),
    }

    appendMessage(userMessage)
    setCurrentMessage('')
    setIsLoading(true)
    recordInteraction('user_message', { content: text })
    scrollToBottom()

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null

      const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
        (import.meta.env.DEV ? 'http://127.0.0.1:8787' : 'https://alou-edge.yuanjieliu65.workers.dev')
      
      const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          message: text,
          wallet_address: walletAddress || undefined,
          chain: preferredChain || undefined,
          context_events: contextSnapshot,
        }),
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      const data = await response.json()

      if (data.tool_calls) {
        await handleToolCalls(data.tool_calls)
      }

      const assistantMessage = {
        id: `assistant_${Date.now()}`,
        type: 'assistant',
        content: data.content || data.response || '收到响应',
        timestamp: data.timestamp || Date.now(),
        source: data.source || 'alou-edge',
      }
      appendMessage(assistantMessage)

      if (data.session_id) {
        setSessionId(data.session_id)
      }
    } catch (error) {
      appendMessage({
        id: `error_${Date.now()}`,
        type: 'assistant',
        content: `❌ 抱歉，发生了错误：${error instanceof Error ? error.message : '未知错误'}`,
        timestamp: Date.now(),
        source: 'error',
      })
    } finally {
      setIsLoading(false)
      scrollToBottom()
      consoleDockRef.current?.adjustInputHeight?.()
    }
  }, [
    appendMessage,
    contextEventsRef,
    createSession,
    currentMessage,
    handleToolCalls,
    isLoading,
    isSessionReady,
    preferredChain,
    recordInteraction,
    scrollToBottom,
    sessionId,
    setCurrentMessage,
    setIsLoading,
    setSessionId,
    setSessionReady,
  ])

  // ==================== Bootstrap Effect ====================
  useEffect(() => {
    const bootstrap = async () => {
      try {
        initLanguage()
        await Promise.all([
          checkConnection(),
          createSession().then(() => setSessionReady(true)),
          refreshWallet().catch((err) => {
            console.warn('Failed to refresh wallet:', err)
          }),
        ])
      } catch (error) {
        console.error('Error in AgentChat initialization:', error)
      }
    }
    bootstrap()

    // Event listeners
    if (typeof window !== 'undefined') {
      window.addEventListener('wallet-changed', handleWalletChanged)
      window.addEventListener('resize', handleResize)
      window.addEventListener('pointerup', handleGlobalPointerUp)
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('wallet-changed', handleWalletChanged)
        window.removeEventListener('resize', handleResize)
        window.removeEventListener('pointerup', handleGlobalPointerUp)
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ==================== Shell Class Name ====================
  const shellClassName = [
    'app-shell',
    isDarkMode ? 'dark-mode' : '',
    isLeftSidebarCollapsed ? 'collapsed-left' : '',
  ]
    .filter(Boolean)
    .join(' ')

  // ==================== Render ====================
  return (
    <div className={shellClassName}>
      <ChatHeader
        connectionStatus={connectionStatus}
        isDarkMode={isDarkMode}
        isAuthenticated={isAuthenticated}
        userName={userName}
        onToggleTheme={toggleDarkMode}
        onGoToLogin={goToLogin}
        onGoToWallet={goToWallet}
        onLogout={handleLogout}
      />

      <div className="workspace">
        <AgentSidebarLeft
          channels={filteredChannels}
          activeChannelId={activeChannelId}
          keyword={channelKeyword}
          isLoading={isChannelLoading}
          errorMessage={channelError}
          onKeywordChange={handleChannelKeywordChange}
          onRefresh={refreshChannels}
          onSelectChannel={selectChannel}
          onCreateChannel={createChannel}
          onDeleteChannel={handleDeleteChannel}
          onInviteToChannel={handleInviteToChannel}
          isCollapsed={isLeftSidebarCollapsed}
          onToggleCollapse={toggleLeftSidebar}
          selectedModelType={selectedModelType}
          onModelTypeChange={setSelectedModelType}
          onShowIdentityPanel={() => setShowDiapPanel(true)}
        />

        <div className="agent-center">
          <div className={`conversation-stack ${showConversationPanel ? 'open' : ''}`}>
            <div className="agent-visual">
              <AgentCanvas
                ref={canvasRef}
                agentProfile={agentProfile}
                agentStyle={agentStyle}
                onPointerDown={startDrag}
                onPointerMove={onDrag}
                onPointerUp={stopDrag}
                onPointerLeave={stopDrag}
                onAgentActivate={handleAgentActivate}
              />
              {selectedAgent && <AgentProfilePanel agent={selectedAgent} />}
            </div>

            <div className="conversation-shell">
              <AgentConversationOverlay
                ref={conversationOverlayRef}
                connectionStatus={connectionStatus}
                connectionStatusLabel={connectionStatusLabel}
                messages={messages}
                isLoading={isLoading}
                onClose={closeConversationPanel}
                onInspectMessage={handleInspectMessage}
                streamEvents={streamEvents}
                streamStatus={streamStatus}
                embedded
                avatar={agentProfile?.avatar}
                title={selectedAgent ? (selectedAgent.display_name || selectedAgent.name || '智能体') : '会话'}
                subtitle={null}
              />
            </div>
          </div>
        </div>

        <AgentSidebarRight
          walletSnapshot={sidebarWallet}
          transactions={transactions}
          interactionLogs={interactionLogs}
          isInteractionCollapsed={isInteractionCollapsed}
          isCollapsed={isSidebarCollapsed}
          onRefreshWallet={refreshWallet}
          onToggleInteraction={toggleInteractionPanel}
          onToggleCollapse={toggleSidebar}
          onInspectWallet={handleInspectWallet}
          onInspectTransaction={handleInspectTransaction}
          connectActionSlot={() => (
            <button type="button" onClick={goToWallet}>
              {t('agent.sidebar.connectNow')}
            </button>
          )}
        />
      </div>

      <AgentConsoleDock
        ref={consoleDockRef}
        value={currentMessage}
        onChange={setCurrentMessage}
        isLoading={isLoading}
        style={consoleDockStyle}
        showOpenButton={!showConversationPanel && messages.length > 0}
        onSend={sendMessage}
        onNewLine={() => setCurrentMessage((prev) => `${prev}\n`)}
        onOpenConversation={openConversationPanel}
      />

      <CreateAgentModal
        isOpen={isCreateAgentModalOpen}
        onClose={closeCreateAgentModal}
        onSubmit={handleCreateAgentSubmit}
        onResolve={resolveExistingAgentTarget}
        onEarlyChannel={handleEarlyChannel}
        sessionId={sessionId}
      />

      <InviteAgentModal
        isOpen={isInviteModalOpen}
        onClose={closeInviteModal}
        targetChannel={inviteTargetChannel}
        onInvite={handleInviteSubmit}
        onResolve={resolveExistingAgentTarget}
      />

      <McpModal
        resource={isUiModalOpen ? uiResource : null}
        onClose={closeUiResource}
        onUIAction={handleUiAction}
      />

      <DiapPanelToggle
        sessionId={sessionId}
        selectedAgent={selectedAgent}
        isSidebarCollapsed={isSidebarCollapsed}
        isDarkMode={isDarkMode}
        showPanel={showDiapPanel}
        onToggle={() => setShowDiapPanel((prev) => !prev)}
        onClosePanel={() => setShowDiapPanel(false)}
      />

      <button type="button" className="language-switch" onClick={toggleLanguage} title={languageLabel}>
        <img src={TranslationIcon} alt="翻译" className="language-icon" />
      </button>
    </div>
  )
}

export default AgentChat
