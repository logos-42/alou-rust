import { useCallback, useEffect, useMemo, useRef } from 'react'
import agentService from '@/services/agentService'
import useAgentStore, { useAgentStoreHydration } from '@/stores/agentStore'
import { buildChannelFromAgent, extractAgentTarget, extractErrorMessage } from './agentUtils'
import { resolveBackendChain } from '@/hooks/useAgentChat'

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
  preferredChain, // 用于创建智能体时确定链
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

  // ==================== 模式管理 ====================
  // 获取当前活动频道的模式（从 channel.meta.mode 读取，默认为 'agent'）
  const currentMode = useMemo(() => {
    if (!activeChannelId) return 'agent'
    const channel = channels.find(c => c.id === activeChannelId)
    return channel?.meta?.mode || 'agent'
  }, [activeChannelId, channels])

  // 切换指定频道的模式
  const handleModeChange = useCallback((channelId, mode) => {
    setChannels((prev) =>
      prev.map((channel) =>
        channel.id === channelId
          ? {
              ...channel,
              meta: {
                ...channel.meta,
                mode,
              },
            }
          : channel
      )
    )
    recordInteraction('mode_changed', { channelId, mode })
  }, [recordInteraction, setChannels])

  // ==================== 智能体创建 ====================
  // 处理早期频道（头像上传后，但完整创建前）
  const handleEarlyChannel = useCallback(
    (earlyMetadata) => {
      const channel = buildChannelFromAgent(earlyMetadata)
      if (channel) {
        // 确保 tempId 保存在 channel.meta 中用于后续匹配
        const tempId = earlyMetadata.cid && earlyMetadata.cid.startsWith('temp_') 
          ? earlyMetadata.cid 
          : null
        if (tempId) {
          channel.tempId = tempId
          if (channel.meta) {
            channel.meta.tempId = tempId
          }
        }
        
        setChannels((prev) => {
          // 如果已有相同 tempId 的频道，更新它；否则添加新频道
          if (tempId) {
            const existingIndex = prev.findIndex((item) => 
              item.tempId === tempId || 
              item.id === tempId || 
              item.meta?.tempId === tempId ||
              item.meta?.cid === tempId
            )
            if (existingIndex >= 0) {
              // 更新现有频道（保留位置）
              const updated = [...prev]
              updated[existingIndex] = channel
              console.log('[useChannelManager] 更新早期频道（头像已上传）:', tempId)
              return updated
            }
          }
          
          // 移除可能重复的频道（相同 ID）
          const others = prev.filter((item) => item.id !== channel.id)
          console.log('[useChannelManager] 添加早期频道:', channel.id, 'tempId:', tempId)
          return [channel, ...others]
        })
        setActiveChannelId(channel.id)
        setSelectedAgent(earlyMetadata)
        console.log('[useChannelManager] 早期频道已显示，tempId:', tempId, '等待完整创建...')
      }
    },
    [setChannels, setActiveChannelId, setSelectedAgent],
  )

  // 创建智能体
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
              console.log('[useChannelManager] 找到临时频道:', oldChannel.id, '-> 更新为:', channel.id)
              updated[tempIndex] = channel
              return updated
            }

            // 没有找到临时频道，检查是否已存在相同 ID 的频道
            const existingIndex = prev.findIndex((item) => item.id === channel.id)
            if (existingIndex >= 0) {
              const updated = [...prev]
              updated[existingIndex] = channel
              console.log('[useChannelManager] 更新已存在的频道:', channel.id)
              return updated
            }

            // 添加新频道
            console.log('[useChannelManager] 添加新频道:', channel.id)
            return [channel, ...prev]
          })
          setActiveChannelId(channel.id)
        }

        setSelectedAgent(metadata)

        // 保存到本地存储
        saveAgentToStorage(metadata)

        console.log('[useChannelManager] 智能体创建/更新完成:', metadata.did || metadata.cid)
        return result
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        if (!isBackgroundUpdate) {
          setChannelError(message)
        }
        recordInteraction('create_claude_agent_failed', { error: message })
        console.error('[useChannelManager] 创建智能体失败:', message)
        throw new Error(message)
      } finally {
        if (!isBackgroundUpdate) {
          setChannelLoading(false)
        }
      }
    },
    [preferredChain, recordInteraction, saveAgentToStorage, sessionId, setChannels, setActiveChannelId, setSelectedAgent, setChannelLoading, setChannelError],
  )

  // 删除频道和本地存储的智能体
  const deleteChannel = useCallback(async (channel) => {
    if (!channel) return false
    
    // 收集所有可能的标识符，用于匹配存储中的智能体
    const possibleIds = [
      channel.id,
      channel.meta?.ipns,
      channel.meta?.cid,
      channel.meta?.did,
      channel.meta?.sessionId,
    ].filter(Boolean)
    
    const agentSessionId = channel.meta?.sessionId
    
    console.log('[useChannelManager] 开始删除智能体:', {
      channelId: channel.id,
      channelName: channel.name,
      possibleIds,
      agentSessionId,
    })
    
    // 1. 从频道列表中删除
    setChannels((prev) => {
      const filtered = prev.filter((c) => c.id !== channel.id)
      console.log('[useChannelManager] 从频道列表删除，剩余:', filtered.length)
      return filtered
    })
    
    // 2. 从本地存储中删除（尝试所有可能的 ID）
    let deletedFromStore = false
    const beforeCount = useAgentStore.getState().agents.length
    
    for (const id of possibleIds) {
      try {
        // 检查是否存在该智能体
        const existingAgent = useAgentStore.getState().agents.find(
          a => a.id === id || a.sessionId === id || a.ipns === id || a.cid === id || a.did === id
        )
        
        if (existingAgent) {
          removeAgentFromStore(id)
          const afterCount = useAgentStore.getState().agents.length
          
          if (afterCount < beforeCount) {
            deletedFromStore = true
            console.log('[useChannelManager] 已从本地存储删除智能体 (使用 ID:', id, '):', channel.name, '存储中的 ID:', existingAgent.id)
            break // 找到并删除后退出循环
          }
        }
      } catch (error) {
        console.warn('[useChannelManager] 尝试删除 ID', id, '失败:', error)
      }
    }
    
    if (!deletedFromStore) {
      console.warn('[useChannelManager] 未能从本地存储找到并删除智能体，可能 ID 不匹配。尝试的 ID:', possibleIds, '存储中的智能体:', useAgentStore.getState().agents.map(a => ({ id: a.id, sessionId: a.sessionId, ipns: a.ipns, cid: a.cid })))
      // 即使没找到，也继续执行其他清理操作
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
  }, [activeChannelId, recordInteraction, removeAgentFromStore, setActiveChannelId, setChannels, setSelectedAgent, storedAgents.length])

  return {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    saveAgentToStorage,
    deleteChannel,
    // 模式管理
    currentMode,
    handleModeChange,
    // 智能体创建
    handleCreateAgentSubmit,
    handleEarlyChannel,
    // 其他
    hasHydrated,
    storedAgents,
    channelRequestIdRef,
    searchDebounceRef,
  }
}

