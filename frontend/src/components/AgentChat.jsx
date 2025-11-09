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
import agentService from '@/services/agentService'
import { MCP_UI_TARGETS, requestMcpUiResource } from '@/services/mcpUiService'
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import McpModal from '@/components/mcp/McpModal'
import {
  ACTION_LABELS,
  API_BASE_URL,
  NODE_BOUNDARY,
  defaultChannels,
  defaultTransactions,
  ensureMillis,
  estimateFiatValue,
  formatWeiHexToEth,
  mapChainIdToBackendChain,
  mapChainLabel,
  normalizeChannel,
  normalizeTransaction,
  resolveBackendChain,
  useToolCallHandler,
} from '@/hooks/useAgentChat'
import './AgentChat.css'

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
  const [preferredChain, setPreferredChain] = useState(null)
  const [userWalletInfo, setUserWalletInfo] = useState(null)
  const [transactions, setTransactions] = useState(defaultTransactions)

  const activeChain = useMemo(
    () =>
      resolveBackendChain({
        chain: preferredChain || walletSnapshot?.chain,
        chainId: userWalletInfo?.chainId,
      }),
    [preferredChain, userWalletInfo?.chainId, walletSnapshot?.chain],
  )

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

  const conversationOverlayStyle = useMemo(() => ({}), [])

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
    (payload, meta = {}) => {
      if (!payload || !payload.resource) return
      const embedded =
        payload.resource && payload.resource.uri
          ? { resource: payload.resource, metadata: payload.metadata }
          : payload
      setUiResource(embedded)
      setUiModalOpen(true)
      recordInteraction('trigger_mcp', {
        kind: 'ui_resource_open',
        uri: embedded.resource?.uri,
        metadata: payload.metadata,
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
        const { resource, metadata } = await requestMcpUiResource(target, payload)
        if (!resource) {
          throw new Error('empty resource response')
        }
        openUiResource({ resource, metadata }, {
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
      const chainId =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
      const detectedChain = resolveBackendChain({
        chainId,
        chain: activeChain,
      })

      if (detectedChain && detectedChain !== preferredChain) {
        setPreferredChain(detectedChain)
      }

      const response = await fetch(`${API_BASE_URL}/api/session`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet_address: walletAddress || undefined,
          chain: detectedChain || undefined,
        }),
      })
      if (response.ok) {
        const data = await response.json()
        setSessionId(data.session_id)
      }
    } catch (error) {
      console.error('Failed to create session:', error)
    }
  }, [activeChain, preferredChain])

  const refreshWallet = useCallback(async () => {
    try {
      const info = await walletService.getCurrentWalletInfo()
      if (!info) {
        setUserWalletInfo(null)
        recordInteraction('wallet_refresh', { connected: false }, '刷新资产')
        return
      }

      const balance = await walletService.getBalance(info.address)
      const backendChain =
        resolveBackendChain({ chainId: info.chainId }) || preferredChain || null
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
  }, [preferredChain, recordInteraction])

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
          }) || activeChain || 'eth'
        if (chain && chain !== preferredChain) {
          setPreferredChain(chain)
        }
        const agentWalletChain =
          chain.startsWith('base')
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
          content:
            result.summary
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
          ) ||
          {
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

        setTransactions((prev) => [normalizedTx, ...prev])

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
      setTransactions,
      userWalletInfo?.chainId,
    ],
  )

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
        const { metadata } = await requestMcpUiResource(
          MCP_UI_TARGETS.walletOverview,
          payload,
        )

        if (!metadata) {
          return null
        }

        const metadataChain =
          resolveBackendChain({ chain: metadata.chain }) ||
          (targetChain ? resolveBackendChain({ chain: targetChain }) : null)
        if (metadataChain && metadataChain !== preferredChain) {
          setPreferredChain(metadataChain)
        }

        const wallets = Array.isArray(metadata.wallets) ? metadata.wallets : []
        const requestedChain = payload.chain
        const primaryWallet =
          metadata.wallet ||
          (requestedChain
            ? wallets.find((item) => item.chain === requestedChain)
            : null) ||
          wallets[0] ||
          null

        if (primaryWallet) {
          const rawBalance = primaryWallet.balance ?? '0'
          const balanceString =
            typeof rawBalance === 'string'
              ? rawBalance
              : rawBalance?.toString?.() ?? '0'
          const balanceNumeric = parseFloat(balanceString)
          const rawChain = primaryWallet.chain || metadata.chain || targetChain || 'ethereum'
          const normalizedChain =
            resolveBackendChain({ chain: rawChain }) || rawChain || 'ethereum'
          const tokenSymbol = normalizedChain === 'sol' ? 'SOL' : 'ETH'

          setWalletSnapshot({
            address: primaryWallet.address || '0x0000',
            balance: balanceString,
            balanceFiat: estimateFiatValue(balanceNumeric, tokenSymbol),
            networkLabel: mapChainLabel(normalizedChain),
            chain: normalizedChain,
            token: tokenSymbol,
          })

          if (normalizedChain && normalizedChain !== preferredChain) {
            setPreferredChain(normalizedChain)
          }

          const txSource = Array.isArray(primaryWallet.transactions)
            ? primaryWallet.transactions
            : Array.isArray(metadata.transactions)
              ? metadata.transactions
              : []

          const normalizedTxs = txSource
            .map((item, index) =>
              normalizeTransaction(item, tokenSymbol) ||
              normalizeTransaction(
                { ...item, id: `${chain}_${index}` },
                tokenSymbol,
              ),
            )
            .filter(Boolean)

          if (normalizedTxs.length > 0) {
            setTransactions(normalizedTxs)
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
    [activeChain, preferredChain, sessionId, setTransactions],
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

      setTransactions((prev) =>
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
    [appendMessage, loadWalletOverview, recordInteraction, scrollToBottom, setTransactions],
  )

  const handleToolCalls = useToolCallHandler({
    handleTransactionBuild,
    handleTransactionBroadcast,
    refreshWallet,
    recordInteraction,
    appendMessage,
    scrollToBottom,
    openUiResource,
  })

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
          chain: activeChain || undefined,
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
    activeChain,
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

  const loadChannelList = useCallback(
    async () => {
      try {
        const { metadata } = await requestMcpUiResource(
          MCP_UI_TARGETS.channelList,
          {
            session_id: sessionId,
          },
        )
        const channelArray = Array.isArray(metadata?.channels) ? metadata.channels : []
        if (!channelArray.length) {
          return []
        }

        const normalized = channelArray
          .map((item, index) => normalizeChannel(item, index))
          .filter(Boolean)

        if (normalized.length > 0) {
          setChannels(normalized)
          if (!normalized.some((channel) => channel.id === activeChannelId)) {
            setActiveChannelId(normalized[0].id)
          }
        }

        return normalized
      } catch (error) {
        console.error('Failed to load channel list:', error)
        return []
      }
    },
    [activeChannelId, sessionId],
  )

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
    if (!sessionId) {
      return
    }
    void fetchAndOpenUiResource(
      MCP_UI_TARGETS.walletOverview,
      {
        session_id: sessionId,
        chain: walletSnapshot?.chain,
      },
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
        {
          session_id: sessionId,
          transaction_id: transactionId,
          chain: walletSnapshot?.chain,
        },
        { source: 'transaction_list', transactionId },
      )
    },
    [fetchAndOpenUiResource, sessionId, walletSnapshot],
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
      void loadChannelList()
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
    [fetchAndOpenUiResource, loadChannelList, recordInteraction],
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
        session_id: sessionId,
        chain: activeChain,
      },
      { agent: agentProfile?.name },
    )
  }, [activeChain, agentProfile, fetchAndOpenUiResource, sessionId])

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
          walletSnapshot={walletSnapshot || userWalletInfo}
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

