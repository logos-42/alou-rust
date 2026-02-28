/**
 * 消息处理工具
 * 
 * 提供统一的消息处理、转换和验证函数
 * 
 * @module utils/groupchat/messageHelper
 * @author Alou Team
 * @since 1.0.0
 */

import {
  MessageType,
  GroupChatMessage,
  PubSubMessageData,
  AgentInfo,
  LocalIdentity,
  GroupChatError,
  GroupChatErrorCode,
} from '@/types/groupchat'
import { generateMessageId, generateWelcomeMessageId, generateSystemMessageId } from './idGenerator'
import { i18n } from '@/hooks/useI18n'

// ============================================
// 消息创建函数
// ============================================

/**
 * 创建消息的配置选项
 */
export interface CreateMessageOptions {
  /** 消息类型 */
  type?: MessageType | string
  /** 发送者ID */
  from: string
  /** 发送者名称 */
  fromName?: string
  /** 发送者头像 */
  avatar?: string | null
  /** 消息内容 */
  content: string
  /** 时间戳（可选，默认当前时间） */
  timestamp?: number
  /** 消息元数据 */
  metadata?: Record<string, unknown>
  /** 主题（用于PubSub） */
  topic?: string
  /** 接收者（私聊用） */
  to?: string | null
  /** 群聊ID（用于生成消息ID） */
  groupId?: string
  /** 是否本地发送 */
  isLocal?: boolean
}

/**
 * 创建标准群聊消息
 * @param options - 消息配置选项
 * @returns 标准化的群聊消息
 * 
 * @example
 * ```typescript
 * const message = createMessage({
 *   from: 'user123',
 *   fromName: '张三',
 *   content: '大家好！',
 *   groupId: 'group456'
 * })
 * ```
 */
export function createMessage(options: CreateMessageOptions): GroupChatMessage {
  const {
    type = MessageType.CHAT,
    from,
    fromName = from,
    avatar = null,
    content,
    timestamp = Date.now(),
    metadata = {},
    topic,
    to = null,
    groupId,
    isLocal = false,
  } = options

  return {
    id: generateMessageId(groupId),
    type,
    from,
    fromName,
    avatar,
    content,
    timestamp,
    metadata: {
      ...metadata,
      isLocal,
    },
    topic,
    to,
  }
}

/**
 * 创建系统消息
 * @param content - 消息内容
 * @param groupId - 群聊ID
 * @param metadata - 额外元数据
 * @returns 系统消息
 */
export function createSystemMessage(
  content: string,
  groupId?: string,
  metadata: Record<string, unknown> = {}
): GroupChatMessage {
  return createMessage({
    type: MessageType.SYSTEM,
    from: 'system',
    fromName: i18n?.t?.('agent.groupChat.system') || '系统',
    content,
    groupId,
    metadata: {
      ...metadata,
      isSystem: true,
    },
  })
}

/**
 * 创建欢迎消息
 * @param groupId - 群聊ID
 * @param description - 群聊描述
 * @returns 欢迎消息
 */
export function createWelcomeMessage(groupId: string, description?: string): GroupChatMessage {
  const welcomeContent = i18n?.t?.('agent.groupChat.created', { description }) ||
    `群聊 "${description || groupId}" 已创建`

  return {
    id: generateWelcomeMessageId(groupId),
    type: MessageType.WELCOME,
    from: 'system',
    fromName: i18n?.t?.('agent.groupChat.system') || '系统',
    avatar: null,
    content: welcomeContent,
    timestamp: Date.now(),
    metadata: {
      isSystem: true,
      isWelcome: true,
    },
  }
}

/**
 * 创建加入群聊消息
 * @param groupId - 群聊ID
 * @param userName - 用户名
 * @param userId - 用户ID
 * @returns 加入消息
 */
export function createJoinMessage(
  groupId: string,
  userName: string,
  userId?: string
): GroupChatMessage {
  const content = `${userName} 加入了群聊`
  
  return {
    id: generateSystemMessageId('join', groupId),
    type: MessageType.JOIN,
    from: userId || 'system',
    fromName: userName,
    avatar: null,
    content,
    timestamp: Date.now(),
    metadata: {
      isSystem: true,
      eventType: 'join',
      userName,
      userId,
    },
  }
}

/**
 * 创建离开群聊消息
 * @param groupId - 群聊ID
 * @param userName - 用户名
 * @param userId - 用户ID
 * @returns 离开消息
 */
export function createLeaveMessage(
  groupId: string,
  userName: string,
  userId?: string
): GroupChatMessage {
  const content = `${userName} 离开了群聊`
  
  return {
    id: generateSystemMessageId('leave', groupId),
    type: MessageType.LEAVE,
    from: userId || 'system',
    fromName: userName,
    avatar: null,
    content,
    timestamp: Date.now(),
    metadata: {
      isSystem: true,
      eventType: 'leave',
      userName,
      userId,
    },
  }
}

