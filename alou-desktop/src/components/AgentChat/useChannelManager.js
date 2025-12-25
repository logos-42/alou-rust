import { useCallback, useEffect, useMemo, useRef } from 'react'
import agentService from '@/services/agentService'
import useAgentStore, { useAgentStoreHydration } from '@/stores/agentStore'
import { buildChannelFromAgent, extractAgentTarget, extractErrorMessage } from './agentUtils'
import { resolveBackendChain } from '@/hooks/useAgentChat'
import { isIpns, isCid } from '@/services/utils/ipnsUtils'

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
  const resolveExistingAgentTargetRef = useRef(null) // 用于在 loadChannelList 中访问 resolveExistingAgentTarget

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

        // 有搜索关键词时，首先检测是否为 IPNS/CID 格式
        const isIpnsFormat = isIpns(query)
        const isCidFormat = isCid(query)

        // 如果是 IPNS/CID，独立解析并添加到频道列表
        if (isIpnsFormat || isCidFormat) {
          try {
            console.log('[useChannelManager] 检测到 IPNS/CID 格式，开始解析:', query)
            // 调用独立的解析函数，会自动添加到频道列表
            if (resolveExistingAgentTargetRef.current) {
              await resolveExistingAgentTargetRef.current(query)
            } else {
              throw new Error('解析功能未初始化')
            }
            // 解析成功后，返回空数组（因为已经通过 resolveExistingAgentTarget 添加到列表）
            return []
          } catch (error) {
            const message = extractErrorMessage(error)
            console.error('[useChannelManager] IPNS/CID 解析失败:', message)
            applyLatest(() => {
              setChannelError(message || '解析失败，请检查 IPNS/CID 是否正确')
            })
            return []
          }
        }

        // 如果不是 IPNS/CID，进行关键词搜索
        // 先搜索本地已有的频道
        const lowerQuery = query.toLowerCase().trim()
        
        // 改进匹配逻辑：单字符时使用前缀匹配，多字符时使用包含匹配
        // 只匹配名称和描述，不匹配 DID/IPNS/CID 等标识符
        const isSingleChar = lowerQuery.length === 1
        const localMatches = channels.filter((channel) => {
          const channelName = (channel.name || '').toLowerCase()
          const agentName = (channel.meta?.display_name || channel.meta?.name || '').toLowerCase()
          const agentDescription = (channel.meta?.role_description || '').toLowerCase()
          
          // 跳过没有有效名称的频道（只有 DID/IPNS/CID 标识符）
          if (!channelName && !agentName) {
            return false
          }
          
          let matches = false
          if (isSingleChar) {
            // 单字符：只匹配以该字符开头的名称（不匹配标识符）
            matches = (channelName && channelName.startsWith(lowerQuery)) || 
                     (agentName && agentName.startsWith(lowerQuery))
          } else {
            // 多字符：使用包含匹配（只匹配名称和描述）
            matches = (channelName && channelName.includes(lowerQuery)) || 
                     (agentName && agentName.includes(lowerQuery)) || 
                     (agentDescription && agentDescription.includes(lowerQuery))
          }
          return matches
        })

        // 调用后端 API 搜索
        let remoteAgents = []
        try {
          const response = await agentService.searchAgents(query)
          remoteAgents = Array.isArray(response?.agents) ? response.agents : []
          
          // 过滤掉没有名称的 agent（只有 DID/IPNS/CID 标识符，没有实际名称）
          const validAgents = remoteAgents.filter(agent => {
            if (!agent) return false
            // 必须有有效的名称（不是空字符串）
            const hasValidName = !!(agent.name && agent.name.trim()) || 
                                !!(agent.display_name && agent.display_name.trim())
            // 如果从 did_document 中可以解析出名称，也算有效
            const hasNameInDidDocument = agent.did_document && 
              typeof agent.did_document === 'object' &&
              (agent.did_document.name || 
               (agent.did_document.service && Array.isArray(agent.did_document.service) && 
                agent.did_document.service.some(s => s.serviceEndpoint?.name)))
            const isValid = hasValidName || hasNameInDidDocument
            return isValid
          })
          remoteAgents = validAgents
        } catch (error) {
          console.warn('[useChannelManager] 后端搜索失败，仅使用本地结果:', error)
        }

        // 将远程搜索结果转换为频道格式
        const remoteChannels = remoteAgents.map((agent) => buildChannelFromAgent(agent)).filter(Boolean)

        // 合并本地和远程结果，本地结果优先，去重
        const localIds = new Set(localMatches.map(c => c.id))
        const uniqueRemoteChannels = remoteChannels.filter(c => !localIds.has(c.id))
        const mergedChannels = [...localMatches, ...uniqueRemoteChannels]
        
        const mapped = mergedChannels

        applyLatest(() => {
          if (mapped.length > 0) {
            setChannels(mapped)
            // 优先选择本地匹配的频道，如果没有则选择第一个
            const preferredChannel = localMatches.length > 0 ? localMatches[0] : mapped[0]
            if (!mapped.some((channel) => channel.id === activeChannelId)) {
              setActiveChannelId(preferredChannel.id)
              setSelectedAgent(preferredChannel.meta)
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
      channels,
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
    // 延迟加载，确保本地存储的频道先加载完成
    // 这样可以避免本地存储的频道被覆盖
    const timer = setTimeout(() => {
      loadChannelListRef.current(channelKeyword)
    }, 100) // 给本地存储加载一点时间
    return () => clearTimeout(timer)
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
      console.log('[useChannelManager] 开始解析节点:', parsedTarget)
      setChannelLoading(true)
      setChannelError(null)
      recordInteraction('create_channel', { target: parsedTarget })
      try {
        console.log('[useChannelManager] 调用 agentService.resolveAgent...')
        const agent = await agentService.resolveAgent(parsedTarget, sessionId)
        console.log('[useChannelManager] 解析成功，agent 数据:', {
          did: agent?.did,
          cid: agent?.cid,
          ipns: agent?.ipns,
          name: agent?.name || agent?.display_name,
          hasDidDocument: !!agent?.did_document,
        })
        
        const channel = buildChannelFromAgent(agent)
        console.log('[useChannelManager] 构建的 channel:', {
          id: channel?.id,
          name: channel?.name,
          hasMeta: !!channel?.meta,
        })
        
        if (!channel) {
          console.error('[useChannelManager] buildChannelFromAgent 返回 null，agent:', agent)
          throw new Error('解析结果为空：无法构建频道对象')
        }
        
        if (!channel.id) {
          console.error('[useChannelManager] channel.id 为空，channel:', channel)
          throw new Error('频道 ID 为空：无法添加到列表')
        }
        
        console.log('[useChannelManager] 准备添加到频道列表，channel.id:', channel.id)
        setChannels((prev) => {
          console.log('[useChannelManager] 当前频道列表长度:', prev.length)
          const others = prev.filter((item) => item.id !== channel.id)
          const newChannels = [channel, ...others]
          console.log('[useChannelManager] 更新后频道列表长度:', newChannels.length, '新增的频道:', channel.name)
          return newChannels
        })
        
        console.log('[useChannelManager] 设置活动频道 ID:', channel.id)
        setActiveChannelId(channel.id)
        setSelectedAgent(agent)
        
        // 保存到本地存储
        // 重要：使用与 buildChannelFromAgent 相同的ID生成逻辑，确保一致性
        try {
          // 使用与频道相同的ID逻辑（去掉IPNS前缀以便统一）
          const agentIdForStore = channel.id
          console.log('[useChannelManager] 准备保存到本地存储，使用频道ID:', agentIdForStore)
          const result = importAgentToStore({
            ...agent,
            sessionId,
            id: agentIdForStore, // 使用频道ID，确保一致性
          })
          if (result.isNew) {
            console.log('[useChannelManager] ✅ 从网络导入新智能体到本地存储:', result.agent.id)
          } else {
            console.log('[useChannelManager] ℹ️ 智能体已存在于本地存储:', result.agent.id)
          }
        } catch (storeError) {
          console.error('[useChannelManager] ❌ 导入智能体到本地存储失败:', storeError)
        }
        
        console.log('[useChannelManager] ✅ 节点解析并加载完成:', channel.name, 'ID:', channel.id)
        return channel
      } catch (error) {
        const message = extractErrorMessage(error)
        console.error('[useChannelManager] ❌ 解析节点失败:', message, error)
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

  // 更新 ref，使 loadChannelList 可以访问 resolveExistingAgentTarget
  useEffect(() => {
    resolveExistingAgentTargetRef.current = resolveExistingAgentTarget
  }, [resolveExistingAgentTarget])

  // 从本地存储加载智能体到 channels（在 hydration 完成后，只执行一次）
  // 注意：这个逻辑应该在 loadChannelList 之前执行，确保本地存储的频道优先加载
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
      const localChannels = storedAgents
        .map((agent) => {
          const channel = buildChannelFromAgent(agent)
          if (!channel) {
            console.error(`[useChannelManager] 无法构建频道，agent:`, agent)
          }
          return channel
        })
        .filter(Boolean)
      
      if (storedAgents.length !== localChannels.length) {
        console.warn(`[useChannelManager] 警告: ${storedAgents.length - localChannels.length} 个智能体无法构建频道`)
      }
      
      if (localChannels.length > 0) {
        setChannels(prev => {
          // 合并去重（本地存储的智能体添加到列表）
          const existingIds = new Set(prev.map(c => c.id))
          const newChannels = localChannels.filter(lc => !existingIds.has(lc.id))
          
          if (newChannels.length > 0) {
            return [...newChannels, ...prev]
          }
          
          if (localChannels.length !== prev.length) {
            console.warn(`[useChannelManager] 警告: 本地存储有 ${localChannels.length} 个智能体，但频道列表只有 ${prev.length} 个`)
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
      } else {
        console.warn(`[useChannelManager] 本地存储有 ${storedAgents.length} 个智能体，但无法构建任何频道`)
      }
    }
  }, [hasHydrated, isSessionReady, storedAgents, setChannels, setActiveChannelId, setSelectedAgent])

  // 保存新创建的智能体到本地存储
  // 重要：使用与 buildChannelFromAgent 相同的ID生成逻辑，确保保存和加载时的ID一致
  const saveAgentToStorage = useCallback((agentMetadata) => {
    try {
      // 先构建频道以获取一致的ID
      const channel = buildChannelFromAgent(agentMetadata)
      const agentId = channel ? channel.id : (agentMetadata.ipns || agentMetadata.cid || agentMetadata.did || `agent_${Date.now()}`)
      
      console.log('[useChannelManager] 保存智能体到本地存储，使用ID:', agentId)
      addAgentToStore({
        ...agentMetadata,
        sessionId,
        id: agentId, // 使用与频道相同的ID，确保一致性
      })
      console.log('[useChannelManager] ✅ 智能体已保存到本地存储:', agentMetadata.display_name || agentMetadata.name, 'ID:', agentId)
      return true
    } catch (error) {
      console.error('[useChannelManager] ❌ 保存智能体到本地存储失败:', error)
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

  // 导入已解析的智能体（从 CreateAgentModal 导入）
  const handleImportAgent = useCallback(
    async (agent) => {
      if (!agent) {
        console.error('[useChannelManager] handleImportAgent 接收到 null/undefined agent')
        throw new Error('无效的智能体数据')
      }
      
      console.log('[useChannelManager] 开始导入已解析的智能体:', {
        id: agent.id,
        ipns: agent.ipns,
        did: agent.did,
        cid: agent.cid,
        name: agent.name || agent.display_name,
      })
      
      setChannelLoading(true)
      setChannelError(null)
      
      try {
        const channel = buildChannelFromAgent(agent)
        console.log('[useChannelManager] 构建的 channel:', {
          id: channel?.id,
          name: channel?.name,
          hasMeta: !!channel?.meta,
        })
        
        if (!channel) {
          console.error('[useChannelManager] buildChannelFromAgent 返回 null，agent:', agent)
          throw new Error('无法构建频道对象')
        }
        
        if (!channel.id) {
          console.error('[useChannelManager] channel.id 为空，channel:', channel)
          throw new Error('频道 ID 为空：无法添加到列表')
        }
        
        console.log('[useChannelManager] 准备添加到频道列表，channel.id:', channel.id)
        setChannels((prev) => {
          console.log('[useChannelManager] 当前频道列表长度:', prev.length)
          const others = prev.filter((item) => item.id !== channel.id)
          const newChannels = [channel, ...others]
          console.log('[useChannelManager] 更新后频道列表长度:', newChannels.length, '新增的频道:', channel.name)
          return newChannels
        })
        
        console.log('[useChannelManager] 设置活动频道 ID:', channel.id)
        setActiveChannelId(channel.id)
        setSelectedAgent(agent)
        
        // 保存到本地存储
        // 重要：使用与 buildChannelFromAgent 相同的ID生成逻辑，确保一致性
        try {
          // 使用与频道相同的ID逻辑
          const agentIdForStore = channel.id
          console.log('[useChannelManager] 准备保存到本地存储，使用频道ID:', agentIdForStore)
          const result = importAgentToStore({
            ...agent,
            sessionId,
            id: agentIdForStore, // 使用频道ID，确保一致性
          })
          if (result.isNew) {
            console.log('[useChannelManager] ✅ 从网络导入新智能体到本地存储:', result.agent.id)
          } else {
            console.log('[useChannelManager] ℹ️ 智能体已存在于本地存储:', result.agent.id)
          }
        } catch (storeError) {
          console.error('[useChannelManager] ❌ 导入智能体到本地存储失败:', storeError)
        }
        
        console.log('[useChannelManager] ✅ 智能体导入完成:', channel.name, 'ID:', channel.id)
        return channel
      } catch (error) {
        const message = extractErrorMessage(error)
        console.error('[useChannelManager] ❌ 导入智能体失败:', message, error)
        setChannelError(message)
        throw new Error(message)
      } finally {
        setChannelLoading(false)
      }
    },
    [
      importAgentToStore,
      sessionId,
      setActiveChannelId,
      setChannelError,
      setChannelLoading,
      setChannels,
      setSelectedAgent,
    ],
  )

  return {
    loadChannelList,
    refreshChannels,
    handleChannelKeywordChange,
    selectChannel,
    resolveExistingAgentTarget,
    handleImportAgent,
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

