import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'
import { walletService } from '@/services/walletService'
import { MCP_UI_TARGETS, requestMcpUiResource } from '@/services/mcpUiService'
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import McpModal from '@/components/mcp/McpModal'
import './AgentChat.css'

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ||
  (import.meta.env.DEV
    ? 'http://localhost:8787'
    : 'https://alou-edge.yuanjieliu65.workers.dev')

const NODE_BOUNDARY = 140

const defaultChannels = [
  {
    id: 'dev-relay',
    name: 'TRX Smart Contract Staking',
    status: 'online',
    statusLabel: '在线',
    icon: '⚡',
    color: 'linear-gradient(135deg,#6366f1,#8b5cf6)',
    updatedAt: Date.now() - 2 * 60 * 60 * 1000,
  },
  {
    id: 'eth-announce',
    name: 'ETH Contract Announcement',
    status: 'busy',
    statusLabel: '执行任务',
    icon: '⬡',
    color: 'linear-gradient(135deg,#0ea5e9,#2563eb)',
    updatedAt: Date.now() - 6 * 60 * 60 * 1000,
  },
  {
    id: 'firefly',
    name: 'Firefly Research',
    status: 'offline',
    statusLabel: '离线',
    icon: '🛰️',
    color: 'linear-gradient(135deg,#ec4899,#f97316)',
    updatedAt: Date.now() - 24 * 60 * 60 * 1000,
  },
  {
    id: 'wallet-ops',
    name: 'Wallet Operations',
    status: 'online',
    statusLabel: '在线',
    icon: '💼',
    color: 'linear-gradient(135deg,#14b8a6,#0ea5e9)',
    updatedAt: Date.now() - 30 * 60 * 1000,
  },
]

const defaultTransactions = [
  {
    id: 'tx-1',
    direction: 'out',
    amount: '0.42',
    token: 'ETH',
    counterparty: '0x1F345...ab91',
    status: 'confirmed',
    statusLabel: '已完成',
    timestamp: Date.now() - 4 * 60 * 60 * 1000,
  },
  {
    id: 'tx-2',
    direction: 'in',
    amount: '250',
    token: 'USDC',
    counterparty: '0x72ab...cc87',
    status: 'pending',
    statusLabel: '确认中',
    timestamp: Date.now() - 40 * 60 * 1000,
  },
  {
    id: 'tx-3',
    direction: 'out',
    amount: '1.2',
    token: 'ETH',
    counterparty: '0xbf12...9980',
    status: 'failed',
    statusLabel: '失败',
    timestamp: Date.now() - 3 * 24 * 60 * 60 * 1000,
  },
]

const ACTION_LABELS = {
  channel_selected: '切换频道',
  create_channel: '创建频道',
  toggle_theme: '主题切换',
  toggle_language: '语言切换',
  toggle_sidebar: '侧边栏',
  wallet_refresh: '刷新资产',
  wallet_event: '钱包变更',
  navigate_wallet: '打开钱包',
  navigate_login: '跳转登录',
  logout: '退出登录',
  agent_drag_start: '移动智能体',
  agent_drag_end: '智能体位置',
  trigger_mcp: '调用 MCP',
  user_message: '用户消息',
  wallet_instruction: '钱包指令',
}