// ============================================
// 消息转换函数
// ============================================

/**
 * 将 PubSub 消息转换为群聊消息
 * @param pubSubMsg - PubSub 消息数据
 * @param defaultAvatar - 默认头像
 * @returns 群聊消息
 */
export function transformPubSubMessage(
  pubSubMsg: PubSubMessageData,
  defaultAvatar?: string | null
): GroupChatMessage {
  const {
    id,
    type = MessageType.CHAT,
    from = 'system',
    content = '',
    timestamp = Date.now(),
    metadata = {},
    topic,
    to,
  } = pubSubMsg

  // 获取发送者名称
  const fromName = from === 'system'
    ? (i18n?.t?.('agent.groupChat.system') || '系统')
    : (metadata?.fromName as string) || from

  // 获取头像
  const avatar = (metadata?.avatar as string) || defaultAvatar || null

  return {
    id: id || generateMessageId(),
    type: type as MessageType,
    from,
    fromName,
    avatar,
    content,
    timestamp,
    metadata: {
      ...metadata,
      originalFrom: from,
      avatarSource: metadata?.avatar ? 'metadata' : 'default',
    },
    topic,
    to: to || null,
  }
}

/**
 * 将群聊消息转换为 PubSub 消息数据
 * @param message - 群聊消息
 * @returns PubSub 消息数据
 */
export function toPubSubMessageData(message: GroupChatMessage): PubSubMessageData {
  return {
    id: message.id,
    type: message.type,
    from: message.from,
    to: message.to,
    content: message.content,
    topic: message.topic || '',
    timestamp: message.timestamp,
    metadata: {
      ...message.metadata,
      fromName: message.fromName,
      avatar: message.avatar,
    },
  }
}

// ============================================
// 消息验证函数
// ============================================

/**
 * 验证消息是否有效
 * @param message - 要验证的消息
 * @throws 如果消息无效则抛出 GroupChatError
 */
export function validateMessage(message: Partial<GroupChatMessage>): void {
  if (!message) {
    throw new GroupChatError(
      GroupChatErrorCode.INVALID_PARAMS,
      '消息不能为空'
    )
  }

  if (!message.from) {
    throw new GroupChatError(
      GroupChatErrorCode.INVALID_PARAMS,
      '消息发送者不能为空'
    )
  }

  if (!message.content || typeof message.content !== 'string') {
    throw new GroupChatError(
      GroupChatErrorCode.INVALID_PARAMS,
      '消息内容不能为空且必须是字符串'
    )
  }

  if (message.content.trim().length === 0) {
    throw new GroupChatError(
      GroupChatErrorCode.INVALID_PARAMS,
      '消息内容不能为空字符串'
    )
  }

  // 检查内容长度（防止过大消息）
  const MAX_CONTENT_LENGTH = 10000 // 最大10KB
  if (message.content.length > MAX_CONTENT_LENGTH) {
    throw new GroupChatError(
      GroupChatErrorCode.INVALID_PARAMS,
      `消息内容过长，最大允许 ${MAX_CONTENT_LENGTH} 字符`
    )
  }
}

/**
 * 检查消息是否重复
 * @param message - 要检查的消息
 * @param existingMessages - 已有消息列表
 * @returns 是否重复
 */
export function isDuplicateMessage(
  message: GroupChatMessage,
  existingMessages: GroupChatMessage[]
): boolean {
  return existingMessages.some(m => m.id === message.id)
}

/**
 * 检查消息是否为本地发送
 * @param message - 消息
 * @returns 是否为本地发送
 */
export function isLocalMessage(message: GroupChatMessage): boolean {
  return message.metadata?.isLocal === true
}

/**
 * 检查消息是否为系统消息
 * @param message - 消息
 * @returns 是否为系统消息
 */
export function isSystemMessage(message: GroupChatMessage): boolean {
  return message.type === MessageType.SYSTEM ||
    message.type === MessageType.JOIN ||
    message.type === MessageType.LEAVE ||
    message.type === MessageType.WELCOME ||
    message.from === 'system'
}

// ============================================
// 消息过滤和排序
// ============================================

/**
 * 消息过滤选项
 */
export interface MessageFilterOptions {
  /** 只包含特定类型 */
  types?: MessageType[]
  /** 排除特定类型 */
  excludeTypes?: MessageType[]
  /** 只包含特定发送者 */
  fromUsers?: string[]
  /** 排除特定发送者 */
  excludeUsers?: string[]
  /** 时间范围开始 */
  startTime?: number
  /** 时间范围结束 */
  endTime?: number
  /** 搜索关键词 */
  searchKeyword?: string
}

