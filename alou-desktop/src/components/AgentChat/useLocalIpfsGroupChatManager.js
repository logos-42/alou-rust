/**
 * useLocalIpfsGroupChatManager - 基于本地IPFS PubSub的群聊管理器
 * 使用本地IPFS节点创建pubsub，消息存储在本地内存和KV中
 */

import { useCallback, useEffect, useMemo, useState, useRef } from 'react'
import { useLocalIpfsGroupChat } from '@/hooks/useLocalIpfsGroupChat'
import useClusterActionStore from '@/stores/clusterActionStore'
import { i18n } from '@/hooks/useI18n'

/**
 * 本地存储键名
 */
const STORAGE_KEYS = {
  GROUP_CHAT_PREFIX: 'ipfs_group_chat_',
  ACTIVE_GROUP: 'ipfs_active_group',
  SHOW_PANEL: 'ipfs_show_panel'
}

/**
 * 群聊持久化数据结构
 */
class PersistentLocalGroupChat {
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
    return new PersistentLocalGroupChat(json)
  }

  // 保存到本地存储
  save() {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${this.groupId}`
      localStorage.setItem(key, JSON.stringify(this.toJSON()))
      console.log('[PersistentLocalGroupChat] 群聊已保存到本地:', this.groupId)
    } catch (error) {
      console.error('[PersistentLocalGroupChat] 保存群聊失败:', error)
    }
  }

  // 从本地存储加载
  static load(groupId) {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${groupId}`
      const data = localStorage.getItem(key)
      if (data) {
        const json = JSON.parse(data)
        return PersistentLocalGroupChat.fromJSON(json)
      }
    } catch (error) {
      console.error('[PersistentLocalGroupChat] 加载群聊失败:', error)
    }
    return null
  }

  // 删除本地存储
  static delete(groupId) {
    try {
      const key = `${STORAGE_KEYS.GROUP_CHAT_PREFIX}${groupId}`
      localStorage.removeItem(key)
      console.log('[PersistentLocalGroupChat] 群聊已从本地删除:', groupId)
    } catch (error) {
      console.error('[PersistentLocalGroupChat] 删除群聊失败:', error)
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
            groups.push(PersistentLocalGroupChat.fromJSON(json))
          }
        }
      }
      return groups.sort((a, b) => b.lastActivity - a.lastActivity)
    } catch (error) {
      console.error('[PersistentLocalGroupChat] 加载所有群聊失败:', error)
      return []
    }
  }
}

/**
 * 本地IPFS群聊管理器Hook
 */
export const useLocalIpfsGroupChatManager = ({
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

  // 本地IPFS群聊Hook
  const localIpfsGroupChat = useLocalIpfsGroupChat()

  // 更新引用
  useEffect(() => {
    openConversationPanelRef.current = openConversationPanel
  }, [openConversationPanel])

  // 加载本地持久化的群聊
  const loadPersistentGroups = useCallback(() => {
    try {
      const groups = PersistentLocalGroupChat.loadAll()
      console.log('[useLocalIpfsGroupChatManager] 加载本地群聊:', groups.length)
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
      console.error('[useLocalIpfsGroupChatManager] 加载持久化群聊失败:', error)
    }
  }, [])

  // 初始化时加载本地群聊
  useEffect(() => {
    loadPersistentGroups()
  }, [loadPersistentGroups])

  // 监听频道变化，加载对应群聊
  useEffect(() => {
    if (activeChannelId && loadedChannelRef.current !== activeChannelId) {
      loadedChannelRef.current = activeChannelId
      setIsLoading(true)

      // 加载频道的群聊（从本地存储）
      setTimeout(() => {
        const channelGroups = persistentGroups.filter(g => g.channelId === activeChannelId)
        console.log('[useLocalIpfsGroupChatManager] 频道群聊:', activeChannelId, channelGroups.length)

        // 如果频道有群聊，恢复第一个为活跃
        if (channelGroups.length > 0) {
          const firstGroup = channelGroups[0]
          setActiveGroupIdState(firstGroup.groupId)

          // 检查是否应该显示面板
          const savedShowPanel = localStorage.getItem(`${STORAGE_KEYS.SHOW_PANEL}_${activeChannelId}`)
          if (savedShowPanel === 'true') {
            setShowGroupChat(true)
            openConversationPanelRef.current?.()
          }
        } else {
          setShowGroupChat(false)
        }

        setIsLoading(false)
      }, 0)
    } else if (!activeChannelId) {
      loadedChannelRef.current = null
      setShowGroupChat(false)
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

      // 使用本地IPFS创建群聊
      const group = await localIpfsGroupChat.createGroup({
        groupName,
        description: `${channel.name} 的群聊`,
        members: agents.map(agent => agent.did || agent.ipns || agent.cid || agent.id),
        isPublic: false,
        metadata: {
          channelId: activeChannelId,
          channelName: channel.name,
          createdAt: Date.now()
        }
      })

      // 创建持久化对象
      const persistentGroup = new PersistentLocalGroupChat({
        groupId: group.groupId,
        groupName: group.groupName,
        description: group.description,
        topic: group.topic,
        channelId: activeChannelId,
        channelName: channel.name,
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
          type: 'ipfs_group_chat',
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

      setIsLoading(false)
      return group

    } catch (error) {
      setIsLoading(false)
      console.error('[useLocalIpfsGroupChatManager] 创建群聊失败:', error)
      throw error
    }
  }, [localIdentity, activeChannelId, getActions, localIpfsGroupChat])

  // 发送消息
  const sendMessage = useCallback(async (groupId, content) => {
    try {
      await localIpfsGroupChat.sendMessage(content)
    } catch (error) {
      console.error('[useLocalIpfsGroupChatManager] 发送消息失败:', error)
      throw error
    }
  }, [localIpfsGroupChat])

  // 切换群聊
  const switchGroupChat = useCallback((groupId) => {
    if (persistentGroups.some(g => g.groupId === groupId)) {
      setActiveGroupIdState(groupId)
      localStorage.setItem(STORAGE_KEYS.ACTIVE_GROUP, groupId)
      console.log('[useLocalIpfsGroupChatManager] 切换群聊:', groupId)
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
      PersistentLocalGroupChat.delete(groupId)

      // 更新状态
      setPersistentGroups(prev => prev.filter(g => g.groupId !== groupId))

      // 如果删除的是当前活跃群聊，清除活跃状态
      if (activeGroupId === groupId) {
        setActiveGroupIdState(null)
        localStorage.removeItem(STORAGE_KEYS.ACTIVE_GROUP)
        setShowGroupChat(false)
      }

      console.log('[useLocalIpfsGroupChatManager] 群聊已删除:', groupId)
    } catch (error) {
      console.error('[useLocalIpfsGroupChatManager] 删除群聊失败:', error)
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

    // 操作方法
    createGroupChat,
    sendMessage,
    switchGroupChat,
    openGroupChat,
    closeGroupChat,
    deleteGroupChat,

    // 工具方法
    refreshGroups: loadPersistentGroups,
    clearError: localIpfsGroupChat.clearError
  }
}

export default useLocalIpfsGroupChatManager