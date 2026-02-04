/**
 * ClusterActionStore types
 * Extracted for type exports
 */

export interface ClusterActionStore {
  (state: ClusterActionState): ClusterActionState
  actions: ClusterActionActions
}

export interface ClusterActionState {
  actionsByChannel: Record<string, GroupChatAction[]>
  activeActionIdByChannel: Record<string, string | null>
  groupChatMessages: Record<string, GroupChatMessage[]>
  actionStatuses: Record<string, ActionStatus>
  actionDetails: Record<string, ActionDetails>
}

export interface ClusterActionActions {
  addAction: (action: GroupChatAction, channelId?: string) => void
  removeAction: (actionId: string, channelId?: string) => void
  setActiveAction: (actionId: string, channelId?: string) => void
  getActiveAction: (channelId: string) => GroupChatAction | null
  addGroupChatMessage: (message: GroupChatMessage, actionId: string) => void
  setGroupChatMessages: (actionId: string, messages: GroupChatMessage[]) => void
  getState: () => ClusterActionState
}

// Re-export types from the main store file
export type { GroupChatAction, GroupChatMessage, ActionStatus, ActionDetails } from './clusterActionStore'