/**
 * 过滤消息列表
 * @param messages - 消息列表
 * @param options - 过滤选项
 * @returns 过滤后的消息列表
 */
export function filterMessages(
  messages: GroupChatMessage[],
  options: MessageFilterOptions = {}
): GroupChatMessage[] {
  return messages.filter(message => {
    const { types, excludeTypes, fromUsers, excludeUsers, startTime, endTime, searchKeyword } = options

    // 类型过滤
    if (types && !types.includes(message.type as MessageType)) {
      return false
    }

    // 排除类型
    if (excludeTypes && excludeTypes.includes(message.type as MessageType)) {
      return false
    }

    // 发送者过滤
    if (fromUsers && !fromUsers.includes(message.from)) {
      return false
    }

    // 排除发送者
    if (excludeUsers && excludeUsers.includes(message.from)) {
      return false
    }

    // 时间范围过滤
    if (startTime && message.timestamp < startTime) {
      return false
    }

    if (endTime && message.timestamp > endTime) {
      return false
    }

    // 关键词搜索
    if (searchKeyword && !message.content.toLowerCase().includes(searchKeyword.toLowerCase())) {
      return false
    }

    return true
  })
}

/**
 * 按时间戳排序消息（升序）
 * @param messages - 消息列表
 * @returns 排序后的消息列表
 */
export function sortMessagesByTime(messages: GroupChatMessage[]): GroupChatMessage[] {
  return [...messages].sort((a, b) => a.timestamp - b.timestamp)
}

/**
 * 按时间戳排序消息（降序）
 * @param messages - 消息列表
 * @returns 排序后的消息列表
 */
export function sortMessagesByTimeDesc(messages: GroupChatMessage[]): GroupChatMessage[] {
  return [...messages].sort((a, b) => b.timestamp - a.timestamp)
}

// ============================================
// 消息批量处理
// ============================================

/**
 * 合并新消息到现有消息列表（去重并排序）
 * @param existingMessages - 现有消息
 * @param newMessages - 新消息
 * @returns 合并后的消息列表
 */
export function mergeMessages(
  existingMessages: GroupChatMessage[],
  newMessages: GroupChatMessage[]
): GroupChatMessage[] {
  const existingIds = new Set(existingMessages.map(m => m.id))
  const uniqueNewMessages = newMessages.filter(m => !existingIds.has(m.id))
  return sortMessagesByTime([...existingMessages, ...uniqueNewMessages])
}

/**
 * 批量添加消息（线程安全方式）
 * @param existingMessages - 现有消息列表
 * @param messagesToAdd - 要添加的消息列表
 * @returns 添加后的消息列表
 */
export function batchAddMessages(
  existingMessages: GroupChatMessage[],
  messagesToAdd: GroupChatMessage[]
): GroupChatMessage[] {
  const existingIds = new Set(existingMessages.map(m => m.id))
  const uniqueMessages = messagesToAdd.filter(m => !existingIds.has(m.id))
  return [...existingMessages, ...uniqueMessages]
}

/**
 * 截断消息列表到最大长度
 * @param messages - 消息列表
 * @param maxLength - 最大长度
 * @returns 截断后的消息列表
 */
export function truncateMessages(
  messages: GroupChatMessage[],
  maxLength: number
): GroupChatMessage[] {
  if (messages.length <= maxLength) {
    return messages
  }
  return messages.slice(messages.length - maxLength)
}

// ============================================
// 消息统计
// ============================================

/**
 * 消息统计结果
 */
export interface MessageStats {
  total: number
  byType: Record<string, number>
  byUser: Record<string, number>
  timeRange: {
    earliest: number | null
    latest: number | null
  }
}

/**
 * 统计消息列表
 * @param messages - 消息列表
 * @returns 统计结果
 */
export function getMessageStats(messages: GroupChatMessage[]): MessageStats {
  const byType: Record<string, number> = {}
  const byUser: Record<string, number> = {}
  let earliest: number | null = null
  let latest: number | null = null

  messages.forEach(message => {
    // 按类型统计
    byType[message.type] = (byType[message.type] || 0) + 1

    // 按用户统计
    byUser[message.from] = (byUser[message.from] || 0) + 1

    // 时间范围
    if (earliest === null || message.timestamp < earliest) {
      earliest = message.timestamp
    }
    if (latest === null || message.timestamp > latest) {
      latest = message.timestamp
    }
  })

  return {
    total: messages.length,
    byType,
    byUser,
    timeRange: { earliest, latest },
  }
}

// ============================================
// 导出兼容函数
// ============================================

/** @deprecated 使用 createMessage 替代 */
export const createChatMessage = createMessage

/** @deprecated 使用 transformPubSubMessage 替代 */
export const convertPubSubToGroupMessage = transformPubSubMessage
