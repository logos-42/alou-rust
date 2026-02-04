import { create } from 'zustand'
import {
  setGroupChatData,
  getGroupChatData,
  setActiveGroupId,
  getActiveGroupId,
  getKeys,
  removeItem
} from '../utils/storageAdapter'

// Export the store type for use elsewhere
export type { ClusterActionStore } from './clusterActionStore.types'

/**
 * 集群行动状态管理 Store
 * 使用内存存储替代localStorage，解决空间限制问题
 */

// 智能体信息接口
export interface AgentInfo {
  id: string
  name: string
  avatar?: string
  avatar_url?: string
  mode?: string
  [key: string]: any
}

// 群聊元数据接口
export interface GroupChatMetadata {
  type?: string
  channel_id?: string
  channel_name?: string
  [key: string]: any
}

// 群聊行动接口
export interface GroupChatAction {
  action_id: string
  description?: string
  status?: string
  created_at?: string | number
  agents?: AgentInfo[]
  metadata?: GroupChatMetadata
  [key: string]: any
}

// 群聊消息接口
export interface GroupChatMessage {
  id: string
  content: string
  sender?: string
  timestamp?: number
  [key: string]: any
}

// 行动状态
export type ActionStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'

// 行动详情接口
export interface ActionDetails {
  [key: string]: any
}

// 存储的行动数据（优化后）
export interface StoredAction {
  action_id: string
  description?: string
  status?: string
  created_at?: string | number
  agents: Partial<AgentInfo>[]
  metadata: Partial<GroupChatMetadata>
}

// Store 状态接口
interface ClusterActionState {
  actionsByChannel: Record<string, GroupChatAction[]>
  activeActionIdByChannel: Record<string, string | null>
  groupChatMessages: Record<string, GroupChatMessage[]>
  actionStatuses: Record<string, ActionStatus>
  actionDetails: Record<string, ActionDetails>
}

// Store 动作接口
interface ClusterActionActions {
  getActions: (channelId: string) => GroupChatAction[]
  getActiveActionId: (channelId: string) => string | null
  loadChannelGroupChats: (channelId: string) => { actions: GroupChatAction[]; activeActionId: string | null }
  addAction: (action: GroupChatAction) => void
  updateAction: (actionId: string, updates: Partial<GroupChatAction>) => void
  setActiveAction: (actionId: string, channelId?: string) => void
  addGroupChatMessage: (actionId: string, message: GroupChatMessage) => void
  addGroupChatMessages: (actionId: string, newMessages: GroupChatMessage[]) => void
  updateActionStatus: (actionId: string, status: ActionStatus) => void
  setActionDetails: (actionId: string, details: ActionDetails) => void
  getActionDetails: (actionId: string) => ActionDetails | null
  getGroupChatMessages: (actionId: string) => GroupChatMessage[]
  getActionStatus: (actionId: string) => ActionStatus
  getActiveAction: (channelId: string) => GroupChatAction | null
  removeAction: (actionId: string) => void
  clearAll: () => void
}

type ClusterActionStore = ClusterActionState & ClusterActionActions

// 从内存存储加载群聊（按频道）
const loadGroupChatsFromStorage = (channelId: string): GroupChatAction[] => {
  if (typeof window === 'undefined') return []
  
  try {
    const result = getGroupChatData(channelId)
    if (result && result.length > 0) {
      console.log(`[clusterActionStore] 从内存加载群聊 (频道: ${channelId}):`, result.length, '个群聊', result.map((a: any) => a.action_id))
    }
    return result || []
  } catch (error) {
    console.error('[clusterActionStore] 加载群聊失败:', error)
  }
  return []
}

// 从内存存储加载活跃的行动ID（按频道）
const loadActiveActionIdFromStorage = (channelId: string): string | null => {
  if (typeof window === 'undefined') return null
  
  try {
    const stored = getActiveGroupId(channelId)
    if (stored) {
      console.log(`[clusterActionStore] 从内存加载活跃行动ID (频道: ${channelId}):`, stored)
    }
    return stored || null
  } catch (error) {
    console.error('[clusterActionStore] 加载活跃行动ID失败:', error)
  }
  return null
}

// 保存活跃行动ID到内存存储
const saveActiveActionIdToStorage = (actionId: string | null, channelId: string): void => {
  try {
    setActiveGroupId(channelId, actionId)
  } catch (error) {
    console.error('[clusterActionStore] 保存活跃行动ID失败:', error)
  }
}

