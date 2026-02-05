/**
 * ClusterActionStore types
 * Extracted for type exports
 */

// 智能体信息接口
export interface AgentInfo {
  id: string
  name: string
  avatar?: string
  avatar_url?: string
  mode?: string
  [key: string]: unknown
}

// 群聊元数据接口
export interface GroupChatMetadata {
  type?: string
  channel_id?: string
  channel_name?: string
  [key: string]: unknown
}

// 群聊行动接口
export interface GroupChatAction {
  action_id: string
  description?: string
  status?: string
  created_at?: string | number
  agents?: AgentInfo[]
  metadata?: GroupChatMetadata
  [key: string]: unknown
}

// 群聊消息接口
export interface GroupChatMessage {
  id: string
  content: string
  sender?: string
  timestamp?: number
  [key: string]: unknown
}

// 行动状态
export type ActionStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'

// 行动详情接口
export interface ActionDetails {
  [key: string]: unknown
}

// Zustand Store 类型
export type ClusterActionStore = ClusterActionState & ClusterActionActions

export interface ClusterActionState {
  actionsByChannel: Record<string, GroupChatAction[]>
  activeActionIdByChannel: Record<string, string | null>
  groupChatMessages: Record<string, GroupChatMessage[]>
  actionStatuses: Record<string, ActionStatus>
  actionDetails: Record<string, ActionDetails>
}

export interface ClusterActionActions {
  addAction: (action: GroupChatAction) => void
  removeAction: (actionId: string) => void
  setActiveAction: (actionId: string, channelId?: string) => void
  getActiveAction: (channelId: string) => GroupChatAction | null
  addGroupChatMessage: (actionId: string, message: GroupChatMessage) => void
  addGroupChatMessages: (actionId: string, messages: GroupChatMessage[]) => void
  getGroupChatMessages: (actionId: string) => GroupChatMessage[]
  getState: () => ClusterActionState
}

