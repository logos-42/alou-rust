/**
 * 群聊类型定义
 * 
 * 此文件作为所有群聊相关类型的单一真实来源（Single Source of Truth）
 * 所有群聊相关的组件、hooks、services 都应该从这里导入类型
 * 
 * @module types/groupchat
 * @author Alou Team
 * @since 1.0.0
 */

// ============================================
// 基础类型定义
// ============================================

/**
 * 消息类型枚举
 * 定义群聊中支持的所有消息类型
 */
export enum MessageType {
  /** 普通聊天消息 */
  CHAT = 'chat',
  /** 系统消息（如群聊创建、用户加入/离开） */
  SYSTEM = 'system',
  /** 加入群聊 */
  JOIN = 'join',
  /** 离开群聊 */
  LEAVE = 'leave',
  /** 智能体请求 */
  AGENT_REQUEST = 'agent_request',
  /** 智能体响应 */
  AGENT_RESPONSE = 'agent_response',
  /** 任务分配 */
  TASK_ASSIGN = 'task_assign',
  /** 任务进度 */
  TASK_PROGRESS = 'task_progress',
  /** 任务完成 */
  TASK_COMPLETE = 'task_complete',
  /** 错误消息 */
  ERROR = 'error',
  /** 欢迎消息 */
  WELCOME = 'welcome',
}

/**
 * 群聊状态枚举
 */
export enum GroupChatStatus {
  /** 空闲状态 */
  IDLE = 'idle',
  /** 创建/加入中 */
  JOINING = 'joining',
  /** 活跃状态 */
  ACTIVE = 'active',
  /** 离开中 */
  LEAVING = 'leaving',
  /** 错误状态 */
  ERROR = 'error',
  /** 已关闭 */
  CLOSED = 'closed',
}

/**
 * 行动状态类型
 * 用于集群行动的状态追踪
 */
export type ActionStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled' | 'Active'

// ============================================
// 实体接口定义
// ============================================

/**
 * 智能体信息接口
 * 用于描述群聊中的智能体成员
 */
export interface AgentInfo {
  /** 智能体唯一标识 */
  id: string
  /** 智能体显示名称 */
  name: string
  /** 智能体头像URL */
  avatar?: string
  /** 智能体头像URL（别名） */
  avatar_url?: string
  /** 智能体头像CID（IPFS内容标识） */
  avatar_cid?: string
  /** 智能体DID（去中心化身份） */
  did?: string
  /** IPNS名称 */
  ipns?: string
  /** CID标识 */
  cid?: string
  /** 智能体模式：agent-智能体，user-用户 */
  mode?: 'agent' | 'user' | string
  /** 额外元数据 */
  [key: string]: unknown
}

/**
 * 本地身份信息接口
 */
export interface LocalIdentity {
  /** 去中心化身份标识 */
  did: string
  /** 显示名称 */
  name?: string
  /** 头像URL */
  avatar?: string
  /** 额外元数据 */
  [key: string]: unknown
}

/**
 * 群聊元数据接口
 */
export interface GroupChatMetadata {
  /** 群聊类型 */
  type?: 'group_chat' | 'diap_group_chat' | 'local_group' | string
  /** 关联频道ID */
  channel_id?: string
  /** 关联频道名称 */
  channel_name?: string
  /** DIAP群组ID */
  diap_group_id?: string
  /** DIAP主题 */
  diap_topic?: string
  /** 是否本地群聊 */
  local?: boolean
  /** 创建时间 */
  createdAt?: number
  /** 额外元数据 */
  [key: string]: unknown
}

/**
 * 群聊消息接口
 * 统一的消息结构，用于所有群聊场景
 */
export interface GroupChatMessage {
  /** 消息唯一标识 */
  id: string
  /** 消息类型 */
  type: MessageType | string
  /** 发送者ID */
  from: string
  /** 发送者显示名称 */
  fromName: string
  /** 发送者头像 */
  avatar?: string | null
  /** 消息内容 */
  content: string
  /** 时间戳（毫秒） */
  timestamp: number
  /** 消息元数据 */
  metadata: {
    /** 是否本地发送 */
    isLocal?: boolean
    /** 原始发送者 */
    originalFrom?: string
    /** 头像来源 */
    avatarSource?: string
    /** 消息状态 */
    status?: 'pending' | 'sent' | 'delivered' | 'failed'
    /** 错误信息 */
    error?: string
    [key: string]: unknown
  }
  /** 主题（用于PubSub） */
  topic?: string
  /** 接收者（用于私聊，null表示群聊广播） */
  to?: string | null
}

/**
 * PubSub消息数据接口
 * 用于与后端PubSub服务交互
 */
export interface PubSubMessageData {
  id?: string
  type: MessageType | string
  from?: string
  to?: string | null
  content: string
  topic: string
  timestamp?: number
  metadata?: Record<string, unknown>
}

/**
 * 群聊接口
 * 描述一个群聊的基本信息
 */
