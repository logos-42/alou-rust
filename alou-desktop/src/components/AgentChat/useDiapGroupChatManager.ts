/**
 * useDiapGroupChatManager - 基于DIAP SDK的群聊管理器
 * 完全替代原有的群聊系统，支持本地持久化和PubSub通信
 */

import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useDiapGroupChat } from '@/hooks/useDiapGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'
import { i18n } from '@/hooks/useI18n'

/**
 * 本地存储键名
 */
const STORAGE_KEYS = {
  GROUP_CHAT_PREFIX: 'diap_group_chat_',
  ACTIVE_GROUP: 'diap_active_group',
  SHOW_PANEL: 'diap_show_panel'
}

/**
 * 群聊持久化数据结构
 */
class PersistentGroupChat {
  constructor({
    groupId,
    groupName,
    description,
    topic,
    channelId,
    channelName,
    agents = [],
    messages = [],
    createdAt,
    lastActivity,
    metadata = {}
  }) {
    this.groupId = groupId
    this.groupName = groupName
    this.description = description
    this.topic = topic
    this.channelId = channelId
    this.channelName = channelName
    this.agents = agents
    this.messages = messages
    this.createdAt = createdAt || Date.now()
    this.lastActivity = lastActivity || Date.now()
    this.metadata = metadata
  }

  toJSON() {
    return {
      groupId: this.groupId,
      groupName: this.groupName,
      description: this.description,
      topic: this.topic,
      channelId: this.channelId,
      channelName: this.channelName,
      agents: this.agents,
      messages: this.messages,
      createdAt: this.createdAt,
      lastActivity: this.lastActivity,
      metadata: this.metadata
    }
  }

  static fromJSON(json) {
    return new PersistentGroupChat(json)
  }

