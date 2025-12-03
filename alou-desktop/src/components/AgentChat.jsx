import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import agentService from '@/services/agentService'
import { MCP_UI_TARGETS } from '@/services/mcpUiService'
import { useAgentStream } from '@/hooks/useAgentStream'
import { resolveBackendChain, useToolCallHandler } from '@/hooks/useAgentChat'

// UI Components
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import DiapPanelToggle from '@/components/agent/DiapPanelToggle'
import McpModal from '@/components/mcp/McpModal'
import CreateAgentModal from '@/components/CreateAgentModal'
import AgentProfilePanel from '@/components/AgentProfilePanel'
import TranslationIcon from '@/assets/icon_翻译.png'

// Custom Hooks
import { useAgentConnection } from './AgentChat/useAgentConnection'
import { useAgentWallet } from './AgentChat/useAgentWallet'
import { useAgentMessages } from './AgentChat/useAgentMessages'
import { useAgentDrag } from './AgentChat/useAgentDrag'
import { useAgentUI } from './AgentChat/useAgentUI'
import { useChannelManager } from './AgentChat/useChannelManager'
import { buildChannelFromAgent, computeAgentProfile, extractErrorMessage } from './AgentChat/agentUtils'

import './AgentChat/index.css'

const AgentChat = () => {
  const navigate = useNavigate()

  // Auth Store
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const logout = useAuthStore((state) => state.logout)
  const userNameGetter = useAuthStore((state) => state.userName)
  const userName = useMemo(() => userNameGetter?.() ?? 'User', [userNameGetter])

  // Agent Store for persistence
  const addAgentToStore = useAgentStore((state) => state.addAgent)
  const storedAgents = useAgentStore((state) => state.agents)

  // i18n
  const { t, initLanguage, setLanguage, currentLanguage } = useI18n()
  const languageLabel = useMemo(
    () => (currentLanguage === 'zh' ? '中 / EN' : 'EN / 中'),
    [currentLanguage],
  )

  // Refs
  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)

  // Chain State (shared between hooks)
  const [preferredChain, setPreferredChain] = useState(null)
  const [transactions, setTransactions] = useState([])

  // Channel State (shared)
  const [channelKeyword, setChannelKeyword] = useState('')
  const [channels, setChannels] = useState([])
  const [activeChannelId, setActiveChannelId] = useState(null)
  const [selectedAgent, setSelectedAgent] = useState(null)
  const [isChannelLoading, setChannelLoading] = useState(false)
  const [channelError, setChannelError] = useState(null)
  const [selectedModelType, setSelectedModelType] = useState(null)
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)

  // ============================================
  // Custom Hooks
  // ============================================

  // 1. UI State Hook
  const uiState = useAgentUI({
    sessionId: null,
    viewportWidth: typeof window !== 'undefined' ? window.innerWidth : 1440,
  })

  // 2. Connection State Hook
  const connectionState = useAgentConnection({
    activeChain: preferredChain,
    preferredChain,
    setPreferredChain,
  })

  // Compute active chain
  const activeChain = useMemo(
    () => resolveBackendChain({ chain: preferredChain }) || preferredChain,
    [preferredChain],
  )

  // 3. Drag State Hook
  const dragState = useAgentDrag({
    canvasRef,
    recordInteraction: uiState.recordInteraction,
    openConversationPanel: uiState.openConversationPanel,
  })

  // 4. Message State Hook
  const messageState = useAgentMessages({
    sessionId: connectionState.sessionId,
    isSessionReady: connectionState.isSessionReady,
    createSession: connectionState.createSession,
    setSessionReady: connectionState.setSessionReady,
    activeChain,
    recordInteraction: uiState.recordInteraction,
    handleToolCalls: useCallback((toolCalls) => {
      // Will be set up after walletState is defined
    }, []),
    conversationOverlayRef,
    consoleDockRef,
    isConversationVisible: uiState.isConversationVisible,
    setConversationVisible: uiState.setConversationVisible,
  })

  // 5. Wallet State Hook
  const walletState = useAgentWallet({
    sessionId: connectionState.sessionId,
    activeChain,
    preferredChain,
    setPreferredChain,
    recordInteraction: uiState.recordInteraction,
    appendMessage: messageState.appendMessage,
    scrollToBottom: messageState.scrollToBottom,
    setTransactions,
  })

  // Tool call handler
  const handleToolCalls = useToolCallHandler({
    handleTransactionBuild: walletState.handleTransactionBuild,
    handleTransactionBroadcast: walletState.handleTransactionBroadcast,
    refreshWallet: walletState.refreshWallet,
    recordInteraction: uiState.recordInteraction,
    appendMessage: messageState.appendMessage,
    scrollToBottom: messageState.scrollToBottom,
    openUiResource: uiState.openUiResource,
  })

  // 6. Channel Manager Hook
  const channelManager = useChannelManager({
    sessionId: connectionState.sessionId,
    isSessionReady: connectionState.isSessionReady,
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
    recordInteraction: uiState.recordInteraction,
    openConversationPanel: uiState.openConversationPanel,
  })

  // ============================================
  // Computed Values
  // ============================================

  const agentProfile = useMemo(() => computeAgentProfile(selectedAgent), [selectedAgent])

  const agentStyle = useMemo(
    () => ({
      transform: `translate(calc(-50% + ${dragState.agentPosition.x}px), calc(-50% + ${dragState.agentPosition.y}px))`,
    }),
    [dragState.agentPosition],
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

  const showConversationPanel = uiState.isConversationVisible

  // ============================================
  // Stream Events
  // ============================================

  const handleStreamEvent = useCallback(
    (event) => {
      if (!event) return

      uiState.recordInteraction('stream_event', {
          event: event.event,
          payload: event.payload,
      }, event.label)

      if (event.isFinal && event.timestamp && uiState.finalEventHandledRef.current !== event.timestamp) {
        uiState.finalEventHandledRef.current = event.timestamp
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
              uiState.openUiResource(result.resources[0], { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.resource) {
              uiState.openUiResource(result.resource, { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.uri) {
              uiState.openUiResource(result, { source: 'stream_final', toolCall: uiCall.name })
            }
          }
        }
      }
    },
    [uiState],
  )

  const { status: streamStatus, events: streamEvents } = useAgentStream(connectionState.sessionId, {
    enabled: connectionState.isSessionReady,
    onEvent: handleStreamEvent,
  })

  // Update connection status based on stream status
  useEffect(() => {
    if (streamStatus === 'error') {
      connectionState.setConnectionStatus('error')
    } else if (streamStatus === 'active' || streamStatus === 'completed') {
      connectionState.setConnectionStatus('connected')
    } else if (streamStatus === 'polling') {
      connectionState.setConnectionStatus('connecting')
    } else {
      connectionState.setConnectionStatus('disconnected')
    }
  }, [streamStatus, connectionState])

  // ============================================
  // Event Handlers
  // ============================================

  const goToLogin = useCallback(() => {
    uiState.recordInteraction('navigate_login')
    navigate('/login')
  }, [navigate, uiState])

  const goToWallet = useCallback(() => {
    uiState.recordInteraction('navigate_wallet')
    navigate('/wallet')
  }, [navigate, uiState])

  const handleLogout = useCallback(async () => {
    uiState.recordInteraction('logout')
    await logout()
  }, [logout, uiState])

  const toggleLanguage = useCallback(() => {
    const next = currentLanguage === 'zh' ? 'en' : 'zh'
    setLanguage(next)
    uiState.recordInteraction('toggle_language', { language: next })
  }, [currentLanguage, setLanguage, uiState])

  const handleInspectWallet = useCallback(() => {
    if (!connectionState.sessionId) return
    void uiState.fetchAndOpenUiResource(
      MCP_UI_TARGETS.walletOverview,
      { session_id: connectionState.sessionId, chain: walletState.walletSnapshot?.chain },
      { source: 'wallet_card' },
    )
  }, [connectionState.sessionId, uiState, walletState.walletSnapshot?.chain])

  const handleInspectTransaction = useCallback(
    (transaction) => {
      if (!transaction) return
      const transactionId = transaction.hash || transaction.id
      if (!transactionId) return
      void uiState.fetchAndOpenUiResource(
        MCP_UI_TARGETS.transactionDetail,
        { session_id: connectionState.sessionId, transaction_id: transactionId, chain: walletState.walletSnapshot?.chain },
        { source: 'transaction_list', transactionId },
      )
    },
    [connectionState.sessionId, uiState, walletState.walletSnapshot?.chain],
  )

  const handleInspectMessage = useCallback(
    (message) => {
      if (!message?.id) return
      void uiState.fetchAndOpenUiResource(
        MCP_UI_TARGETS.conversationDetail,
        { message_id: message.id, role: message.type, timestamp: message.timestamp },
        { source: 'conversation', messageId: message.id },
      )
    },
    [uiState],
  )

  const createChannel = useCallback(() => {
    setCreateAgentModalOpen(true)
    uiState.recordInteraction('open_create_agent_modal')
  }, [uiState])

  const closeCreateAgentModal = useCallback(() => {
    setCreateAgentModalOpen(false)
  }, [])

  // Early channel display for agent creation
  const handleEarlyChannel = useCallback(
    (earlyMetadata) => {
      const channel = buildChannelFromAgent(earlyMetadata)
      if (channel) {
        const matchKey = earlyMetadata.avatar_cid || earlyMetadata.cid
            setChannels((prev) => {
          const others = prev.filter((item) => {
            const itemAvatarCid = item.meta?.avatar_cid
            const itemCid = item.meta?.cid
            return itemAvatarCid !== matchKey && itemCid !== matchKey && item.id !== channel.id
          })
          return [channel, ...others]
        })
        setActiveChannelId(channel.id)
        setSelectedAgent(earlyMetadata)
        console.log('[AgentChat] 早期频道已显示，等待完整创建...')
      }
    },
    [],
  )

  // Create agent submit handler
  const handleCreateAgentSubmit = useCallback(
    async ({ name, roleDescription, avatarCid, mcpConfigCid, mcpPorts, diapIdentity }) => {
      setChannelLoading(true)
      setChannelError(null)
      uiState.recordInteraction('create_claude_agent', { name })

      try {
        const walletAddress = typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
        const chainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
        const detectedChain = resolveBackendChain({ chainId, chain: activeChain })

        const result = await agentService.createClaudeAgent({
          sessionId: connectionState.sessionId,
          walletAddress,
          chain: detectedChain || activeChain,
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
        }

        if (diapIdentity) {
          metadata.diap_identity = diapIdentity
          metadata.did = metadata.did || diapIdentity.did
          metadata.cid = metadata.cid || diapIdentity.cid
          metadata.ipns = metadata.ipns || diapIdentity.ipns
        }

        const channel = buildChannelFromAgent(metadata)
        if (channel) {
          try {
            addAgentToStore({ ...metadata, sessionId: connectionState.sessionId, id: channel.id })
            console.log('[AgentChat] 智能体已保存到本地存储')
          } catch (error) {
            console.error('[AgentChat] 保存智能体到本地存储失败:', error)
          }

          const matchKey = avatarCid || metadata.cid
          setChannels((prev) => {
            const others = prev.filter((item) => {
              const itemAvatarCid = item.meta?.avatar_cid
              const itemCid = item.meta?.cid
              if (item.id === channel.id) return false
              if (matchKey && (itemAvatarCid === matchKey || itemCid === matchKey || itemCid === `temp_${matchKey}`)) return false
              return true
            })
            return [channel, ...others]
          })
          setActiveChannelId(channel.id)
        }

        setSelectedAgent(metadata)
        setCreateAgentModalOpen(false)
        return result
      } catch (error) {
        const message = extractErrorMessage(error)
        setChannelError(message)
        uiState.recordInteraction('create_claude_agent_failed', { error: message })
        throw new Error(message)
      } finally {
        setChannelLoading(false)
      }
    },
    [activeChain, addAgentToStore, connectionState.sessionId, uiState],
  )

  // ============================================
  // Initialization Effect
  // ============================================

  useEffect(() => {
    // Initialize dark mode
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      if (savedTheme) {
        uiState.setIsDarkMode(savedTheme === 'dark')
      } else if (typeof window !== 'undefined') {
        uiState.setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
      }
    }

    // Bootstrap
    const bootstrap = async () => {
      try {
        initLanguage()
        await Promise.all([
          connectionState.checkConnection(),
          connectionState.createSession().then(() => connectionState.setSessionReady(true)),
          walletState.refreshWallet().catch((err) => console.warn('Failed to refresh wallet:', err)),
        ])
      } catch (error) {
        console.error('Error in AgentChat initialization:', error)
      }
    }
    bootstrap()

    // Event listeners
    if (typeof window !== 'undefined') {
      const handleWalletChanged = async (event) => {
        const detail = event?.detail
        uiState.recordInteraction('wallet_event', { address: detail?.address })
        await walletState.refreshWallet()
        await connectionState.createSession()
        connectionState.setSessionReady(true)
      }

      window.addEventListener('wallet-changed', handleWalletChanged)
      window.addEventListener('resize', uiState.handleResize)
      window.addEventListener('pointerup', dragState.handleGlobalPointerUp)

    return () => {
        window.removeEventListener('wallet-changed', handleWalletChanged)
        window.removeEventListener('resize', uiState.handleResize)
        window.removeEventListener('pointerup', dragState.handleGlobalPointerUp)
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ============================================
  // Render
  // ============================================

  const shellClassName = [
    'app-shell',
    uiState.isDarkMode ? 'dark-mode' : '',
    uiState.isLeftSidebarCollapsed ? 'collapsed-left' : '',
  ].filter(Boolean).join(' ')

  return (
    <div className={shellClassName}>
      <ChatHeader
        connectionStatus={connectionState.connectionStatus}
        isDarkMode={uiState.isDarkMode}
        isAuthenticated={isAuthenticated}
        userName={userName}
        onToggleTheme={uiState.toggleDarkMode}
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
          onKeywordChange={channelManager.handleChannelKeywordChange}
          onRefresh={channelManager.refreshChannels}
          onSelectChannel={channelManager.selectChannel}
          onCreateChannel={createChannel}
          isCollapsed={uiState.isLeftSidebarCollapsed}
          onToggleCollapse={uiState.toggleLeftSidebar}
          selectedModelType={selectedModelType}
          onModelTypeChange={setSelectedModelType}
          onShowIdentityPanel={() => uiState.setShowDiapPanel(true)}
        />

        <div className="agent-center">
          <div className={`conversation-stack ${showConversationPanel ? 'open' : ''}`}>
            <div className="agent-visual">
              <AgentCanvas
                ref={canvasRef}
                agentProfile={agentProfile}
                agentStyle={agentStyle}
                onPointerDown={dragState.startDrag}
                onPointerMove={dragState.onDrag}
                onPointerUp={dragState.stopDrag}
                onPointerLeave={dragState.stopDrag}
                onAgentActivate={dragState.handleAgentActivate}
              />
              {selectedAgent && <AgentProfilePanel agent={selectedAgent} />}
            </div>

            <div className="conversation-shell">
              <AgentConversationOverlay
                ref={conversationOverlayRef}
                connectionStatus={connectionState.connectionStatus}
                connectionStatusLabel={connectionState.connectionStatusLabel}
                messages={messageState.messages}
                isLoading={messageState.isLoading}
                onClose={uiState.closeConversationPanel}
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
          walletSnapshot={walletState.sidebarWallet}
          transactions={transactions}
          interactionLogs={uiState.interactionLogs}
          isInteractionCollapsed={uiState.isInteractionCollapsed}
          isCollapsed={uiState.isSidebarCollapsed}
          onRefreshWallet={walletState.refreshWallet}
          onToggleInteraction={uiState.toggleInteractionPanel}
          onToggleCollapse={uiState.toggleSidebar}
          onInspectWallet={handleInspectWallet}
          onInspectTransaction={handleInspectTransaction}
          connectActionSlot={() => (
            <button type="button" onClick={goToWallet}>
              立即连接
            </button>
          )}
        />
      </div>

      <AgentConsoleDock
        ref={consoleDockRef}
        value={messageState.currentMessage}
        onChange={messageState.setCurrentMessage}
        isLoading={messageState.isLoading}
        style={uiState.consoleDockStyle}
        showOpenButton={!showConversationPanel && messageState.messages.length > 0}
        onSend={messageState.sendMessage}
        onNewLine={() => messageState.setCurrentMessage((prev) => `${prev}\n`)}
        onOpenConversation={uiState.openConversationPanel}
      />

      <CreateAgentModal
        isOpen={isCreateAgentModalOpen}
        onClose={closeCreateAgentModal}
        onSubmit={handleCreateAgentSubmit}
        onResolve={channelManager.resolveExistingAgentTarget}
        onEarlyChannel={handleEarlyChannel}
        sessionId={connectionState.sessionId}
      />

      <McpModal
        resource={uiState.isUiModalOpen ? uiState.uiResource : null}
        onClose={uiState.closeUiResource}
        onUIAction={uiState.handleUiAction}
      />

      <DiapPanelToggle
        sessionId={connectionState.sessionId}
        isSidebarCollapsed={uiState.isSidebarCollapsed}
        isDarkMode={uiState.isDarkMode}
        showPanel={uiState.showDiapPanel}
        onToggle={() => uiState.setShowDiapPanel((prev) => !prev)}
        onClosePanel={() => uiState.setShowDiapPanel(false)}
      />

      <button type="button" className="language-switch" onClick={toggleLanguage} title={languageLabel}>
        <img src={TranslationIcon} alt="翻译" className="language-icon" />
      </button>
    </div>
  )
}

export default AgentChat
