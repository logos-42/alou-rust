/**
 * useMemoryGroupChatManager - 基于内存存储的群聊管理器
 * 完全使用内存存储，避免localStorage空间限制
 */

import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useDiapGroupChat } from '@/hooks/useDiapGroupChat'
import { memoryStore } from '@/stores/memoryStore'

/**
 * useMemoryGroupChatManager - 内存群聊管理 Hook
 * 所有数据存储在内存中，提供快速的群聊体验
 */
export const useMemoryGroupChatManager = ({ 
  openConversationPanel, 
  activeChannelId, 
  localIdentity 
}) => {
  // 状态管理
  const [showGroupChat, setShowGroupChat] = useState(false)
  const [activeGroupId, setActiveGroupId] = useState(null)
  const [memoryStats, setMemoryStats] = useState(memoryStore.getStats())

  // 引用管理
  const openConversationPanelRef = useRef(openConversationPanel)
  const cleanupIntervalRef = useRef(null)

  // 更新引用
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // DIAP群聊Hook
  const diapGroupChat = useDiapGroupChat({
    localIdentity,
    onGroupCreated: (group, agents) => {
      console.log('[useMemoryGroupChatManager] DIAP群聊创建成功:', group.groupName)
      
      // 保存到内存存储
      const groupData = {
        groupId: group.groupId,
        groupName: group.groupName,
        description: group.description,
        topic: group.topic,
        agents: [
          // 添加当前用户
          {
            id: localIdentity?.did,
            did: localIdentity?.did,
            name: localIdentity?.name || '用户',
            avatar: localIdentity?.avatar,
            mode: 'user'
          },
          // 添加被邀请的智能体
          ...agents.map(agent => ({
            id: agent.did || agent.ipns || agent.cid || agent.id,
            did: agent.did,
            ipns: agent.ipns,
            cid: agent.cid,
            name: agent.name || agent.display_name || '智能体',
            avatar: agent.avatar || agent.avatar_url,
            mode: agent.mode || 'agent'
          }))
        ],
        channelId: activeChannelId,
        channelName: `频道 ${activeChannelId?.slice(-8) || 'Unknown'}`,
        createdAt: Date.now(),
        lastActivity: Date.now(),
        metadata: {
          type: 'diap_group_chat',
          diap_topic: group.topic,
          channel: activeChannelId ? { id: activeChannelId } : null,
          local: false
        }
      }

      memoryStore.setDiapGroup(group.groupId, groupData)
      
      // 设置为活跃群聊
      setActiveGroupId(group.groupId)
      setShowGroupChat(true)
      
      // 打开对话面板
      openConversationPanelRef.current?.()
      
      // 更新内存统计
      setMemoryStats(memoryStore.getStats())
      
      // 触发群聊显示事件
      window.dispatchEvent(
        new CustomEvent('cluster-action-created', {
          detail: { actionId: `diap_group_${group.groupId}` },
        }),
      )
    },
    onMessage: (groupId, message) => {
      console.log('[useMemoryGroupChatManager] 收到DIAP消息:', groupId, message.content)
      
      // 更新群聊的lastActivity
      const groupData = memoryStore.getItems([`diap_group_chat_${groupId}`])[`diap_group_chat_${groupId}`]
      if (groupData) {
        const parsedGroup = JSON.parse(groupData)
        parsedGroup.lastActivity = Date.now()
        
        // 添加消息到群聊（如果需要）
        if (!parsedGroup.messages) {
          parsedGroup.messages = []
        }
        parsedGroup.messages.push({
          id: message.id,
          from: message.from,
          fromName: message.fromName,
          content: message.content,
          type: message.type,
          timestamp: message.timestamp,
          metadata: message.metadata
        })
        
        // 限制消息数量，避免内存无限增长
        if (parsedGroup.messages.length > 100) {
          parsedGroup.messages = parsedGroup.messages.slice(-50) // 只保留最近50条消息
          console.log('[useMemoryGroupChatManager] 限制消息数量到50条')
        }
        
        memoryStore.setDiapGroup(groupId, parsedGroup)
        setMemoryStats(memoryStore.getStats())
      }
    },
    onError: (error) => {
      console.error('[useMemoryGroupChatManager] DIAP群聊错误:', error)
    }
  })

  // 获取当前活跃群聊
  const activeGroup = useMemo(() => {
    if (!activeGroupId) return null
    
    const groupData = memoryStore.getItems([`diap_group_chat_${activeGroupId}`])[`diap_group_chat_${activeGroupId}`]
    return groupData ? JSON.parse(groupData) : null
  }, [activeGroupId])

  // 获取当前活跃群聊的消息
  const activeGroupMessages = useMemo(() => {
    return activeGroup?.messages || []
  }, [activeGroup])

  // 获取当前频道的群聊列表
  const groupChatList = useMemo(() => {
    if (!activeChannelId) return []
    
    const allGroups = memoryStore.getDiapGroups()
    return allGroups.filter(group => 
      group.channelId === activeChannelId
    )
  }, [activeChannelId, memoryStats])

  // 创建群聊
  const createGroupChat = useCallback(async (groupName, agents = []) => {
    if (!localIdentity) {
      throw new Error('未设置本地身份，无法创建群聊')
    }

    if (!activeChannelId) {
      throw new Error('未选择频道，无法创建群聊')
    }

    try {
      console.log('[useMemoryGroupChatManager] 创建内存群聊:', groupName, agents.length)

      // 获取频道信息
      const channel = {
        id: activeChannelId,
        name: `频道 ${activeChannelId.slice(-8)}`,
      }

      // 使用DIAP创建群聊
      const group = await diapGroupChat.createGroupWithAgents({
        groupName,
        description: `${channel.name} 的内存群聊`,
        agents: agents,
        channel: channel,
        metadata: {
          channelId: activeChannelId,
          channelName: channel.name,
          storage: 'memory', // 标记为内存存储
          createdAt: Date.now()
        }
      })

      return group

    } catch (error) {
      console.error('[useMemoryGroupChatManager] 创建群聊失败:', error)
      throw error
    }
  }, [localIdentity, activeChannelId, diapGroupChat, openConversationPanelRef])

  // 发送消息
  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await diapGroupChat.sendMessage(groupId, content)
      console.log('[useMemoryGroupChatManager] 消息发送成功:', groupId, content)
    } catch (error) {
      console.error('[useMemoryGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [diapGroupChat])

  // 切换群聊
  const switchGroupChat = useCallback((groupId) => {
    const allGroups = memoryStore.getDiapGroups()
    if (allGroups.some(g => g.groupId === groupId)) {
      setActiveGroupId(groupId)
      console.log('[useMemoryGroupChatManager] 切换群聊:', groupId)
    }
  }, [])

  // 打开群聊
  const openGroupChat = useCallback(() => {
    if (activeGroupId) {
      setShowGroupChat(true)
      openConversationPanelRef.current?.()
    }
  }, [activeGroupId, openConversationPanelRef])

  // 关闭群聊
  const closeGroupChat = useCallback(() => {
    setShowGroupChat(false)
  }, [])

  // 删除群聊
  const deleteGroupChat = useCallback(async (groupId) => {
    try {
      await diapGroupChat.leaveGroup(groupId)
      
      // 从内存存储删除
      memoryStore.removeDiapGroup(groupId)
      
      // 如果删除的是当前活跃群聊，清除活跃状态
      if (activeGroupId === groupId) {
        setActiveGroupId(null)
        setShowGroupChat(false)
      }
      
      // 更新内存统计
      setMemoryStats(memoryStore.getStats())
      
      console.log('[useMemoryGroupChatManager] 群聊已删除:', groupId)
    } catch (error) {
      console.error('[useMemoryGroupChatManager] 删除群聊失败:', error)
    }
  }, [activeGroupId, diapGroupChat])

  // 内存清理
  const cleanupMemory = useCallback((options = {}) => {
    const removed = memoryStore.cleanup({
      maxAge: 30 * 60 * 1000, // 30分钟
      maxItems: 200, // 最多200个项目
      keepPatterns: [
        'diap_group_chat_', // 保留所有群聊数据
        `cluster_actions_active_id_${activeChannelId}`, // 保留当前频道的活跃ID
        activeGroupId ? `diap_group_chat_${activeGroupId}` : null // 保留当前活跃群聊
      ].filter(Boolean)
    })
    
    setMemoryStats(memoryStore.getStats())
    console.log(`[useMemoryGroupChatManager] 内存清理完成，删除了 ${removed} 个项目`)
    
    return removed
  }, [activeChannelId, activeGroupId])

  // 自动清理定时器
  useEffect(() => {
    // 每5分钟自动清理一次过期数据
    cleanupIntervalRef.current = setInterval(() => {
      cleanupMemory({ maxAge: 10 * 60 * 1000 }) // 清理10分钟前的数据
    }, 5 * 60 * 1000)

    return () => {
      if (cleanupIntervalRef.current) {
        clearInterval(cleanupIntervalRef.current)
      }
    }
  }, [cleanupMemory])

  // 监听DIAP群聊创建事件
  useEffect(() => {
    const handleClusterActionCreated = (event) => {
      const { actionId } = event.detail
      if (actionId && actionId.startsWith('diap_group_')) {
        const groupId = actionId.replace('diap_group_', '')
        setActiveGroupId(groupId)
        setShowGroupChat(true)
        openConversationPanelRef.current?.()
      }
    }

    window.addEventListener('cluster-action-created', handleClusterActionCreated)
    return () => {
      window.removeEventListener('cluster-action-created', handleClusterActionCreated)
    }
  }, [openConversationPanelRef])

  // 监听内存存储变化
  useEffect(() => {
    const handleStorageChange = () => {
      setMemoryStats(memoryStore.getStats())
    }

    memoryStore.addEventListener('*', handleStorageChange)
    return () => {
      memoryStore.removeEventListener('*', handleStorageChange)
    }
  }, [])

  return {
    // 状态
    showGroupChat,
    activeGroupId,
    activeGroup,
    activeGroupMessages,
    groupChatList,
    isLoading: diapGroupChat.isLoading,
    memoryStats,

    // DIAP群聊状态
    diapGroupChat,

    // 操作方法
    createGroupChat,
    sendMessage,
    switchGroupChat,
    openGroupChat,
    closeGroupChat,
    deleteGroupChat,
    cleanupMemory,

    // 内存管理
    clearMemory: () => {
      memoryStore.clear()
      setMemoryStats(memoryStore.getStats())
      console.log('[useMemoryGroupChatManager] 内存已清空')
    }
  }
}

export default useMemoryGroupChatManager
