import { useCallback, useEffect, useRef } from 'react'
import agentService from '@/services/agentService'
import useAgentStore, { useAgentStoreHydration } from '@/stores/agentStore'
import { buildChannelFromAgent, extractAgentTarget, extractErrorMessage } from './agentUtils'

export const useChannelManager = ({
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
}) => {
  const channelRequestIdRef = useRef(0)
  const searchDebounceRef = useRef(null)

  // 本地持久化
  const hasHydrated = useAgentStoreHydration()
  const storedAgents = useAgentStore((state) => state.agents)
  const addAgentToStore = useAgentStore((state) => state.addAgent)
  const importAgentToStore = useAgentStore((state) => state.importAgent)

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
          const isConnectionError =
            error.code === 'ECONNREFUSED' ||
            error.code === 'ERR_NETWORK' ||
            error.message?.includes('ERR_CONNECTION_REFUSED') ||
            error.message?.includes('Failed to fetch') ||
            !error.response

          const now = Date.now()
          const lastErrorTime = window.__lastLoadChannelsError || 0

          if (isConnectionError) {
            if (now - lastErrorTime > 10000) {
              window.__lastLoadChannelsError = now
              console.warn('[AgentChat] Cannot load agent channels: backend server unavailable.')
            }
          } else {
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
      channelKeyword,
      sessionId,
      setActiveChannelId,
      setChannelError,
      setChannelLoading,
      setChannels,
      setSelectedAgent,
    ],
  )

  const loadChannelListRef = useRef(loadChannelList)
  useEffect(() => {
    loadChannelListRef.current = loadChannelList
  }, [loadChannelList])

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
    loadChannelListRef.current(channelKeyword)
  }, [channelKeyword, isSessionReady])

  const refreshChannels = useCallback(() => {
    void loadChannelList(channelKeyword)
  }, [channelKeyword, loadChannelList])

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
    [loadChannelList, setChannelKeyword],
  )

  const selectChannel = useCallback(
    (channel) => {
      if (!channel) {
        return
      }

      setActiveChannelId(channel.id)
      setSelectedAgent(channel.meta ?? null)
      setChannelError(null)
      recordInteraction('channel_selected', {
        channelId: channel.id,
        name: channel.name,
        status: channel.status,
        statusLabel: channel.statusLabel,
      })
      openConversationPanel()

      const target = extractAgentTarget(channel.meta ?? channel)
      if (!target) {
        return
      }

      setChannelLoading(true)
      void (async () => {
        try {
          const resolvedAgent = await agentService.resolveAgent(target, sessionId)
          const originalMeta = channel.meta ?? {}
          const mergedAgent = {
            ...resolvedAgent,
            ipns: originalMeta.ipns || resolvedAgent.ipns,
            display_name: originalMeta.display_name || resolvedAgent.display_name,
            name: originalMeta.name || resolvedAgent.name,
            avatar_cid: originalMeta.avatar_cid || resolvedAgent.avatar_cid,
            avatar: originalMeta.avatar || resolvedAgent.avatar,
            avatar_url: originalMeta.avatar_url || resolvedAgent.avatar_url,
            agent_type: originalMeta.agent_type || resolvedAgent.agent_type,
            role_description: originalMeta.role_description || resolvedAgent.role_description,
            mcp_config_cid: originalMeta.mcp_config_cid || resolvedAgent.mcp_config_cid,
            mcp_ports: originalMeta.mcp_ports || resolvedAgent.mcp_ports,
          }
          const refreshed = buildChannelFromAgent(mergedAgent)
          setSelectedAgent(mergedAgent)
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
    },
    [
      openConversationPanel,
      recordInteraction,
      sessionId,
      setActiveChannelId,
      setChannelError,
      setChannelLoading,
      setChannels,
      setSelectedAgent,
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
        
        // 保存到本地存储
        try {
          const result = importAgentToStore({
            ...agent,
            sessionId,
            id: agent.ipns || agent.cid || agent.did || parsedTarget,
          })
          if (result.isNew) {
            console.log('[useChannelManager] 从网络导入新智能体到本地存储:', result.agent.id)
          }
        } catch (storeError) {
          console.error('[useChannelManager] 导入智能体到本地存储失败:', storeError)
        }
        
        return channel
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
      importAgentToStore,
      recordInteraction,
      sessionId,
      setActiveChannelId,
      setChannelError,
      setChannelLoading,
      setChannels,
      setSelectedAgent,
    ],
  )

  // 从本地存储加载智能体到 channels（在 hydration 完成后）
  useEffect(() => {
    if (!hasHydrated || !isSessionReady) {
      return
    }

    // 如果本地存储有智能体，加载到 channels
    if (storedAgents && storedAgents.length > 0) {
      console.log(`[useChannelManager] 从本地存储加载 ${storedAgents.length} 个智能体`)
      
      const localChannels = storedAgents
        .map(agent => buildChannelFromAgent(agent))
        .filter(Boolean)
      
      if (localChannels.length > 0) {
        setChannels(prev => {
          // 合并去重（本地存储的智能体添加到列表末尾）
          const existingIds = new Set(prev.map(c => c.id))
          const newChannels = localChannels.filter(lc => !existingIds.has(lc.id))
          
          if (newChannels.length > 0) {
            console.log(`[useChannelManager] 合并 ${newChannels.length} 个本地智能体到频道列表`)
            return [...prev, ...newChannels]
          }
          return prev
        })
        
        // 如果没有活动频道，选择第一个本地智能体
        if (!activeChannelId && localChannels.length > 0) {
          setActiveChannelId(localChannels[0].id)
          setSelectedAgent(localChannels[0].meta)
        }
      }
    }
  }, [hasHydrated, isSessionReady, storedAgents, activeChannelId, setChannels, setActiveChannelId, setSelectedAgent])

  // 保存新创建的智能体到本地存储
  const saveAgentToStorage = useCallback((agentMetadata) => {
    try {
      addAgentToStore({
        ...agentMetadata,
        sessionId,
        id: agentMetadata.ipns || agentMetadata.cid || agentMetadata.did || `agent_${Date.now()}`,
      })
      console.log('[useChannelManager] 智能体已保存到本地存储:', agentMetadata.display_name || agentMetadata.name)
      return true
    } catch (error) {
      console.error('[useChannelManager] 保存智能体到本地存储失败:', error)
      return false
    }
  }, [addAgentToStore, sessionId])

  return {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    saveAgentToStorage,
    hasHydrated,
    storedAgents,
    channelRequestIdRef,
    searchDebounceRef,
  }
}

