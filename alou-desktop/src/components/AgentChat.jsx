import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'
import { walletService } from '@/services/walletService'
import agentService from '@/services/agentService'
import apiClient from '@/services/api'
import { MCP_UI_TARGETS, requestMcpUiResource } from '@/services/mcpUiService'
import { useAgentStream } from '@/hooks/useAgentStream'
import ChatHeader from '@/components/ChatHeader'
import AgentSidebarLeft from '@/components/agent/AgentSidebarLeft'
import AgentSidebarRight from '@/components/agent/AgentSidebarRight'
import AgentCanvas from '@/components/agent/AgentCanvas'
import AgentConsoleDock from '@/components/agent/AgentConsoleDock'
import AgentConversationOverlay from '@/components/agent/AgentConversationOverlay'
import DiapIdentityPanel from '@/components/agent/DiapIdentityPanel'
import McpModal from '@/components/mcp/McpModal'
import CreateAgentModal from '@/components/CreateAgentModal'
import AgentProfilePanel from '@/components/AgentProfilePanel'
import {
  ACTION_LABELS,
  API_BASE_URL,
  NODE_BOUNDARY,
  defaultTransactions,
  ensureMillis,
  estimateFiatValue,
  formatWeiHexToEth,
  mapChainIdToBackendChain,
  mapChainLabel,
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
    if (connectionStatus === 'connecting') return '连接中'
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

  const [channelKeyword, setChannelKeyword] = useState('')
  const [activeChannelId, setActiveChannelId] = useState(null)
  const [channels, setChannels] = useState([])
  const [selectedAgent, setSelectedAgent] = useState(null)
  const [isChannelLoading, setChannelLoading] = useState(false)
  const [channelError, setChannelError] = useState(null)
  const channelRequestIdRef = useRef(0)
  const searchDebounceRef = useRef(null)
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)

  const buildChannelFromAgent = useCallback((agent) => {
    if (!agent) {
      return null
    }
    const id = agent.did || agent.cid || agent.ipns || `agent_${Date.now()}`
    const nameFromIpns = agent.ipns ? agent.ipns.replace(/^\/?ipns\//, '') : null
    const nameFromDid = agent.did ? agent.did.split(':').filter(Boolean).slice(-1)[0] : null
    const fallbackName = agent.cid || id
    const displayName = agent.display_name || agent.name || nameFromIpns || nameFromDid || fallbackName
    const statusLabel = agent.ipns
      ? 'IPNS 解析'
      : agent.did
        ? 'DID 解析'
        : agent.agent_type === 'claude_agent_sdk'
          ? '自定义智能体'
          : 'CID 解析'

    return {
      id,
      name: displayName,
      status: 'online',
      statusLabel,
      icon: '🛰️',
      color: 'linear-gradient(135deg,#6366f1,#8b5cf6)',
      updatedAt: Math.floor(Date.now() / 1000),
      meta: agent,
    }
  }, [])

  const extractAgentTarget = useCallback((agent) => {
    if (!agent) return null
    if (agent.ipns) return agent.ipns
    if (agent.did) return agent.did
    if (agent.cid) return agent.cid
    if (agent.meta) {
      return agent.meta.ipns || agent.meta.did || agent.meta.cid || null
    }
    return null
  }, [])

  const extractErrorMessage = useCallback((error) => {
    if (!error) return '未知错误'
    if (error.response?.data?.error) {
      return error.response.data.error
    }
    if (typeof error.message === 'string' && error.message.trim().length > 0) {
      return error.message
    }
    return '请求失败'
  }, [])

  const loadChannelList = useCallback(
    async (keyword = channelKeyword) => {
      if (!sessionId) {
        return []
      }

      const query = keyword?.trim() || ''
      const requestId = channelRequestIdRef.current + 1
      channelRequestIdRef.current = requestId

      const applyLatest = (updater) => {
        if (channelRequestIdRef.current === requestId) {
          updater()
        }
      }

      setChannelLoading(true)
      setChannelError(null)

      try {
        if (!query) {
          const session = await agentService.getSession(sessionId)
          const metadata = session?.agent_metadata
          const channel = buildChannelFromAgent(metadata)

          applyLatest(() => {
            if (channel) {
              setChannels([channel])
              setActiveChannelId(channel.id)
              setSelectedAgent(metadata)
            } else {
              setChannels([])
              setActiveChannelId(null)
              setSelectedAgent(null)
            }
          })

          return channel ? [channel] : []
        }

        const response = await agentService.searchAgents(query)
        const agents = Array.isArray(response?.agents) ? response.agents : []
        const mapped = agents.map((agent) => buildChannelFromAgent(agent)).filter(Boolean)

        applyLatest(() => {
          setChannels(mapped)
          if (mapped.length > 0) {
            if (!mapped.some((channel) => channel.id === activeChannelId)) {
              setActiveChannelId(mapped[0].id)
              setSelectedAgent(mapped[0].meta)
            }
          } else {
            setActiveChannelId(null)
            setSelectedAgent(null)
          }
        })

        return mapped
      } catch (error) {
        const message = extractErrorMessage(error)
        applyLatest(() => {
          // Only log connection errors occasionally to avoid spam
          const isConnectionError = 
            error.code === 'ECONNREFUSED' || 
            error.code === 'ERR_NETWORK' ||
            error.message?.includes('ERR_CONNECTION_REFUSED') ||
            error.message?.includes('Failed to fetch') ||
            !error.response
          
          const now = Date.now()
          const lastErrorTime = window.__lastLoadChannelsError || 0
          
          if (isConnectionError) {
            // For connection errors, only log every 10 seconds
            if (now - lastErrorTime > 10000) {
              window.__lastLoadChannelsError = now
              console.warn('[AgentChat] Cannot load agent channels: backend server unavailable.')
            }
          } else {
            // For other errors, log normally
            if (now - lastErrorTime > 5000) {
              window.__lastLoadChannelsError = now
              console.error('Failed to load agent channels:', error)
            }
          }
          
          setChannelError(message)
          setChannels([])
          setActiveChannelId(null)
          setSelectedAgent(null)
        })
        return []
      } finally {
        applyLatest(() => {
          setChannelLoading(false)
        })
      }
    },
    [
      activeChannelId,
      buildChannelFromAgent,
      channelKeyword,
      extractErrorMessage,
      sessionId,
    ],
  )

  useEffect(() => {
    return () => {
      if (searchDebounceRef.current) {
        clearTimeout(searchDebounceRef.current)
      }
    }
  }, [])

  useEffect(() => {
    if (!isSessionReady) {
      return
    }
    void loadChannelList(channelKeyword)
  }, [channelKeyword, isSessionReady, loadChannelList])

  const refreshChannels = useCallback(() => {
    void loadChannelList(channelKeyword)
  }, [channelKeyword, loadChannelList])
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
  const finalEventHandledRef = useRef(null)

  const contextEventsRef = useRef([])

  const handleChannelKeywordChange = useCallback(
    (value) => {
      setChannelKeyword(value)
      if (searchDebounceRef.current) {
        clearTimeout(searchDebounceRef.current)
      }
      searchDebounceRef.current = setTimeout(() => {
        void loadChannelList(value)
      }, 400)
    },
    [loadChannelList],
  )

  const agentProfile = useMemo(() => {
    if (!selectedAgent) {
      return {
        name: 'alou',
        role: 'Web3 Multi-Agent Coordinator',
        avatar: 'https://avatars.githubusercontent.com/u/16309930?v=4',
      }
    }

    const displayName =
      (selectedAgent.ipns && selectedAgent.ipns.replace(/^\/?ipns\//, '').slice(0, 42)) ||
      (selectedAgent.did && selectedAgent.did.split(':').filter(Boolean).slice(-1)[0]) ||
      selectedAgent.cid ||
      '解析智能体'

    const role = selectedAgent.did ? 'DID 智能体' : '去中心化智能体'

    return {
      name: displayName,
      role,
      avatar: 'https://avatars.githubusercontent.com/u/16309930?v=4',
    }
  }, [selectedAgent])

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
      contextEventsRef.current.splice(0, contextEventsRef.current.length - 50)
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
        openUiResource(
          { resource, metadata },
          {
            source: 'frontend',
            target,
            ...meta,
          },
        )
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

  const handleStreamEvent = useCallback(
    (event) => {
      if (!event) {
        return
      }

      recordInteraction(
        'stream_event',
        {
          event: event.event,
          payload: event.payload,
        },
        event.label,
      )

      if (event.isFinal && event.timestamp && finalEventHandledRef.current !== event.timestamp) {
        finalEventHandledRef.current = event.timestamp
        if (Array.isArray(event.payload?.tool_calls)) {
          const uiCall = event.payload.tool_calls.find((call) => {
            const result = call?.result
            if (!result) return false
            if (Array.isArray(result.resources) && result.resources.length > 0) {
              return true
            }
            return Boolean(result.resource || result.uri)
          })

          if (uiCall?.result) {
            const result = uiCall.result
            if (Array.isArray(result.resources) && result.resources.length > 0) {
              openUiResource(result.resources[0], {
                source: 'stream_final',
                toolCall: uiCall.name,
              })
            } else if (result.resource) {
              openUiResource(result.resource, {
                source: 'stream_final',
                toolCall: uiCall.name,
              })
            } else if (result.uri) {
              openUiResource(result, {
                source: 'stream_final',
                toolCall: uiCall.name,
              })
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
  }, [streamStatus])

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
      const chainId = typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
      const detectedChain = resolveBackendChain({
        chainId,
        chain: activeChain,
      })

      if (detectedChain && detectedChain !== preferredChain) {
        setPreferredChain(detectedChain)
      }

      // Use agentService for consistency
      const data = await agentService.createSession(walletAddress || undefined)
      setSessionId(data.session_id)
    } catch (error) {
      // Only log connection errors occasionally to avoid spam
      const isConnectionError = 
        error.code === 'ECONNREFUSED' || 
        error.code === 'ERR_NETWORK' ||
        error.message?.includes('ERR_CONNECTION_REFUSED') ||
        error.message?.includes('Failed to fetch') ||
        !error.response
      
      const now = Date.now()
      const lastErrorTime = window.__lastCreateSessionError || 0
      
      if (isConnectionError) {
        // For connection errors, only log every 10 seconds
        if (now - lastErrorTime > 10000) {
          window.__lastCreateSessionError = now
          console.warn('[AgentChat] Cannot create session: backend server unavailable. Please start the backend server or configure VITE_API_BASE_URL.')
        }
      } else {
        // For other errors, log normally
        if (now - lastErrorTime > 5000) {
          window.__lastCreateSessionError = now
          console.error('Failed to create session:', error)
        }
      }
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

      let balance = '0'
      try {
        balance = await walletService.getBalance(info.address)
      } catch (balanceError) {
        console.warn('Failed to get wallet balance:', balanceError)
        // 如果获取余额失败，使用默认值 0，不中断流程
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

        const normalizedTx = normalizeTransaction(
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
                normalizeTransaction({ ...item, id: `${chain}_${index}` }, tokenSymbol),
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

    const contextSnapshot = contextEventsRef.current.splice(0, contextEventsRef.current.length)

    try {
      const walletAddress =
        typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
      // Use agentService for consistency and proper error handling
      // apiClient (axios) automatically handles errors and returns response.data
      const data = await apiClient.post('/agent/chat', {
        session_id: sessionId,
        message: text,
        wallet_address: walletAddress || undefined,
        chain: activeChain || undefined,
        context_events: contextSnapshot,
      }).then(response => response.data)

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
      walletService.removeListener('chainChanged', handleChainChanged).catch(console.error)
    }
  }, [loadWalletOverview, preferredChain, recordInteraction, refreshWallet])

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
      await agentService.healthCheck()
      setConnectionStatus('connected')
    } catch (error) {
      // Only log connection errors occasionally to avoid spam
      const now = Date.now()
      const lastErrorTime = window.__lastHealthCheckError || 0
      
      if (now - lastErrorTime > 10000) { // Log every 10 seconds max
        window.__lastHealthCheckError = now
        const isConnectionError = 
          error.code === 'ECONNREFUSED' || 
          error.code === 'ERR_NETWORK' ||
          error.message?.includes('ERR_CONNECTION_REFUSED') ||
          error.message?.includes('Failed to fetch') ||
          !error.response
        
        if (isConnectionError) {
          console.warn('[AgentChat] Backend server unavailable. Please start the backend server or configure VITE_API_BASE_URL.')
        } else {
          console.error('Connection check failed:', error)
        }
      }
      
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
      dragStateRef.current.offsetX = event.clientX - (centerX + agentPositionRef.current.x)
      dragStateRef.current.offsetY = event.clientY - (centerY + agentPositionRef.current.y)
      event.target?.setPointerCapture?.(event.pointerId)
      recordInteraction('agent_drag_start', { ...agentPositionRef.current })
    },
    [recordInteraction],
  )

  const onDrag = useCallback(
    (event) => {
      if (!dragStateRef.current.dragging) return
      const canvasElement = canvasRef.current?.getElement?.()
      if (!canvasElement) return

      const rect = canvasElement.getBoundingClientRect()
      const centerX = rect.left + rect.width / 2
      const centerY = rect.top + rect.height / 2

      const nextX = event.clientX - centerX - dragStateRef.current.offsetX
      const nextY = event.clientY - centerY - dragStateRef.current.offsetY

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
    },
    [updateAgentPosition],
  )

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
      if (!channel) {
        return
      }

      setActiveChannelId(channel.id)
      setSelectedAgent(channel.meta ?? null)
      recordInteraction('channel_selected', {
        channelId: channel.id,
        name: channel.name,
        status: channel.status,
        statusLabel: channel.statusLabel,
      })

      const target = extractAgentTarget(channel.meta ?? channel)
      if (target) {
        setChannelLoading(true)
        setChannelError(null)
        void (async () => {
          try {
            const agent = await agentService.resolveAgent(target, sessionId)
            const refreshed = buildChannelFromAgent(agent)
            setSelectedAgent(agent)
            if (refreshed) {
              setChannels((prev) => {
                const others = prev.filter((item) => item.id !== channel.id)
                return [refreshed, ...others]
              })
              setActiveChannelId(refreshed.id)
            }
          } catch (error) {
            const message = extractErrorMessage(error)
            console.error('Failed to resolve agent', error)
            setChannelError(message)
          } finally {
            setChannelLoading(false)
          }
        })()
      }

      void fetchAndOpenUiResource(
        MCP_UI_TARGETS.channelDetail,
        {
          channel_id: channel.id,
          did: channel.meta?.did,
          cid: channel.meta?.cid,
          ipns: channel.meta?.ipns,
        },
        { channelId: channel.id },
      )
    },
    [
      buildChannelFromAgent,
      extractAgentTarget,
      extractErrorMessage,
      fetchAndOpenUiResource,
      recordInteraction,
      sessionId,
    ],
  )

  const resolveExistingAgentTarget = useCallback(
    async (target) => {
      const parsedTarget = target?.trim()
      if (!parsedTarget) {
        throw new Error('请输入 IPNS / CID / DID 标识')
      }
      setChannelLoading(true)
      setChannelError(null)
      recordInteraction('create_channel', { target: parsedTarget })
      try {
        const agent = await agentService.resolveAgent(parsedTarget, sessionId)
        const channel = buildChannelFromAgent(agent)
        if (!channel) {
          throw new Error('解析结果为空')
        }
        setChannels((prev) => {
          const others = prev.filter((item) => item.id !== channel.id)
          return [channel, ...others]
        })
        setActiveChannelId(channel.id)
        setSelectedAgent(agent)
        setCreateAgentModalOpen(false)
      } catch (error) {
        const message = extractErrorMessage(error)
        setChannelError(message)
        recordInteraction('create_channel_failed', { target: parsedTarget, error: message })
        throw new Error(message)
      } finally {
        setChannelLoading(false)
      }
    },
    [
      buildChannelFromAgent,
      extractErrorMessage,
      recordInteraction,
      sessionId,
      setChannels,
      setSelectedAgent,
    ],
  )

  const handleCreateAgentSubmit = useCallback(
    async ({ name, roleDescription, avatarCid, mcpConfigCid, mcpPorts, diapIdentity }) => {
      setChannelLoading(true)
      setChannelError(null)
      recordInteraction('create_claude_agent', { name })
      try {
        const walletAddress =
          typeof window !== 'undefined' ? localStorage.getItem('wallet_address') : null
        const chainId =
          typeof window !== 'undefined' ? localStorage.getItem('wallet_chain_id') : null
        const detectedChain = resolveBackendChain({
          chainId,
          chain: activeChain,
        })

        const result = await agentService.createClaudeAgent({
          sessionId,
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
          setChannels((prev) => {
            const others = prev.filter((item) => item.id !== channel.id)
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
        recordInteraction('create_claude_agent_failed', { error: message })
        throw new Error(message)
      } finally {
        setChannelLoading(false)
      }
    },
    [
      activeChain,
      buildChannelFromAgent,
      extractErrorMessage,
      recordInteraction,
      resolveBackendChain,
      sessionId,
    ],
  )

  const createChannel = useCallback(() => {
    setCreateAgentModalOpen(true)
    recordInteraction('open_create_agent_modal')
  }, [recordInteraction])

  const closeCreateAgentModal = useCallback(() => {
    setCreateAgentModalOpen(false)
  }, [])

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

  const handleInspectSelectedAgent = useCallback(() => {
    if (!selectedAgent) {
      return
    }
    void fetchAndOpenUiResource(
      MCP_UI_TARGETS.agentProfile,
      {
        agent_id: selectedAgent.did || selectedAgent.cid || selectedAgent.ipns,
        session_id: sessionId,
        metadata: selectedAgent,
      },
      { source: 'agent_profile_panel' },
    )
  }, [fetchAndOpenUiResource, selectedAgent, sessionId])

  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      if (savedTheme) {
        setIsDarkMode(savedTheme === 'dark')
      } else if (typeof window !== 'undefined') {
        setIsDarkMode(window.matchMedia('(prefers-color-scheme: dark)').matches)
      }
    }

    try {
      initLanguage()
      checkConnection()
      createSession().then(() => setSessionReady(true)).catch(err => console.error('Failed to create session:', err))
      refreshWallet().catch(err => {
        console.warn('Failed to refresh wallet:', err)
        // 不中断流程，只是记录警告
      })
    } catch (error) {
      console.error('Error in AgentChat initialization:', error)
    }

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
          isLoading={isChannelLoading}
          errorMessage={channelError}
          onKeywordChange={handleChannelKeywordChange}
          onRefresh={refreshChannels}
          onSelectChannel={selectChannel}
          onCreateChannel={createChannel}
          isCollapsed={isLeftSidebarCollapsed}
          onToggleCollapse={toggleLeftSidebar}
        />

        <div className="agent-center">
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
          {selectedAgent && (
            <AgentProfilePanel agent={selectedAgent} onInspect={handleInspectSelectedAgent} />
          )}
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
            streamEvents={streamEvents}
            streamStatus={streamStatus}
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

      <CreateAgentModal
        isOpen={isCreateAgentModalOpen}
        onClose={closeCreateAgentModal}
        onSubmit={handleCreateAgentSubmit}
        onResolve={resolveExistingAgentTarget}
        sessionId={sessionId}
      />

      <McpModal
        resource={isUiModalOpen ? uiResource : null}
        onClose={closeUiResource}
        onUIAction={handleUiAction}
      />

      {/* DIAP Identity Panel - Show when a Claude Agent SDK is selected */}
      {activeChannelId && (
        <div
          className="diap-identity-overlay"
          style={{
            position: 'fixed',
            bottom: '20px',
            right: isSidebarCollapsed ? '20px' : '320px',
            zIndex: 1000,
            maxWidth: '400px',
          }}
        >
          <DiapIdentityPanel
            sessionId={activeChannelId}
            onClose={() => {
              // Optionally hide the panel
            }}
          />
        </div>
      )}

      <button type="button" className="language-switch" onClick={toggleLanguage}>
        {languageLabel}
      </button>
    </div>
  )
}

export default AgentChat
