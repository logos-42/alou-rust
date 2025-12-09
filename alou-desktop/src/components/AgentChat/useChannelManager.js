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
  loadMessagesFromIpfs, // 新增：加载历史消息的回调
}) => {
  const channelRequestIdRef = useRef(0)
  const searchDebounceRef = useRef(null)
  const hasLoadedFromStorageRef = useRef(false) // 防止重复从本地加载

  // 本地持久化
  const hasHydrated = useAgentStoreHydration()
  const storedAgents = useAgentStore((state) => state.agents)
  const addAgentToStore = useAgentStore((state) => state.addAgent)
  const removeAgentFromStore = useAgentStore((state) => state.removeAgent)
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
        // 没有搜索关键词时，不要覆盖本地已加载的频道
        if (!query) {
          const session = await agentService.getSession(sessionId)
          const metadata = session?.agent_metadata
          const channel = buildChannelFromAgent(metadata)

          applyLatest(() => {
            if (channel) {
              // 合并而不是替换：将后端返回的频道添加到列表（如果不存在）
              setChannels((prev) => {
                const existingIds = new Set(prev.map(c => c.id))
                if (existingIds.has(channel.id)) {
                  return prev // 已存在，不做改变
                }
                return [channel, ...prev]
              })
              // 只有当前没有选中的频道时才设置
              setActiveChannelId((prev) => prev || channel.id)
              setSelectedAgent((prev) => prev || metadata)
            }
            // 注意：如果后端没有返回频道，保持现有列表不变（本地存储的智能体）
          })

          return channel ? [channel] : []
        }

        // 有搜索关键词时，显示搜索结果
        const response = await agentService.searchAgents(query)
        const agents = Array.isArray(response?.agents) ? response.agents : []
        const mapped = agents.map((agent) => buildChannelFromAgent(agent)).filter(Boolean)

        applyLatest(() => {
          if (mapped.length > 0) {
            setChannels(mapped)
            if (!mapped.some((channel) => channel.id === activeChannelId)) {
              setActiveChannelId(mapped[0].id)
              setSelectedAgent(mapped[0].meta)
            }
          } else {
            // 搜索无结果时显示提示，但不清空列表（用户可以清除搜索词看到所有频道）
            setChannelError('未找到匹配的智能体')
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
          // 出错时也不要清空本地已有的频道列表
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

      // 直接选中，不创建新频道
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

      // 尝试从 IPFS 加载历史消息
      const meta = channel.meta ?? {}
      if (meta.messages_cid && loadMessagesFromIpfs) {
        loadMessagesFromIpfs(channel.id, meta.messages_cid).catch((err) => {
          console.error('[useChannelManager] 加载历史消息失败:', err)
        })
      }

      // 只有当需要解析更多信息时才调用后端
      const target = extractAgentTarget(channel.meta ?? channel)
      if (!target) {
        return
      }

      // 如果已经有完整信息，不需要再次解析
      if (meta.did && meta.display_name) {
        // 已有完整数据，无需重新解析
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
          // 更新选中的智能体信息，但保持原 channel.id 不变
          setSelectedAgent(mergedAgent)
          // 原地更新频道信息，不改变 ID，不添加新频道
          setChannels((prev) => 
            prev.map((item) => 
              item.id === channel.id 
                ? { ...item, meta: mergedAgent, name: mergedAgent.display_name || mergedAgent.name || item.name }
                : item
            )
          )
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
      loadMessagesFromIpfs,
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

  // 从本地存储加载智能体到 channels（在 hydration 完成后，只执行一次）
  useEffect(() => {
    // 防止重复加载
    if (hasLoadedFromStorageRef.current) {
      return
    }
    
    if (!hasHydrated || !isSessionReady) {
      return
    }

    // 标记已加载
    hasLoadedFromStorageRef.current = true

    // 如果本地存储有智能体，加载到 channels
    if (storedAgents && storedAgents.length > 0) {
      console.log(`[useChannelManager] 从本地存储加载 ${storedAgents.length} 个智能体（一次性）`)
      
      const localChannels = storedAgents
        .map(agent => buildChannelFromAgent(agent))
        .filter(Boolean)
      
      if (localChannels.length > 0) {
        setChannels(prev => {
          // 合并去重（本地存储的智能体添加到列表）
          const existingIds = new Set(prev.map(c => c.id))
          const newChannels = localChannels.filter(lc => !existingIds.has(lc.id))
          
          if (newChannels.length > 0) {
            console.log(`[useChannelManager] 合并 ${newChannels.length} 个本地智能体到频道列表`)
            return [...newChannels, ...prev]
          }
          return prev
        })
        
        // 如果没有活动频道，选择第一个本地智能体
        setActiveChannelId(prev => {
          if (!prev && localChannels.length > 0) {
            setSelectedAgent(localChannels[0].meta)
            return localChannels[0].id
          }
          return prev
        })
      }
    }
  }, [hasHydrated, isSessionReady, storedAgents, setChannels, setActiveChannelId, setSelectedAgent])

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

  // 删除频道和本地存储的智能体
  const deleteChannel = useCallback(async (channel) => {
    if (!channel) return false
    
    const agentId = channel.meta?.ipns || channel.meta?.cid || channel.meta?.did || channel.id
    const agentSessionId = channel.meta?.sessionId
    
    // 1. 从频道列表中删除
    setChannels((prev) => {
      const filtered = prev.filter((c) => c.id !== channel.id)
      return filtered
    })
    
    // 2. 从本地存储中删除
    try {
      removeAgentFromStore(agentId)
      console.log('[useChannelManager] 已从本地存储删除智能体:', channel.name, agentId)
    } catch (error) {
      console.error('[useChannelManager] 删除本地智能体失败:', error)
    }
    
    // 3. 删除 localStorage 中的 DIAP identity 映射
    if (agentSessionId && typeof window !== 'undefined' && window.localStorage) {
      try {
        localStorage.removeItem(`diap_identity_${agentSessionId}`)
        console.log('[useChannelManager] 已删除 DIAP identity 映射:', agentSessionId)
      } catch (error) {
        console.error('[useChannelManager] 删除 DIAP identity 映射失败:', error)
      }
    }
    
    // 4. 调用后端 API 删除 session（异步，不阻塞 UI）
    if (agentSessionId) {
      agentService.deleteSession(agentSessionId)
        .then(() => {
          console.log('[useChannelManager] 已删除后端 session:', agentSessionId)
        })
        .catch((error) => {
          console.error('[useChannelManager] 删除后端 session 失败:', error)
        })
    }
    
    // 5. 如果删除的是当前活动频道，清除选中状态
    if (channel.id === activeChannelId) {
      setActiveChannelId(null)
      setSelectedAgent(null)
    }
    
    recordInteraction('delete_channel', { channelId: channel.id, name: channel.name })
    return true
  }, [activeChannelId, recordInteraction, removeAgentFromStore, setActiveChannelId, setChannels, setSelectedAgent])

  return {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    saveAgentToStorage,
    deleteChannel,
    hasHydrated,
    storedAgents,
    channelRequestIdRef,
    searchDebounceRef,
  }
}