  // 保存到本地存储
  save() {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${this.groupId}`
      localStorage.setItem(key, JSON.stringify(this.toJSON()))
      console.log('[PersistentGroupChat] 群聊已保存到本地:', this.groupId)
    } catch (error) {
      console.error('[PersistentGroupChat] 保存群聊失败:', error)
    }
  }

  // 从本地存储加载
  static load(groupId) {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${groupId}`
      const data = localStorage.getItem(key)
      if (data) {
        const json = JSON.parse(data)
        return PersistentGroupChat.fromJSON(json)
      }
    } catch (error) {
      console.error('[PersistentGroupChat] 加载群聊失败:', error)
    }
    return null
  }

  // 删除本地存储
  static delete(groupId) {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${groupId}`
      localStorage.removeItem(key)
      console.log('[PersistentGroupChat] 群聊已从本地删除:', groupId)
    } catch (error) {
      console.error('[PersistentGroupChat] 删除群聊失败:', error)
    }
  }

  // 获取所有本地群聊
  static loadAll() {
    try {
      const groups = []
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i)
        if (key && key.startsWith(STORAGE_KEYS.GROUP_CHAT_PREFIX)) {
          const data = localStorage.getItem(key)
          if (data) {
            const json = JSON.parse(data)
            groups.push(PersistentGroupChat.fromJSON(json))
          }
        }
      }
      return groups.sort((a, b) => b.lastActivity - a.lastActivity)
    } catch (error) {
      console.error('[PersistentGroupChat] 加载所有群聊失败:', error)
      return []
    }
  }
}

/**
 * DIAP群聊管理器Hook
 */
export const useDiapGroupChatManager = ({ 
  openConversationPanel, 
  activeChannelId,
  localIdentity 
}) => {
  // 状态管理
  const [showGroupChat, setShowGroupChat] = useState(false)
  const [activeGroupId, setActiveGroupIdState] = useState(null)
  const [persistentGroups, setPersistentGroups] = useState([])
  const [isLoading, setIsLoading] = useState(false)

  // 引用管理
  const loadedChannelRef = useRef(null)
  const openConversationPanelRef = useRef(openConversationPanel)

  // Store相关
  const { 
    getActiveAction, 
    setActiveAction, 
    getActions,
    addAction,
    loadChannelGroupChats 
  } = useClusterActionStore()

  // DIAP群聊Hook
  const diapGroupChat = useDiapGroupChat({
    localIdentity,
    onGroupCreated: (group, agents) => {
      console.log('[useDiapGroupChatManager] DIAP群聊创建成功:', group.groupName)
      
      // 创建持久化对象
      const persistentGroup = new PersistentGroupChat({
        groupId: group.groupId,
        groupName: group.groupName,
        description: group.description,
        topic: group.topic,
        channelId: activeChannelId,
        channelName: group.metadata?.channel?.name || '未知频道',
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
        metadata: {
          ...group.metadata,
          type: 'diap_group_chat',
          localIdentity: localIdentity?.did
        }
      })

      // 保存到本地
      persistentGroup.save()
      
      // 更新状态
      setPersistentGroups(prev => [...prev, persistentGroup])
      setActiveGroupIdState(group.groupId)
      setShowGroupChat(true)

      // 保存活跃群聊
      localStorage.setItem(STORAGE_KEYS.ACTIVE_GROUP, group.groupId)
      
      // 打开对话面板
      openConversationPanelRef.current?.()
    },
    onMessage: (groupId, message) => {
      console.log('[useDiapGroupChatManager] 收到消息:', groupId, message.content)
      
      // 更新持久化数据
      const persistentGroup = PersistentGroupChat.load(groupId)
      if (persistentGroup) {
        persistentGroup.messages.push({
          id: message.id,
          from: message.from,
          fromName: message.fromName,
          content: message.content,
          type: message.type,
          timestamp: message.timestamp,
          metadata: message.metadata
        })
        persistentGroup.lastActivity = Date.now()
        persistentGroup.save()
        
        // 更新状态
        setPersistentGroups(prev => 
          prev.map(g => g.groupId === groupId ? persistentGroup : g)
        )
      }
    },
    onError: (error) => {
      console.error('[useDiapGroupChatManager] DIAP群聊错误:', error)
    }
  })

  // 更新引用
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // 加载本地持久化的群聊
  const loadPersistentGroups = useCallback(() => {
    try {
      const groups = PersistentGroupChat.loadAll()
      console.log('[useDiapGroupChatManager] 加载本地群聊:', groups.length)
      setPersistentGroups(groups)
      
      // 恢复活跃群聊
      const savedActiveGroupId = localStorage.getItem(STORAGE_KEYS.ACTIVE_GROUP)
      if (savedActiveGroupId && groups.some(g => g.groupId === savedActiveGroupId)) {
        setActiveGroupIdState(savedActiveGroupId)
        
        // 恢复面板显示状态
        const savedShowPanel = localStorage.getItem(STORAGE_KEYS.SHOW_PANEL)
        if (savedShowPanel === 'true') {
          setShowGroupChat(true)
          setTimeout(() => {
            openConversationPanelRef.current?.()
          }, 100)
        }
      }
    } catch (error) {
      console.error('[useDiapGroupChatManager] 加载持久化群聊失败:', error)
    }
  }, [])

  // 初始化时加载本地群聊
  useEffect(() => {
    const timer = setTimeout(() => {
      loadPersistentGroups()
    }, 0)
    return () => clearTimeout(timer)
  }, [loadPersistentGroups])

  // 监听频道变化，加载对应群聊
  useEffect(() => {
    if (activeChannelId && loadedChannelRef.current !== activeChannelId) {
      loadedChannelRef.current = activeChannelId
      const loadingTimer = setTimeout(() => {
        setIsLoading(true)

        // 加载频道的群聊（从本地存储）
        setTimeout(() => {
          const channelGroups = persistentGroups.filter(g => g.channelId === activeChannelId)
          console.log('[useDiapGroupChatManager] 频道群聊:', activeChannelId, channelGroups.length)

          // 如果频道有群聊，恢复第一个为活跃
          if (channelGroups.length > 0) {
            const firstGroup = channelGroups[0]

            const setActiveTimer = setTimeout(() => {
              setActiveGroupIdState(firstGroup.groupId)
            }, 0)

            // 检查是否应该显示面板
            const savedShowPanel = localStorage.getItem(`${STORAGE_KEYS.SHOW_PANEL}_${activeChannelId}`)
            if (savedShowPanel === 'true') {
              const setShowTimer = setTimeout(() => {
                setShowGroupChat(true)
              }, 0)

              setTimeout(() => {
                openConversationPanelRef.current?.()
              }, 0)
            } else {
              const setShowTimer = setTimeout(() => {
                setShowGroupChat(false)
              }, 0)
            }
          } else {
            const setShowTimer = setTimeout(() => {
              setShowGroupChat(false)
            }, 0)
          }

          const setLoadingTimer = setTimeout(() => {
            setIsLoading(false)
          }, 0)
        }, 0)
      }, 0)

      return () => {
        clearTimeout(loadingTimer)
      }
    } else if (!activeChannelId) {
      loadedChannelRef.current = null

      const timer = setTimeout(() => {
        setShowGroupChat(false)
      }, 0)

      return () => clearTimeout(timer)
    }
  }, [activeChannelId, persistentGroups, openConversationPanel])

  // 创建群聊
  const createGroupChat = useCallback(async (groupName, agents = []) => {
    if (!localIdentity) {
      throw new Error('未设置本地身份，无法创建群聊')
    }

    if (!activeChannelId) {
      throw new Error('未选择频道，无法创建群聊')
    }

    try {
      setIsLoading(true)

      // 获取频道信息
      const channelActions = getActions(activeChannelId) || []
      const channel = {
        id: activeChannelId,
        name: `频道 ${activeChannelId.slice(-8)}`,
        // 可以从其他地方获取更详细的频道信息
      }

      // 使用DIAP创建群聊
      const group = await diapGroupChat.createGroupWithAgents({
        groupName,
        description: `${channel.name} 的群聊`,
        agents: agents,
        channel: channel,
        metadata: {
          channelId: activeChannelId,
          channelName: channel.name,
          createdAt: Date.now()
        }
      })

      setIsLoading(false)
      return group

    } catch (error) {
      setIsLoading(false)
      console.error('[useDiapGroupChatManager] 创建群聊失败:', error)
      throw error
    }
  }, [localIdentity, activeChannelId, getActions, diapGroupChat])

  // 发送消息
  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await diapGroupChat.sendMessage(groupId, content)
    } catch (error) {
      console.error('[useDiapGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [diapGroupChat])

  // 切换群聊
  const switchGroupChat = useCallback((groupId) => {
    if (persistentGroups.some(g => g.groupId === groupId)) {
      setActiveGroupIdState(groupId)
      localStorage.setItem(STORAGE_KEYS.ACTIVE_GROUP, groupId)
      console.log('[useDiapGroupChatManager] 切换群聊:', groupId)
    }
  }, [persistentGroups])

  // 关闭群聊面板
  const closeGroupChat = useCallback(() => {
    setShowGroupChat(false)
    if (activeChannelId) {
      localStorage.setItem(`${STORAGE_KEYS.SHOW_PANEL}_${activeChannelId}`, 'false')
    }
  }, [activeChannelId])

  // 打开群聊面板
  const openGroupChat = useCallback(() => {
    setShowGroupChat(true)
    if (activeChannelId) {
      localStorage.setItem(`${STORAGE_KEYS.SHOW_PANEL}_${activeChannelId}`, 'true')
    }
    openConversationPanelRef.current?.()
  }, [activeChannelId])

  // 删除群聊
  const deleteGroupChat = useCallback((groupId) => {
    try {
      // 从本地存储删除
      PersistentGroupChat.delete(groupId)
      
      // 更新状态
      setPersistentGroups(prev => prev.filter(g => g.groupId !== groupId))
      
      // 如果删除的是当前活跃群聊，清除活跃状态
      if (activeGroupId === groupId) {
        setActiveGroupIdState(null)
        localStorage.removeItem(STORAGE_KEYS.ACTIVE_GROUP)
        setShowGroupChat(false)
      }
      
      console.log('[useDiapGroupChatManager] 群聊已删除:', groupId)
    } catch (error) {
      console.error('[useDiapGroupChatManager] 删除群聊失败:', error)
    }
  }, [activeGroupId])

  // 获取当前活跃群聊
  const activeGroup = useMemo(() => {
    return persistentGroups.find(g => g.groupId === activeGroupId) || null
  }, [persistentGroups, activeGroupId])

  // 获取当前频道的群聊列表
  const channelGroupChats = useMemo(() => {
    if (!activeChannelId) return []
    return persistentGroups.filter(g => g.channelId === activeChannelId)
  }, [persistentGroups, activeChannelId])

  // 获取当前活跃群聊的消息
  const activeGroupMessages = useMemo(() => {
    return activeGroup?.messages || []
  }, [activeGroup])

  return {
    // 状态
    showGroupChat,
    activeGroupId,
    activeGroup,
    activeGroupMessages,
    persistentGroups,
    channelGroupChats,
    isLoading,
    diapGroupChat,

    // 操作方法
    createGroupChat,
    sendMessage,
    switchGroupChat,
    openGroupChat,
    closeGroupChat,
    deleteGroupChat,
    
    // 工具方法
    refreshGroups: loadPersistentGroups,
    clearError: diapGroupChat.clearError
  }
}

export default useDiapGroupChatManager