export interface GroupChat {
  /** 群聊唯一标识 */
  groupId: string
  /** 行动ID（兼容旧版本） */
  action_id?: string
  /** 群聊主题（用于PubSub） */
  topic: string
  /** 群聊名称 */
  groupName: string
  /** 群聊描述 */
  description?: string
  /** 群聊成员 */
  members?: string[]
  /** 群聊中的智能体 */
  agents?: AgentInfo[]
  /** 创建时间 */
  created_at?: string | number
  /** 群聊状态 */
  status?: GroupChatStatus | ActionStatus | string
  /** 群聊元数据 */
  metadata?: GroupChatMetadata
  /** 额外字段 */
  [key: string]: unknown
}

/**
 * 群聊行动接口
 * 用于集群行动场景
 */
export interface GroupChatAction {
  /** 行动唯一标识 */
  action_id: string
  /** 行动描述 */
  description?: string
  /** 行动状态 */
  status?: ActionStatus | string
  /** 创建时间 */
  created_at?: string | number
  /** 参与的智能体 */
  agents?: AgentInfo[]
  /** 行动元数据 */
  metadata?: GroupChatMetadata
  /** 额外字段 */
  [key: string]: unknown
}

// ============================================
// 错误类型定义
// ============================================

/**
 * 群聊错误代码枚举
 */
export enum GroupChatErrorCode {
  /** 未知错误 */
  UNKNOWN = 'UNKNOWN_ERROR',
  /** 网络错误 */
  NETWORK = 'NETWORK_ERROR',
  /** 身份验证失败 */
  AUTHENTICATION = 'AUTHENTICATION_ERROR',
  /** 权限不足 */
  AUTHORIZATION = 'AUTHORIZATION_ERROR',
  /** 群聊不存在 */
  GROUP_NOT_FOUND = 'GROUP_NOT_FOUND',
  /** 消息发送失败 */
  MESSAGE_SEND_FAILED = 'MESSAGE_SEND_FAILED',
  /** IPFS不可用 */
  IPFS_UNAVAILABLE = 'IPFS_UNAVAILABLE',
  /** 服务未初始化 */
  NOT_INITIALIZED = 'NOT_INITIALIZED',
  /** 参数无效 */
  INVALID_PARAMS = 'INVALID_PARAMS',
  /** 存储错误 */
  STORAGE = 'STORAGE_ERROR',
  /** 超时 */
  TIMEOUT = 'TIMEOUT_ERROR',
}

/**
 * 群聊错误接口
 */
export interface GroupChatError {
  /** 错误代码 */
  code: GroupChatErrorCode | string
  /** 错误消息 */
  message: string
  /** 原始错误 */
  originalError?: Error | unknown
  /** 错误上下文 */
  context?: Record<string, unknown>
  /** 时间戳 */
  timestamp: number
}

/**
 * 群聊错误类
 * 用于抛出标准化的群聊错误
 */
export class GroupChatError extends Error {
  code: GroupChatErrorCode | string
  originalError?: Error | unknown
  context?: Record<string, unknown>
  timestamp: number

  constructor(
    code: GroupChatErrorCode | string,
    message: string,
    originalError?: Error | unknown,
    context?: Record<string, unknown>
  ) {
    super(message)
    this.name = 'GroupChatError'
    this.code = code
    this.originalError = originalError
    this.context = context
    this.timestamp = Date.now()
  }

  toJSON(): GroupChatError {
    return {
      code: this.code,
      message: this.message,
      originalError: this.originalError,
      context: this.context,
      timestamp: this.timestamp,
    }
  }
}

// ============================================
// Store 相关类型
// ============================================

/**
 * 群聊状态（用于Zustand Store）
 */
export interface GroupChatState {
  /** 按频道组织的群聊列表 */
  actionsByChannel: Record<string, GroupChatAction[]>
  /** 按频道组织的活跃行动ID */
  activeActionIdByChannel: Record<string, string | null>
  /** 群聊消息映射 */
  groupChatMessages: Record<string, GroupChatMessage[]>
  /** 行动状态映射 */
  actionStatuses: Record<string, ActionStatus>
  /** 行动详情映射 */
  actionDetails: Record<string, Record<string, unknown>>
}

/**
 * 群聊 Store Actions
 */
export interface GroupChatActions {
  /** 获取指定频道的群聊列表 */
  getActions: (channelId: string) => GroupChatAction[]
  /** 获取指定频道的活跃行动ID */
  getActiveActionId: (channelId: string) => string | null
  /** 加载频道的群聊 */
  loadChannelGroupChats: (channelId: string) => { actions: GroupChatAction[]; activeActionId: string | null }
  /** 添加群聊行动 */
  addAction: (action: GroupChatAction) => void
  /** 更新群聊行动 */
  updateAction: (actionId: string, updates: Partial<GroupChatAction>) => void
  /** 设置活跃行动 */
  setActiveAction: (actionId: string, channelId?: string) => void
  /** 获取活跃行动 */
  getActiveAction: (channelId: string) => GroupChatAction | null
  /** 添加单条消息 */
  addGroupChatMessage: (actionId: string, message: GroupChatMessage) => void
  /** 批量添加消息 */
  addGroupChatMessages: (actionId: string, messages: GroupChatMessage[]) => void
  /** 获取群聊消息 */
  getGroupChatMessages: (actionId: string) => GroupChatMessage[]
  /** 更新行动状态 */
  updateActionStatus: (actionId: string, status: ActionStatus) => void
  /** 获取行动状态 */
  getActionStatus: (actionId: string) => ActionStatus
  /** 设置行动详情 */
  setActionDetails: (actionId: string, details: Record<string, unknown>) => void
  /** 获取行动详情 */
  getActionDetails: (actionId: string) => Record<string, unknown> | null
  /** 移除行动 */
  removeAction: (actionId: string) => void
  /** 清除所有数据 */
  clearAll: () => void
  /** 获取当前状态 */
  getState: () => GroupChatState
}

