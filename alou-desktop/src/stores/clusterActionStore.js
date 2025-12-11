import { create } from 'zustand'

/**
 * 集群行动状态管理 Store
 * 使用 Zustand 管理集群行动状态、消息和UI状态
 */
const useClusterActionStore = create((set, get) => ({
  // 集群行动列表
  actions: [],

  // 当前选中的行动ID
  activeActionId: null,

  // 群聊消息 { actionId: [messages] }
  groupChatMessages: {},

  // 行动状态 { actionId: status }
  actionStatuses: {},

  // 行动详情 { actionId: action }
  actionDetails: {},

  // 添加集群行动
  addAction: (action) => {
    set((state) => {
      const existingIndex = state.actions.findIndex((a) => a.action_id === action.action_id)
      if (existingIndex >= 0) {
        // 更新现有行动
        const updated = [...state.actions]
        updated[existingIndex] = action
        return { actions: updated }
      }
      // 添加新行动
      return { actions: [...state.actions, action] }
    })
  },

  // 更新集群行动
  updateAction: (actionId, updates) => {
    set((state) => {
      const actions = state.actions.map((a) =>
        a.action_id === actionId ? { ...a, ...updates } : a
      )
      return { actions }
    })
  },

  // 设置当前选中的行动
  setActiveAction: (actionId) => {
    set({ activeActionId: actionId })
  },

  // 添加群聊消息
  addGroupChatMessage: (actionId, message) => {
    set((state) => {
      const messages = state.groupChatMessages[actionId] || []
      // 检查消息是否已存在（避免重复）
      const exists = messages.some((m) => m.id === message.id)
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
  addGroupChatMessages: (actionId, newMessages) => {
    set((state) => {
      const existingMessages = state.groupChatMessages[actionId] || []
      const existingIds = new Set(existingMessages.map((m) => m.id))
      const uniqueMessages = newMessages.filter((m) => !existingIds.has(m.id))
      return {
        groupChatMessages: {
          ...state.groupChatMessages,
          [actionId]: [...existingMessages, ...uniqueMessages],
        },
      }
    })
  },

  // 更新行动状态
  updateActionStatus: (actionId, status) => {
    set((state) => ({
      actionStatuses: {
        ...state.actionStatuses,
        [actionId]: status,
      },
    }))
  },

  // 设置行动详情
  setActionDetails: (actionId, details) => {
    set((state) => ({
      actionDetails: {
        ...state.actionDetails,
        [actionId]: details,
      },
    }))
  },

  // 获取行动详情
  getActionDetails: (actionId) => {
    return get().actionDetails[actionId] || null
  },

  // 获取群聊消息
  getGroupChatMessages: (actionId) => {
    return get().groupChatMessages[actionId] || []
  },

  // 获取行动状态
  getActionStatus: (actionId) => {
    return get().actionStatuses[actionId] || 'Pending'
  },

  // 获取当前活跃的行动
  getActiveAction: () => {
    const { activeActionId, actions } = get()
    if (!activeActionId) return null
    return actions.find((a) => a.action_id === activeActionId) || null
  },

  // 清除行动（完成或取消后）
  removeAction: (actionId) => {
    set((state) => {
      const actions = state.actions.filter((a) => a.action_id !== actionId)
      const groupChatMessages = { ...state.groupChatMessages }
      delete groupChatMessages[actionId]
      const actionStatuses = { ...state.actionStatuses }
      delete actionStatuses[actionId]
      const actionDetails = { ...state.actionDetails }
      delete actionDetails[actionId]
      const activeActionId =
        state.activeActionId === actionId ? (actions.length > 0 ? actions[0].action_id : null) : state.activeActionId
      return {
        actions,
        groupChatMessages,
        actionStatuses,
        actionDetails,
        activeActionId,
      }
    })
  },

  // 清除所有数据
  clearAll: () => {
    set({
      actions: [],
      activeActionId: null,
      groupChatMessages: {},
      actionStatuses: {},
      actionDetails: {},
    })
  },
}))

export default useClusterActionStore

