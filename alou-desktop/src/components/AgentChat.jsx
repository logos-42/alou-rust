import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'

// Components
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import GroupChatPanel from '@/components/agent/GroupChatPanel'
import SplitView from '@/components/common/SplitView'
import DiapPanelToggle from '@/components/agent/DiapPanelToggle'
import McpModal from '@/components/mcp/McpModal'
import CreateAgentModal from '@/components/CreateAgentModal'
import ImportAgentModal from '@/components/ImportAgentModal'
import InviteAgentModal from '@/components/InviteAgentModal'
import AgentProfilePanel from '@/components/AgentProfilePanel'
import AgentDetailPanel from '@/components/agent/AgentDetailPanel'
import RateLimitModal from '@/components/RateLimitModal'
import SkillsManager from '@/components/SkillsManager'
import SDKModal from '@/components/SDKModal'
import WorkflowProgress from '@/components/WorkflowProgress'
import TranslationIcon from '@/assets/icon_翻译.png'

// Hooks
import { useAgentUI } from './AgentChat/useAgentUI.jsx'
import { useAgentConnection } from './AgentChat/useAgentConnection.jsx'
import { useAgentDrag } from './AgentChat/useAgentDrag'
import { useAgentMessages } from './AgentChat/useAgentMessages'
import { useAgentWallet } from './AgentChat/useAgentWallet'
import { useChannelManager } from './AgentChat/useChannelManager'
import { useAgentInvite } from './AgentChat/useAgentInvite'
import { useMultiAgentCoordinator } from './AgentChat/useMultiAgentCoordinator'
import { useAgentStreamHandler } from './AgentChat/useAgentStreamHandler'
import { useAgentEventHandlers } from './AgentChat/useAgentEventHandlers'
import { useGroupChatManager } from './AgentChat/useGroupChatManager'
import { useGroupChatButton } from './AgentChat/useGroupChatButton'
import { useRateLimitModal } from './AgentChat/useRateLimitModal'
import { useAgentBackground } from './AgentChat/useAgentBackground'
import { useAgentModals } from './AgentChat/useAgentModals'
import { useGroupChatRemoteControl } from './AgentChat/useGroupChatRemoteControl'
import { useAgentWorkflow } from './AgentChat/useAgentWorkflow'
import { useAutoAgentCreator } from './AgentChat/useAutoAgentCreator'