// 保存群聊到内存存储（带存储优化）
const saveGroupChatsToStorage = (groupChats: GroupChatAction[], channelId: string): boolean => {
  try {
    // 存储优化：只保存必要的数据，减少存储占用
    const optimizedChats: StoredAction[] = groupChats.map(chat => ({
      action_id: chat.action_id,
      description: chat.description,
      status: chat.status,
      created_at: chat.created_at,
      // 只保留前3个智能体的信息，减少数据量
      agents: chat.agents ? chat.agents.slice(0, 3).map(agent => ({
        id: agent.id,
        name: agent.name,
        avatar: agent.avatar,
        mode: agent.mode
      })) : [],
      // 只保留metadata的关键信息
      metadata: chat.metadata ? {
        type: chat.metadata.type,
        channel_id: chat.metadata.channel_id,
        channel_name: chat.metadata.channel_name
      } : {}
    }))
    
    // 保存到内存存储
    const success = setGroupChatData(channelId, optimizedChats)
    
    if (success && optimizedChats.length > 0) {
      console.log(`[clusterActionStore] 保存群聊到内存存储 (频道: ${channelId}):`, optimizedChats.length, '个群聊（已优化）')
    }
    
    return success
  } catch (error) {
    console.error('[clusterActionStore] 保存群聊失败:', error)
    return false
  }
}