/** 群聊 Store 完整类型 */
export type GroupChatStore = GroupChatState & GroupChatActions

// ============================================
// Hook 相关类型
// ============================================

/**
 * useGroupChat Hook 选项
 */
export interface UseGroupChatOptions {
  /** 行动ID */
  actionId: string | null
  /** 是否启用 */
  enabled?: boolean
}

/**
 * useGroupChat Hook 返回值
 */
export interface UseGroupChatReturn {
  /** 当前消息列表 */
  messages: GroupChatMessage[]
  /** 当前状态 */
  status: ActionStatus | string | null
  /** 订阅群聊 */
  subscribeToGroupChat: (actionId: string, onMessage?: (msg: GroupChatMessage) => void) => Promise<() => void>
  /** 加载历史消息 */
  loadGroupMessages: (actionId: string) => Promise<void>
  /** 轮询行动状态 */
  pollActionStatus: (actionId: string) => Promise<() => void>
}

/**
 * useGroupChatManager Hook 选项
 */
export interface UseGroupChatManagerOptions {
  /** 打开对话面板的回调 */
  openConversationPanel?: () => void
  /** 当前频道ID */
  activeChannelId?: string | null
  /** 本地身份信息 */
  localIdentity?: LocalIdentity | null
}

/**
 * useGroupChatManager Hook 返回值
 */
export interface UseGroupChatManagerReturn {
  // 状态
  showGroupChat: boolean
  activeGroupId: string | null
  activeActionId: string | null
  activeAction: GroupChatAction | null
  activeGroup: GroupChat | null
  activeGroupMessages: GroupChatMessage[]
  groupChatList: GroupChatAction[]
  isLoading: boolean
  actionStatus: ActionStatus | string | null
  splitPosition: number
  hasActiveAction: boolean
  canOpenGroupChat: boolean

  // 操作方法
  createGroupChat: (groupName: string, agents?: AgentInfo[]) => Promise<GroupChat>
  sendMessage: (groupId: string, content: string) => Promise<void>
  switchGroupChat: (groupId: string) => Promise<void>
  openGroupChat: () => Promise<void>
  closeGroupChat: () => void
  closeGroupChatCompletely: () => void
  toggleGroupChat: () => void
  deleteGroupChat: (groupId: string) => Promise<void>
  setSplitPosition: (position: number) => void

  // 兼容接口
  getActions: (channelId: string) => GroupChatAction[]
  getActiveActionId: () => string | null
  setActiveAction: (actionId: string, channelId?: string) => void
  getActiveAction: (channelId: string) => GroupChatAction | null
  getGroupChatMessages: (actionId: string) => GroupChatMessage[]
  getActionStatus: (actionId: string) => ActionStatus
}

// ============================================
// 工具类型
// ============================================

/** 消息处理器类型 */
export type MessageHandler = (message: GroupChatMessage) => void

/** 取消订阅函数类型 */
export type UnsubscribeFunction = () => void

/** 群聊消息映射类型 */
export type GroupMessagesMap = Record<string, GroupChatMessage[]>

/** 群聊列表项（简化版） */
export interface GroupChatListItem {
  action_id: string
  description?: string
  agents?: AgentInfo[]
  metadata?: GroupChatMetadata
}

// ============================================
// 类型守卫函数
// ============================================

/**
 * 检查值是否为 GroupChatMessage
 */
export function isGroupChatMessage(value: unknown): value is GroupChatMessage {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'type' in value &&
    'from' in value &&
    'content' in value &&
    'timestamp' in value
  )
}

/**
 * 检查值是否为 GroupChatAction
 */
export function isGroupChatAction(value: unknown): value is GroupChatAction {
  return (
    typeof value === 'object' &&
    value !== null &&
    'action_id' in value
  )
}

/**
 * 检查值是否为 AgentInfo
 */
export function isAgentInfo(value: unknown): value is AgentInfo {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'name' in value
  )
}

// ============================================
// 重新导出兼容类型
// ============================================

/** @deprecated 使用 GroupChat 替代 */
export type Group = GroupChat

/** @deprecated 使用 MessageType 替代 */
export { MessageType as PubSubMessageType }

/** @deprecated 使用 LocalIdentity 替代 */
export type Identity = LocalIdentity