const AgentChat = () => {
  const navigate = useNavigate()

  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const logout = useAuthStore((state) => state.logout)
  const userNameGetter = useAuthStore((state) => state.userName)

  const { t, initLanguage, setLanguage, currentLanguage } = useI18n()

  const [isDarkMode, setIsDarkMode] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState('disconnected')
  const connectionStatusLabel = useMemo(() => {
    if (connectionStatus === 'connected') return '已连接'
    if (connectionStatus === 'error') return '服务异常'
    return '未连接'
  }, [connectionStatus])

  const [isSidebarCollapsed, setSidebarCollapsed] = useState(true)
  const [isLeftSidebarCollapsed, setLeftSidebarCollapsed] = useState(false)
  const [isInteractionCollapsed, setInteractionCollapsed] = useState(true)
  const [isConversationVisible, setConversationVisible] = useState(false)

  const [messages, setMessages] = useState([])
  const [currentMessage, setCurrentMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [sessionId, setSessionId] = useState(
    `frontend_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
  )
  const [isSessionReady, setSessionReady] = useState(false)
  const [interactionLogs, setInteractionLogs] = useState([])
  const [viewportWidth, setViewportWidth] = useState(
    typeof window !== 'undefined' ? window.innerWidth : 1440,
  )
  const languageLabel = useMemo(
    () => (currentLanguage === 'zh' ? '中 / EN' : 'EN / 中'),
    [currentLanguage],
  )
  const showConversationPanel = messages.length > 0 && isConversationVisible

  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)
  const dragStateRef = useRef({ dragging: false, offsetX: 0, offsetY: 0, moved: false })
  const agentPositionRef = useRef({ x: 0, y: 0 })
  const [agentPosition, setAgentPosition] = useState({ x: 0, y: 0 })
  const [uiResource, setUiResource] = useState(null)
  const [isUiModalOpen, setUiModalOpen] = useState(false)

  const contextEventsRef = useRef([])

  const [channelKeyword, setChannelKeyword] = useState('')
  const [activeChannelId, setActiveChannelId] = useState('dev-relay')
  const [channels, setChannels] = useState(defaultChannels)

  const agentProfile = useMemo(
    () => ({
      name: 'alou',
      role: 'Web3 Multi-Agent Coordinator',
      avatar: 'https://avatars.githubusercontent.com/u/16309930?v=4',
    }),
    [],
  )

  const [walletSnapshot, setWalletSnapshot] = useState(null)
  const [transactions] = useState(defaultTransactions)

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
    const leftWidth = isLeftSidebarCollapsed
      ? 84
      : viewportWidth <= 1280
        ? 240
        : 300
    const rightWidth = isSidebarCollapsed ? 80 : 340
    return {
      marginLeft: `${leftWidth + 24}px`,
      marginRight: `${rightWidth + 24}px`,
    }
  }, [viewportWidth, isSidebarCollapsed, isLeftSidebarCollapsed])

  const conversationOverlayStyle = useMemo(() => {
    if (viewportWidth <= 1024) {
      return { left: '1rem', right: '1rem', bottom: '6rem' }
    }
    const leftWidth = isLeftSidebarCollapsed
      ? 84
      : viewportWidth <= 1280
        ? 240
        : 300
    const rightWidth = isSidebarCollapsed ? 80 : 340
    return {
      left: `${leftWidth + 24}px`,
      right: `${rightWidth + 24}px`,
      bottom: '6.5rem',
    }
  }, [viewportWidth, isSidebarCollapsed, isLeftSidebarCollapsed])

  const userName = useMemo(() => userNameGetter?.() ?? 'User', [userNameGetter])
  const updateAgentPosition = useCallback((next) => {
    agentPositionRef.current = next
    setAgentPosition(next)
  }, [])

  const clampPosition = useCallback(() => {
    const canvasElement = canvasRef.current?.getElement?.()
    if (!canvasElement) return
    const rect = canvasElement.getBoundingClientRect()
    const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
    const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)
    const { x, y } = agentPositionRef.current
    const clamped = {
      x: Math.min(Math.max(x, -limitX), limitX),
      y: Math.min(Math.max(y, -limitY), limitY),
    }
    updateAgentPosition(clamped)
  }, [updateAgentPosition])

  const recordInteraction = useCallback((action, detail, label) => {
    const timestamp = Date.now()
    const entry = {
      id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
      action,
      label: label || ACTION_LABELS[action] || action,
      timestamp,
      detail,
    }

    setInteractionLogs((prev) => {
      const next = [entry, ...prev]
      return next.slice(0, 20)
    })

    contextEventsRef.current.push({ action, detail, timestamp })
    if (contextEventsRef.current.length > 50) {
      contextEventsRef.current.splice(
        0,
        contextEventsRef.current.length - 50,
      )
    }

    if (typeof window !== 'undefined') {
      window.dispatchEvent(
        new CustomEvent('agent-context-event', { detail: { action, detail, timestamp } }),
      )
    }
  }, [])

  const openUiResource = useCallback(
    (resource, meta = {}) => {
      if (!resource) return
      const embedded = resource.resource ? resource : { resource }
      setUiResource(embedded)
      setUiModalOpen(true)
      recordInteraction('trigger_mcp', {
        kind: 'ui_resource_open',
        uri: embedded.resource?.uri,
        ...meta,
      })
    },
    [recordInteraction],
  )

  const fetchAndOpenUiResource = useCallback(
    async (target, params = {}, meta = {}) => {
      if (!target) {
        return
      }
      try {
        const payload = {
          session_id: sessionId,
          ...params,
        }
        const resource = await requestMcpUiResource(target, payload)
        if (!resource) {
          throw new Error('empty resource response')
        }
        openUiResource(resource, {
          source: 'frontend',
          target,
          ...meta,
        })
      } catch (error) {
        console.error('Failed to load MCP UI resource:', error)
        const errorMessage = error instanceof Error ? error.message : String(error)
        recordInteraction('trigger_mcp', {
          kind: 'ui_error',
          target,
          params,
          error: errorMessage,
        })
        setInteractionLogs((prev) => [
          {
            id: `mcp_error_${Date.now()}`,
            action: 'trigger_mcp',
            label: 'MCP UI 加载失败',
            timestamp: Date.now(),
            detail: { target, error: errorMessage },
          },
          ...prev,
        ])
      } finally {
        // no-op
      }
    },
    [openUiResource, recordInteraction, sessionId],
  )

  const closeUiResource = useCallback(() => {
    setUiModalOpen(false)
    setUiResource(null)
  }, [])

  const handleUiAction = useCallback(
    async (action) => {
      console.debug('MCP UI action', action)
      recordInteraction('trigger_mcp', {
        kind: 'ui_action',
        action,
      })
      if (action?.type === 'close') {
        closeUiResource()
      }
      return { type: 'success' }
    },
    [closeUiResource, recordInteraction],
  )

  const appendMessage = useCallback(
    (message) => {
      setMessages((prev) => [...prev, message])
      if (!isConversationVisible) {
        setConversationVisible(true)
      }
    },
    [isConversationVisible],
  )

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      conversationOverlayRef.current?.scrollToBottom?.()
    })
  }, [])

  const createSession = useCallback(async () => {
    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      const response = await fetch(`${API_BASE_URL}/api/session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ wallet_address: walletAddress || undefined }),
      })
      if (response.ok) {
        const data = await response.json()
        setSessionId(data.session_id)
      }
    } catch (error) {
      console.error('Failed to create session:', error)
    }
  }, [])

  const handleToolCalls = useCallback(
    async (toolCalls = []) => {
      for (const toolCall of toolCalls) {
        if (toolCall.name === 'wallet_manager' && toolCall.result) {
          const result = toolCall.result

          if (result.instruction) {
            try {
              if (result.action === 'switch_network' && result.network) {
                const success = await walletService.switchNetwork(result.network)
                if (success) {
                  recordInteraction('wallet_instruction', {
                    instruction: 'switch_network',
                    chainId: result.network.chainId,
                    name: result.network.name,
                  })
                  appendMessage({
                    id: `system_${Date.now()}`,
                    type: 'assistant',
                    content: `✅ 已成功切换到 ${result.network.name} (${result.network.type})`,
                    timestamp: Date.now(),
                    source: 'system',
                  })
                  scrollToBottom()
                }
              } else {
                await walletService.executeInstruction(result.instruction)
                recordInteraction('wallet_instruction', {
                  instruction: result.instruction?.method || 'unknown',
                })
              }
            } catch (error) {
              console.error('Failed to execute wallet instruction:', error)
              appendMessage({
                id: `error_${Date.now()}`,
                type: 'assistant',
                content: `❌ 钱包操作失败：${
                  error instanceof Error ? error.message : '未知错误'
                }`,
                timestamp: Date.now(),
                source: 'error',
              })
              scrollToBottom()
            }
          }
        }

        if (toolCall.name === 'ui_resource' || toolCall.name === 'mcp_ui') {
          const { resource, resources } = toolCall.result || {}
          if (Array.isArray(resources) && resources.length > 0) {
            openUiResource(resources[0], { source: toolCall.name })
          } else if (resource) {
            openUiResource(resource, { source: toolCall.name })
          }
        }
      }
    },
    [appendMessage, openUiResource, recordInteraction, scrollToBottom],
  )

  const sendMessage = useCallback(async () => {
    const text = currentMessage.trim()
    if (!text || isLoading) {
      return
    }

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

    const contextSnapshot = contextEventsRef.current.splice(
      0,
      contextEventsRef.current.length,
    )

    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId,
          message: text,
          wallet_address: walletAddress || undefined,
          context_events: contextSnapshot,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: '未知错误' }))
        throw new Error(`HTTP ${response.status}: ${errorData.error || response.statusText}`)
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
        content: `❌ 抱歉，发生了错误：${
          error instanceof Error ? error.message : '未知错误'
        }`,
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
    createSession,
    currentMessage,
    handleToolCalls,
    isLoading,
    isSessionReady,
    recordInteraction,
    scrollToBottom,
    sessionId,
  ])

  const refreshWallet = useCallback(async () => {
    try {
      const info = await walletService.getCurrentWalletInfo()
      if (!info) {
        setWalletSnapshot(null)
        recordInteraction('wallet_refresh', { connected: false }, '刷新资产')
        return
      }

      const balance = await walletService.getBalance(info.address)
      const snapshot = {
        address: info.address,
        chainId: info.chainId,
        balance,
        balanceFiat: (parseFloat(balance || '0') * 3400).toFixed(2),
        networkLabel:
          {
            '0x1': 'Ethereum Mainnet',
            '0x14a34': 'Base Sepolia',
            '0x2105': 'Base Mainnet',
          }[info.chainId] || `Chain ${info.chainId}`,
      }
      setWalletSnapshot(snapshot)
      recordInteraction(
        'wallet_refresh',
        {
          connected: true,
          balance,
          token: 'ETH',
        },
        '刷新资产',
      )
    } catch (error) {
      console.error('Failed to refresh wallet info:', error)
    }
  }, [recordInteraction])

  const handleWalletChanged = useCallback(
    async (event) => {
      const detail = event?.detail
      recordInteraction('wallet_event', { address: detail?.address })
      await refreshWallet()
      await createSession()
      setSessionReady(true)
    },
    [createSession, recordInteraction, refreshWallet],
  )

  const handleInspectWallet = useCallback(() => {
    if (!walletSnapshot?.address) {
      return
    }
    void fetchAndOpenUiResource(
      MCP_UI_TARGETS.walletOverview,
      {
        wallet_address: walletSnapshot.address,
      },
      { source: 'wallet_card' },
    )
  }, [fetchAndOpenUiResource, walletSnapshot])

  const handleInspectTransaction = useCallback(
    (transaction) => {
      if (!transaction?.id) return
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.transactionDetail,
        {
          transaction_id: transaction.id,
          direction: transaction.direction,
          token: transaction.token,
        },
        { source: 'transaction_list', transactionId: transaction.id },
      )
    },
    [fetchAndOpenUiResource],
  )

  const checkConnection = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE_URL}/api/health`)
      setConnectionStatus(response.ok ? 'connected' : 'error')
    } catch (error) {
      console.error('Connection check failed:', error)
      setConnectionStatus('disconnected')
    }
  }, [])

  const handleResize = useCallback(() => {
    if (typeof window === 'undefined') return
    setViewportWidth(window.innerWidth)
    clampPosition()
  }, [clampPosition])

  const startDrag = useCallback(
    (event) => {
      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) return

      dragStateRef.current.dragging = true
      dragStateRef.current.moved = false
      const rect = canvasElement.getBoundingClientRect()
      const centerX = rect.left + rect.width / 2
      const centerY = rect.top + rect.height / 2
      dragStateRef.current.offsetX =
        event.clientX - (centerX + agentPositionRef.current.x)
      dragStateRef.current.offsetY =
        event.clientY - (centerY + agentPositionRef.current.y)
      event.target?.setPointerCapture?.(event.pointerId)
      recordInteraction('agent_drag_start', { ...agentPositionRef.current })
    },
    [recordInteraction],
  )

  const onDrag = useCallback((event) => {
    if (!dragStateRef.current.dragging) return
    const canvasElement = canvasRef.current?.getElement?.()
    if (!canvasElement) return

    const rect = canvasElement.getBoundingClientRect()
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2

    const nextX =
      event.clientX - centerX - dragStateRef.current.offsetX
    const nextY =
      event.clientY - centerY - dragStateRef.current.offsetY

    const limitX = Math.max(rect.width / 2 - NODE_BOUNDARY, 0)
    const limitY = Math.max(rect.height / 2 - NODE_BOUNDARY, 0)

    const clamped = {
      x: Math.min(Math.max(nextX, -limitX), limitX),
      y: Math.min(Math.max(nextY, -limitY), limitY),
    }
    if (
      Math.abs(clamped.x - agentPositionRef.current.x) > 1 ||
      Math.abs(clamped.y - agentPositionRef.current.y) > 1
    ) {
      dragStateRef.current.moved = true
    }
    updateAgentPosition(clamped)
  }, [updateAgentPosition])

  const stopDrag = useCallback(
    (event) => {
      if (dragStateRef.current.dragging) {
        dragStateRef.current.dragging = false
        event.target?.releasePointerCapture?.(event.pointerId)
        recordInteraction('agent_drag_end', { ...agentPositionRef.current })
      }
    },
    [recordInteraction],
  )

  const handleGlobalPointerUp = useCallback(() => {
    dragStateRef.current.dragging = false
  }, [])

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const toggleInteractionPanel = useCallback(() => {
    setInteractionCollapsed((prev) => !prev)
  }, [])

  const toggleDarkMode = useCallback(() => {
    setIsDarkMode((prev) => {
      const next = !prev
      recordInteraction('toggle_theme', { theme: next ? 'dark' : 'light' })
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('alou-theme', next ? 'dark' : 'light')
      }
      return next
    })
  }, [recordInteraction])

  const toggleLanguage = useCallback(() => {
    const next = currentLanguage === 'zh' ? 'en' : 'zh'
    setLanguage(next)
    recordInteraction('toggle_language', { language: next })
  }, [currentLanguage, recordInteraction, setLanguage])

  const toggleLeftSidebar = useCallback(() => {
    setLeftSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_left_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const openConversationPanel = useCallback(() => {
    if (!isConversationVisible) {
      setConversationVisible(true)
      scrollToBottom()
      const messageCount = messages.length
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.conversationDetail,
        {
          conversation_id: sessionId,
          message_count: messageCount,
        },
        { source: 'conversation_panel' },
      )
    }
  }, [fetchAndOpenUiResource, isConversationVisible, messages, scrollToBottom, sessionId])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  const handleInspectMessage = useCallback(
    (message) => {
      if (!message?.id) {
        return
      }
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.conversationDetail,
        {
          message_id: message.id,
          role: message.type,
          timestamp: message.timestamp,
        },
        { source: 'conversation', messageId: message.id },
      )
    },
    [fetchAndOpenUiResource],
  )

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

  const selectChannel = useCallback(
    (channel) => {
      setActiveChannelId(channel.id)
      recordInteraction('channel_selected', {
        channelId: channel.id,
        name: channel.name,
        status: channel.status,
        statusLabel: channel.statusLabel,
      })
      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.channelDetail,
        {
          channel_id: channel.id,
        },
        { channelId: channel.id },
      )
    },
    [fetchAndOpenUiResource, recordInteraction],
  )

  const createChannel = useCallback(() => {
    recordInteraction('create_channel')
    void fetchAndOpenUiResource(MCP_UI_TARGETS.channelCreate, {})
  }, [fetchAndOpenUiResource, recordInteraction])

  const handleAgentActivate = useCallback(() => {
    if (dragStateRef.current?.moved) {
      dragStateRef.current.moved = false
      return
    }
    dragStateRef.current.moved = false
    void fetchAndOpenUiResource(
      MCP_UI_TARGETS.agentProfile,
      {
        agent_id: agentProfile?.name,
      },
      { agent: agentProfile?.name },
    )
  }, [agentProfile, fetchAndOpenUiResource])

  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      if (savedTheme) {
        setIsDarkMode(savedTheme === 'dark')
      } else if (typeof window !== 'undefined') {
        setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
      }
    }

    initLanguage()
    checkConnection()
    createSession().then(() => setSessionReady(true))
    refreshWallet()

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
  }, [
    checkConnection,
    createSession,
    handleGlobalPointerUp,
    handleResize,
    handleWalletChanged,
    initLanguage,
    refreshWallet,
  ])

  const shellClassName = [
    'app-shell',
    isDarkMode ? 'dark-mode' : '',
    isLeftSidebarCollapsed ? 'collapsed-left' : '',
  ]
    .filter(Boolean)
    .join(' ')

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
          onKeywordChange={setChannelKeyword}
          onSelectChannel={selectChannel}
          onCreateChannel={createChannel}
          isCollapsed={isLeftSidebarCollapsed}
          onToggleCollapse={toggleLeftSidebar}
        />

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

        <AgentSidebarRight
          walletSnapshot={walletSnapshot}
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
              立即连接
            </button>
          )}
        />

        {showConversationPanel && (
          <AgentConversationOverlay
            ref={conversationOverlayRef}
            style={conversationOverlayStyle}
            connectionStatus={connectionStatus}
            connectionStatusLabel={connectionStatusLabel}
            messages={messages}
            isLoading={isLoading}
            onClose={closeConversationPanel}
            onInspectMessage={handleInspectMessage}
          />
        )}
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

      <McpModal
        resource={isUiModalOpen ? uiResource : null}
        onClose={closeUiResource}
        onUIAction={handleUiAction}
      />

      <button type="button" className="language-switch" onClick={toggleLanguage}>
        {languageLabel}
      </button>
    </div>
  )
}

export default AgentChat