const useClusterActionStore = create<ClusterActionStore>((set, get) => ({
  // 初始状态
  actionsByChannel: {},
  activeActionIdByChannel: {},
  groupChatMessages: {},
  actionStatuses: {},
  actionDetails: {},

  // 获取指定频道的 actions
  getActions: (channelId: string): GroupChatAction[] => {
    return get().actionsByChannel[channelId] || []
  },

  // 获取指定频道的 activeActionId
  getActiveActionId: (channelId: string): string | null => {
    return get().activeActionIdByChannel[channelId] || null
  },

  // 加载指定频道的群聊
  loadChannelGroupChats: (channelId: string) => {
    if (!channelId) {
      console.warn('[clusterActionStore] loadChannelGroupChats: channelId 为空')
      return { actions: [], activeActionId: null }
    }
    
    const actions = loadGroupChatsFromStorage(channelId)
    const activeActionId = loadActiveActionIdFromStorage(channelId)
    const validatedId = activeActionId && actions.some((a: GroupChatAction) => a.action_id === activeActionId)
      ? activeActionId
      : (actions.length > 0 ? actions[0].action_id : null)
    
    console.log(`[clusterActionStore] 加载频道群聊 (频道: ${channelId}):`, {
      actionsCount: actions.length,
      loadedActiveId: activeActionId,
      validatedId: validatedId,
      actionIds: actions.map((a: GroupChatAction) => a.action_id)
    })
    
    set((state: ClusterActionState) => ({
      actionsByChannel: {
        ...state.actionsByChannel,
        [channelId]: actions,
      },
      activeActionIdByChannel: {
        ...state.activeActionIdByChannel,
        [channelId]: validatedId,
      },
    }))
    
    if (validatedId && validatedId !== activeActionId) {
      saveActiveActionIdToStorage(validatedId, channelId)
    }
    
    return { actions, activeActionId: validatedId }
  },

  // 添加集群行动
  addAction: (action: GroupChatAction) => {
    set((state: ClusterActionState) => {
      // 优先使用 action.metadata.channel_id
      let channelId = action.metadata?.channel_id
      
      // 如果没有 channel_id，尝试从其他地方获取（向后兼容）
      if (!channelId) {
        // 尝试从所有频道中查找是否已存在该 action
        for (const [chId, actions] of Object.entries(state.actionsByChannel)) {
          if (actions.some((a: GroupChatAction) => a.action_id === action.action_id)) {
            channelId = chId
            break
          }
        }
      }
      
      // 如果仍然没有 channelId，使用 'global' 作为后备
      if (!channelId) {
        channelId = 'global'
      }
      
      // 如果是群聊类型但没有有效的 channelId，记录警告但不阻止保存
      if (channelId === 'global' && (action.metadata?.type === 'group_chat' || action.action_id?.startsWith('local_group_'))) {
        console.warn('[clusterActionStore] addAction: 群聊缺少 channel_id，使用 global:', action.action_id, action.metadata)
      }
      
      const channelActions = state.actionsByChannel[channelId] || []
      const existingIndex = channelActions.findIndex((a: GroupChatAction) => a.action_id === action.action_id)
      
      let newActions: GroupChatAction[]
      if (existingIndex >= 0) {
        newActions = [...channelActions]
        newActions[existingIndex] = action
        console.log(`[clusterActionStore] 更新群聊 - 设置action:`, action)
        console.log(`[clusterActionStore] 更新群聊 - action.agents:`, action.agents)
        console.log(`[clusterActionStore] 更新群聊 - 设置后newActions[existingIndex]:`, newActions[existingIndex])
        console.log(`[clusterActionStore] 更新群聊开始:`, newActions)
      } else {
        newActions = [...channelActions, action]
        console.log(`[clusterActionStore] 添加群聊 - 设置action:`, action)
        console.log(`[clusterActionStore] 添加群聊 - action.agents:`, action.agents)
      }
      
      // 保存群聊到 localStorage
      saveGroupChatsToStorage(newActions, channelId)
      
      // 如果当前频道没有 activeActionId，自动设置为新添加的 action
      const currentActiveId = state.activeActionIdByChannel[channelId]
      const updatedActiveActionIdByChannel = { ...state.activeActionIdByChannel }
      if (!currentActiveId && (action.metadata?.type === 'group_chat' || action.action_id?.startsWith('local_group_'))) {
        updatedActiveActionIdByChannel[channelId] = action.action_id
        saveActiveActionIdToStorage(action.action_id, channelId)
        console.log(`[clusterActionStore] 自动设置 activeActionId: ${action.action_id} (频道: ${channelId})`)
      }
      
      return {
        actionsByChannel: {
          ...state.actionsByChannel,
          [channelId]: newActions,
        },
        activeActionIdByChannel: updatedActiveActionIdByChannel,
      }
    })
  },

  // 更新集群行动
  updateAction: (actionId: string, updates: Partial<GroupChatAction>) => {
    set((state: ClusterActionState) => {
      const updatedActionsByChannel: Record<string, GroupChatAction[]> = {}
      
      // 找到 action 所在的频道
      for (const [chId, actions] of Object.entries(state.actionsByChannel)) {
        const action = actions.find((a: GroupChatAction) => a.action_id === actionId)
        if (action) {
          updatedActionsByChannel[chId] = actions.map((a: GroupChatAction) =>
            a.action_id === actionId ? { ...a, ...updates } : a
          )
          // 保存群聊到 localStorage
          saveGroupChatsToStorage(updatedActionsByChannel[chId], chId)
          break
        }
      }
      
      return {
        actionsByChannel: {
          ...state.actionsByChannel,
          ...updatedActionsByChannel,
        },
      }
    })
  },

  // 设置当前选中的行动（按频道）
  setActiveAction: (actionId: string, channelId?: string) => {
    let targetChannelId = channelId
    if (!targetChannelId) {
      // 如果没有指定频道，尝试从 action 中查找
      const allActions = Object.values(get().actionsByChannel).flat()
      const action = allActions.find((a: GroupChatAction) => a.action_id === actionId)
      targetChannelId = action?.metadata?.channel_id || 'global'
    }
    
    set((state: ClusterActionState) => ({
      activeActionIdByChannel: {
        ...state.activeActionIdByChannel,
        [targetChannelId!]: actionId,
      },
    }))
    
    // 保存到 localStorage
    saveActiveActionIdToStorage(actionId, targetChannelId!)
  },

  // 添加群聊消息
  addGroupChatMessage: (actionId: string, message: GroupChatMessage) => {
    set((state: ClusterActionState) => {
      const messages = state.groupChatMessages[actionId] || []
      // 检查消息是否已存在（避免重复）
      const exists = messages.some((m: GroupChatMessage) => m.id === message.id)
      if (exists) {
        return state
      }
      return {
        groupChatMessages: {
          ...state.groupChatMessages,
          [actionId]: [...messages, message],
        },
      }
    })
  },

  // 批量添加群聊消息
  addGroupChatMessages: (actionId: string, newMessages: GroupChatMessage[]) => {
    set((state: ClusterActionState) => {
      const existingMessages = state.groupChatMessages[actionId] || []
      const existingIds = new Set(existingMessages.map((m: GroupChatMessage) => m.id))
      const uniqueMessages = newMessages.filter((m: GroupChatMessage) => !existingIds.has(m.id))
      return {
        groupChatMessages: {
          ...state.groupChatMessages,
          [actionId]: [...existingMessages, ...uniqueMessages],
        },
      }
    })
  },

  // 更新行动状态
  updateActionStatus: (actionId: string, status: ActionStatus) => {
    set((state: ClusterActionState) => ({
      actionStatuses: {
        ...state.actionStatuses,
        [actionId]: status,
      },
    }))
  },

  // 设置行动详情
  setActionDetails: (actionId: string, details: ActionDetails) => {
    set((state: ClusterActionState) => ({
      actionDetails: {
        ...state.actionDetails,
        [actionId]: details,
      },
    }))
  },

  // 获取行动详情
  getActionDetails: (actionId: string): ActionDetails | null => {
    return get().actionDetails[actionId] || null
  },

  // 获取群聊消息
  getGroupChatMessages: (actionId: string): GroupChatMessage[] => {
    return get().groupChatMessages[actionId] || []
  },

  // 获取行动状态
  getActionStatus: (actionId: string): ActionStatus => {
    return get().actionStatuses[actionId] || 'Pending'
  },

  // 获取当前活跃的行动（按频道）
  getActiveAction: (channelId: string): GroupChatAction | null => {
    const activeActionId = get().activeActionIdByChannel[channelId]
    console.log(`[clusterActionStore] getActiveAction - activeActionId: ${activeActionId}, channelId: ${channelId}`)
    if (!activeActionId) {
      console.log('[clusterActionStore] getActiveAction - 没有activeActionId，返回null')
      return null
    }
    const actions = get().actionsByChannel[channelId] || []
    console.log(`[clusterActionStore] getActiveAction - 该频道的actions:`, actions)
    console.log(`[clusterActionStore] getActiveAction - actions数量: ${actions.length}`)
    
    const foundAction = actions.find((a: GroupChatAction) => a.action_id === activeActionId)
    console.log(`[clusterActionStore] getActiveAction - 查找结果:`, foundAction)
    
    if (foundAction && foundAction.agents) {
      console.log(`[clusterActionStore] getActiveAction - 找到的agents数量: ${foundAction.agents.length}`)
      foundAction.agents.forEach((agent: AgentInfo, index: number) => {
        console.log(`[clusterActionStore] getActiveAction - agent ${index + 1}:`, {
          id: agent.id,
          name: agent.name,
          hasAvatar: !!(agent.avatar || agent.avatar_url)
        })
      })
    } else if (foundAction) {
      console.log('[clusterActionStore] getActiveAction - 找到的action没有agents字段')
    } else {
      console.log(`[clusterActionStore] getActiveAction - 未找到action_id为 ${activeActionId} 的action`)
    }
    
    return foundAction || null
  },

  // 清除行动（完成或取消后）
  removeAction: (actionId: string) => {
    set((state: ClusterActionState) => {
      const updatedActionsByChannel: Record<string, GroupChatAction[]> = {}
      const updatedActiveActionIdByChannel: Record<string, string | null> = {}
      
      // 找到 action 所在的频道并删除
      for (const [channelId, actions] of Object.entries(state.actionsByChannel)) {
        const filtered = actions.filter((a: GroupChatAction) => a.action_id !== actionId)
        if (filtered.length !== actions.length) {
          updatedActionsByChannel[channelId] = filtered
          // 更新 activeActionId
          const currentActiveId = state.activeActionIdByChannel[channelId]
          updatedActiveActionIdByChannel[channelId] =
            currentActiveId === actionId
              ? (filtered.length > 0 ? filtered[0].action_id : null)
              : currentActiveId
          // 保存群聊到 localStorage
          saveGroupChatsToStorage(filtered, channelId)
          // 保存 activeActionId 到 localStorage
          saveActiveActionIdToStorage(updatedActiveActionIdByChannel[channelId], channelId)
        }
      }
      
      const groupChatMessages = { ...state.groupChatMessages }
      delete groupChatMessages[actionId]
      const actionStatuses = { ...state.actionStatuses }
      delete actionStatuses[actionId]
      const actionDetails = { ...state.actionDetails }
      delete actionDetails[actionId]
      
      return {
        actionsByChannel: {
          ...state.actionsByChannel,
          ...updatedActionsByChannel,
        },
        activeActionIdByChannel: {
          ...state.activeActionIdByChannel,
          ...updatedActiveActionIdByChannel,
        },
        groupChatMessages,
        actionStatuses,
        actionDetails,
      }
    })
    },

  // 获取当前状态（供外部使用）
  getState: (): ClusterActionState => {
    return get()
  },

  // 清除所有数据
  clearAll: () => {
    set({
      actionsByChannel: {},
      activeActionIdByChannel: {},
      groupChatMessages: {},
      actionStatuses: {},
      actionDetails: {},
    })
    // 清除内存存储中的所有群聊数据
    try {
      const keys = getKeys()
      const keysToRemove = keys.filter((key: string) => 
        key.startsWith('cluster_actions_group_chats') || 
        key.startsWith('cluster_actions_active_id')
      )
      
      keysToRemove.forEach((key: string) => {
        removeItem(key)
      })
      
      console.log(`[clusterActionStore] 清除了 ${keysToRemove.length} 个群聊存储项`)
    } catch (error) {
      console.error('[clusterActionStore] 清除群聊失败:', error)
    }
  },
}))

export default useClusterActionStore