// Utils & Constants
import { useToolCallHandler } from '@/hooks/useAgentChat'
import { computeAgentProfile } from './AgentChat/agentUtils'
import avatarManager from './AgentChat/avatarManager'
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
  const [showWorkflowPanel, setShowWorkflowPanel] = useState(false)
  const [showSkillsPanel, setShowSkillsPanel] = useState(false)

  // ==================== Rate Limit Modal Hook ====================
  const { rateLimitModal, openRateLimitModal, closeRateLimitModal, handleSubscribe } = useRateLimitModal()

  // ==================== Background Hook ====================
  const { chatBackground, updateBackground } = useAgentBackground(activeChannelId)

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

  // ==================== Modals Hook ====================
  // 需要在 useAgentUI 之后定义，因为需要 recordInteraction
  // 需要在 useAgentDrag 之前定义，因为 useAgentDrag 需要 openDetailPanel
  const {
    isCreateAgentModalOpen,
    isImportAgentModalOpen,
    showDetailPanel,
    createChannel,
    importChannel,
    closeCreateAgentModal,
    closeImportAgentModal,
    openDetailPanel,
    closeDetailPanel,
  } = useAgentModals({ recordInteraction })

  // 处理编辑智能体
  const handleEditAgent = useCallback(() => {
    if (selectedAgent) {
      openDetailPanel()
      recordInteraction('edit_agent_clicked', { agentId: selectedAgent.id })
    }
  }, [selectedAgent, openDetailPanel, recordInteraction])

  // 处理智能体更新
  const handleAgentUpdated = useCallback((updatedAgent) => {
    console.log('[AgentChat] 智能体已更新:', {
      id: updatedAgent.id,
      avatar: updatedAgent.avatar,
      display_name: updatedAgent.display_name,
      name: updatedAgent.name,
      hasAvatarField: 'avatar' in updatedAgent
    })
    
    // 更新selectedAgent状态
    setSelectedAgent(updatedAgent)
    
    // 使用头像管理模块更新频道列表
    setChannels(prevChannels => 
      avatarManager.updateChannelsAvatar(prevChannels, updatedAgent)
    )
    
    console.log('[AgentChat] 频道列表已更新')
  }, [])

  // ==================== Group Chat Manager Hook ====================
  // 需要在 useAgentUI 之后调用，因为需要 openConversationPanel
  const groupChatManager = useGroupChatManager({
    openConversationPanel,
    activeChannelId,
    localIdentity: {
      did: userName || 'user',
      name: userName || '本地用户'
    }
  })

  const {
    showGroupChat,
    activeActionId,
    activeAction,
    groupChatMessages,
    actionStatus,
    splitPosition,
    groupChatList,
    openGroupChat,
    closeGroupChat,
    closeGroupChatCompletely,
    toggleGroupChat,
    setSplitPosition,
    switchGroupChat,
    hasActiveAction,
    canOpenGroupChat,
  } = groupChatManager


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
    onAvatarClick: useCallback(() => {
      if (selectedAgent) {
        openDetailPanel()
      }
    }, [selectedAgent, openDetailPanel]),
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

  // ==================== 5. Channel Manager Hook ====================
  // 需要在 useAgentMessages 之前调用，因为 useAgentMessages 需要 currentMode
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
    loadMessagesFromIpfs: null, // 会在 useAgentMessages 之后更新
    preferredChain,
  })

  const {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    handleImportAgent,
    saveAgentToStorage,
    deleteChannel,
    currentMode,
    handleModeChange,
    handleCreateAgentSubmit,
    handleEarlyChannel,
  } = channelManager

  // 处理从群聊选择智能体
  const handleSelectAgentFromGroupChat = useCallback((agentId, agentInfo) => {
    console.log('[AgentChat] 从群聊选择智能体:', agentId, agentInfo)
    
    // 查找对应的频道
    const targetChannel = channels.find(ch => 
      ch.id === agentId || 
      ch.meta?.did === agentId || 
      ch.agent_id === agentId
    )
    
    if (targetChannel) {
      // 选择该智能体频道
      selectChannel(targetChannel)
      
      // 如果群聊面板打开，关闭它以显示主对话
      if (showGroupChat) {
        closeGroupChat()
      }
      
      // 确保对话面板打开
      openConversationPanel()
      
      // 记录交互
      recordInteraction('select_agent_from_groupchat', {
        agentId,
        agentInfo,
        channelId: targetChannel.id
      })
    } else {
      console.warn('[AgentChat] 未找到对应的智能体频道:', agentId)
    }
  }, [channels, selectChannel, showGroupChat, closeGroupChat, openConversationPanel, recordInteraction])

  // 处理从群聊点击智能体头像
  const handleAgentClickFromGroupChat = useCallback((agent) => {
    console.log('[AgentChat] 从群聊点击智能体头像:', agent)
    
    // 获取智能体ID
    const agentId = agent.id || agent.agent_id || agent.did
    
    // 查找对应的频道
    const targetChannel = channels.find(ch => 
      ch.id === agentId || 
      ch.meta?.did === agentId || 
      ch.agent_id === agentId
    )
    
    if (targetChannel) {
      // 选择该智能体频道
      selectChannel(targetChannel)
      
      // 不关闭群聊面板，让用户可以同时看到群聊和智能体对话
      // 但确保对话面板打开
      openConversationPanel()
      
      // 记录交互
      recordInteraction('click_agent_avatar_from_groupchat', {
        agentId,
        agent,
        channelId: targetChannel.id
      })
      
      // 切换输入目标到智能体模式
      window.dispatchEvent(new CustomEvent('switch-input-target', {
        detail: { target: 'agent' }
      }))
    } else {
      console.warn('[AgentChat] 未找到对应的智能体频道:', agentId)
    }
  }, [channels, selectChannel, openConversationPanel, recordInteraction])

  // ==================== Tool Call Handler (需要先定义，因为 useAgentMessages 需要它) ====================
  // 注意：这里先创建一个占位函数，实际的 handleToolCalls 会在 messageState 之后更新
  const handleToolCallsRef = useRef(null)
  
  // ==================== 自动创建智能体 Hook ====================
  // 使用专门的 hook 处理自动创建，便于 SDK 复用
  const autoAgentCreator = useAutoAgentCreator({
    onCreateAgent: handleCreateAgentSubmit,
  })

  const {
    autoCreateAgent: handleAutoCreateAgent,
    isAutoCreating,
    autoCreationError,
    resetState: resetAutoCreation,
  } = autoAgentCreator

  // ==================== 6. Message State Hook ====================
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
      // 使用 ref 中的实际处理函数
      if (handleToolCallsRef.current) {
        handleToolCallsRef.current(toolCalls)
      }
    }, []),
    conversationOverlayRef,
    consoleDockRef,
    contextEventsRef,
    currentMode, // 传递当前模式（从 channelManager 获取）
    onRateLimitExceeded: openRateLimitModal,
    onCreateAgent: createChannel, // 传递创建智能体的回调函数（打开模态框）
    onAutoCreateAgent: handleAutoCreateAgent, // 新增：传递自动创建智能体的回调函数
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
    cancelAgentExecution,
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
  
  // 使用 useEffect 更新 ref，确保在每次渲染后更新
  useEffect(() => {
    handleToolCallsRef.current = handleToolCalls
  }, [handleToolCalls])

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

  // ==================== 9. Multi-Agent Coordinator ====================
  const multiAgentCoordinator = useMultiAgentCoordinator({
    selectedAgent,
    sessionId,
    channels,
    appendMessage,
  })

  const {
    routeMessageToAgent,
    sendToAgent,
    analyzeIntent,
    isCoordinatorReady,
  } = multiAgentCoordinator

  // ==================== Workflow Hook ====================
  const workflowState = useAgentWorkflow({
    sessionId,
    selectedAgent,
    appendMessage,
    scrollToBottom,
    recordInteraction,
  })

  const {
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    workflowLoading,
    executingWorkflowId,
    executionProgress,
    activeExecutions,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    handleWorkflowEvent,
    agentInfo,
  } = workflowState

  // 工作流控制函数
  const handlePauseExecution = useCallback(async (executionId) => {
    try {
      await workflowService.pauseExecution(executionId)
    } catch (error) {
      console.error('[AgentChat] 暂停执行失败:', error)
    }
  }, [])

  const handleResumeExecution = useCallback(async (executionId) => {
    try {
      await workflowService.resumeExecution(executionId)
    } catch (error) {
      console.error('[AgentChat] 恢复执行失败:', error)
    }
  }, [])

  const handleCancelExecution = useCallback(async (executionId) => {
    try {
      await workflowService.cancelExecution(executionId)
    } catch (error) {
      console.error('[AgentChat] 取消执行失败:', error)
    }
  }, [])

  // ==================== 10. Stream Handler ====================
  const { streamStatus, streamEvents } = useAgentStreamHandler({
    sessionId,
    isSessionReady,
    recordInteraction,
    openUiResource,
    setConnectionStatus,
  })

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
        const channelMode = channel.meta?.mode || 'agent' // 默认为 agent 模式
        // 模式筛选：agent 或 alou
        if (selectedModelType === 'agent') {
          return channelMode === 'agent' || !channelMode // 默认也是 agent
        }
        if (selectedModelType === 'alou') {
          return channelMode === 'alou'
        }
        return true
      })
    }

    return filtered
  }, [channelKeyword, channels, selectedModelType])

  const showConversationPanel = isConversationVisible

  // ==================== Group Chat Button Hook ====================
  const { ConversationPanelWrapper } = useGroupChatButton({
    showConversationPanel,
    showGroupChat,
    canOpenGroupChat,
    openGroupChat,
    closeGroupChat,
  })

  // ==================== 11. Event Handlers ====================
  const eventHandlers = useAgentEventHandlers({
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
  })

  const {
    goToLogin,
    goToWallet,
    handleLogout,
    toggleLanguage,
    handleWalletChanged,
    handleInspectWallet,
    handleInspectTransaction,
    handleInspectMessage,
  } = eventHandlers

  // ==================== Group Chat Remote Control Hook ====================
  // 群聊遥控功能：管理输入目标模式和消息发送逻辑
  const remoteControl = useGroupChatRemoteControl({
    showGroupChat,
    activeActionId,
    activeChannelId,
    currentMessage,
    setCurrentMessage,
    sendMessageToAgent,
    selectedAgent,
    isAgentLoading,
    isSessionReady,
    createSession,
    setSessionReady,
    userName,
  })

  const { inputTargetMode, sendMessage, handleGroupChatPanelClick } = remoteControl


  // ==================== Bootstrap Effect ====================
  // 处理头像更新事件 - 使用 useCallback 避免无限循环
  const handleAvatarUpdated = useCallback((event) => {
    const { agentId, agent } = event.detail
    if (selectedAgent?.id === agentId) {
      setSelectedAgent(agent)
      setChannels(prevChannels => 
        avatarManager.updateChannelsAvatar(prevChannels, agent)
      )
    }
  }, [selectedAgent, setSelectedAgent, setChannels])

  useEffect(() => {
    const bootstrap = async () => {
      try {
        initLanguage()
        await Promise.all([
          checkConnection(),
          createSession().then(() => setSessionReady(true)).catch(err => {
            console.warn('创建会话失败，但应用将继续在本地模式下运行:', err)
            setSessionReady(true) // 即使会话创建失败，也设置会话为就绪状态
          }),
          refreshWallet().catch((err) => {
            console.warn('Failed to refresh wallet:', err)
          }),
        ])
      } catch (error) {
        console.error('Error in AgentChat initialization:', error)
        // 即使初始化出错，也设置会话为就绪状态，让应用可以继续运行
        setSessionReady(true)
      }
    }
    bootstrap()

    // Event listeners
    if (typeof window !== 'undefined') {
      window.addEventListener('wallet-changed', handleWalletChanged)
      window.addEventListener('resize', handleResize)
      window.addEventListener('pointerup', handleGlobalPointerUp)
      window.addEventListener('agent-avatar-updated', handleAvatarUpdated)
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('wallet-changed', handleWalletChanged)
        window.removeEventListener('resize', handleResize)
        window.removeEventListener('pointerup', handleGlobalPointerUp)
        window.removeEventListener('agent-avatar-updated', handleAvatarUpdated)
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [handleAvatarUpdated, handleWalletChanged, handleResize, handleGlobalPointerUp])

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
        onBackgroundChange={updateBackground}
        isSidebarCollapsed={isSidebarCollapsed}
        activeChannelId={activeChannelId}
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
          onImportChannel={importChannel}
          onDeleteChannel={handleDeleteChannel}
          onInviteToChannel={handleInviteToChannel}
          isCollapsed={isLeftSidebarCollapsed}
          onToggleCollapse={toggleLeftSidebar}
          selectedModelType={selectedModelType}
          onModelTypeChange={setSelectedModelType}
          currentMode={currentMode}
          onModeChange={handleModeChange}
          onShowIdentityPanel={() => setShowDiapPanel(true)}
        />

        <div 
          className="agent-center"
          style={chatBackground ? {
            backgroundImage: `url(${chatBackground})`,
            backgroundSize: 'cover',
            backgroundPosition: 'center',
            backgroundRepeat: 'no-repeat',
          } : {}}
        >
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
                onOpenSkills={() => setShowSkillsPanel(true)}
              />
              {selectedAgent && <AgentProfilePanel agent={selectedAgent} />}

              {/* 工作流进度显示 */}
              {executingWorkflowId && executionProgress[executingWorkflowId] && (
                <WorkflowProgress
                  workflowId={executingWorkflowId}
                  progress={executionProgress[executingWorkflowId]}
                  execution={activeExecutions[executionProgress[executingWorkflowId]?.executionId]}
                  onPause={handlePauseExecution}
                  onResume={handleResumeExecution}
                  onCancel={handleCancelExecution}
                  isDarkMode={isDarkMode}
                />
              )}
            </div>

            {showDetailPanel && selectedAgent && (
              <AgentDetailPanel
                agent={selectedAgent}
                sessionId={sessionId}
                onClose={closeDetailPanel}
                isDarkMode={isDarkMode}
                onAgentUpdated={handleAgentUpdated}
              />
            )}

            <div className="conversation-shell">
              {showGroupChat && activeActionId ? (
                <SplitView
                  left={
                    <ConversationPanelWrapper>
                      <AgentConversationOverlay
                        ref={conversationOverlayRef}
                        connectionStatus={connectionStatus}
                        connectionStatusLabel={connectionStatusLabel}
                        messages={messages}
                        isLoading={isAgentLoading(activeChannelId)}
                        onClose={closeConversationPanel}
                        onInspectMessage={handleInspectMessage}
                        onEdit={handleEditAgent}
                        streamEvents={streamEvents}
                        streamStatus={streamStatus}
                        embedded
                        avatar={agentProfile?.avatar}
                        title={selectedAgent ? (selectedAgent.display_name || selectedAgent.name || '智能体') : '会话'}
                        subtitle={null}
                        backgroundImage={chatBackground}
                      />
                    </ConversationPanelWrapper>
                  }
                  right={
                    <GroupChatPanel
                      actionId={activeActionId}
                      actionDescription={activeAction?.description || activeAction?.action?.description}
                      agents={activeAction?.agents || []}
                      messages={groupChatMessages}
                      status={actionStatus}
                      onClose={closeGroupChatCompletely}
                      onRefresh={() => {
                        // 刷新群聊消息的逻辑已在 useGroupChat 中处理
                      }}
                      onAgentClick={handleAgentClickFromGroupChat}
                      isLoading={actionStatus === 'Running'}
                      groupChatList={groupChatList}
                      activeChannelId={activeChannelId}
                      onSwitchGroupChat={switchGroupChat}
                      onPanelClick={handleGroupChatPanelClick}
                      onSelectAgent={handleSelectAgentFromGroupChat}
                      onResize={setSplitPosition}
                      inputTargetMode={inputTargetMode}
                    />
                  }
                  defaultPosition={splitPosition}
                  minLeftWidth={30}
                  minRightWidth={25}
                  storageKey="agent-chat-split-position"
                  onResize={setSplitPosition}
                />
              ) : (
                <ConversationPanelWrapper>
                  <AgentConversationOverlay
                    ref={conversationOverlayRef}
                    connectionStatus={connectionStatus}
                    connectionStatusLabel={connectionStatusLabel}
                    messages={messages}
                    isLoading={isAgentLoading(activeChannelId)}
                    onClose={closeConversationPanel}
                    onInspectMessage={handleInspectMessage}
                    onEdit={handleEditAgent}
                    streamEvents={streamEvents}
                    streamStatus={streamStatus}
                    embedded
                    avatar={agentProfile?.avatar}
                    title={selectedAgent ? (selectedAgent.display_name || selectedAgent.name || '智能体') : '会话'}
                    subtitle={null}
                    backgroundImage={chatBackground}
                  />
                </ConversationPanelWrapper>
              )}
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
          selectedAgent={selectedAgent}
          connectActionSlot={() => (
            <button type="button" onClick={goToWallet}>
              {t('agent.sidebar.connectNow')}
            </button>
          )}
        />

      {/* 技能管理面板 */}
      {showSkillsPanel && (
        <div className="skills-panel-overlay">
          <SkillsManager
            sessionId={sessionId}
            agentInfo={agentInfo}
            onSkillEvent={handleWorkflowEvent}
            className="skills-panel-content"
            isDarkMode={isDarkMode}
          />
          <button
            type="button"
            className="skills-panel-close"
            onClick={() => setShowSkillsPanel(false)}
            title={t('common.close')}
          >
            ✕
          </button>
        </div>
      )}
      </div>

      {/* 限额弹窗 - 显示在输入框上方 */}
      <RateLimitModal
        isOpen={rateLimitModal.isOpen}
        onClose={closeRateLimitModal}
        remainingRequests={rateLimitModal.remainingRequests}
        resetTime={rateLimitModal.resetTime}
        onSubscribe={handleSubscribe}
      />

      <AgentConsoleDock
        ref={consoleDockRef}
        value={currentMessage}
        onChange={setCurrentMessage}
        isLoading={isAgentLoading(activeChannelId)}
        style={consoleDockStyle}
        showOpenButton={!showConversationPanel && messages.length > 0}
        onSend={sendMessage}
        onCancel={() => {
          if (activeChannelId) {
            cancelAgentExecution(activeChannelId)
          }
        }}
        onNewLine={() => setCurrentMessage((prev) => `${prev}\n`)}
        onOpenConversation={openConversationPanel}
        inputTargetMode={showGroupChat ? inputTargetMode : null}
        showGroupChat={showGroupChat}
      />

      <CreateAgentModal
        isOpen={isCreateAgentModalOpen}
        onClose={closeCreateAgentModal}
        onSubmit={handleCreateAgentSubmit}
        onEarlyChannel={handleEarlyChannel}
        sessionId={sessionId}
      />

      <ImportAgentModal
        isOpen={isImportAgentModalOpen}
        onClose={closeImportAgentModal}
        onResolve={resolveExistingAgentTarget}
        onImportAgent={handleImportAgent}
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
