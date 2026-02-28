/**
 * ID 生成工具
 * 
 * 提供统一的 ID 生成规则，确保所有群聊相关 ID 的一致性和可追溯性
 * 
 * @module utils/groupchat/idGenerator
 * @author Alou Team
 * @since 1.0.0
 */

// ============================================
// 常量定义
// ============================================

/** ID 前缀常量 */
export const ID_PREFIX = {
  /** 消息ID前缀 */
  MESSAGE: 'msg',
  /** 群聊ID前缀 */
  GROUP: 'group',
  /** 本地群聊ID前缀 */
  LOCAL_GROUP: 'local_group',
  /** DIAP群聊ID前缀 */
  DIAP_GROUP: 'diap_group',
  /** 行动ID前缀 */
  ACTION: 'action',
  /** 任务ID前缀 */
  TASK: 'task',
  /** 频道ID前缀 */
  CHANNEL: 'channel',
  /** 智能体ID前缀 */
  AGENT: 'agent',
  /** 欢迎消息ID前缀 */
  WELCOME: 'welcome',
  /** 系统消息ID前缀 */
  SYSTEM: 'system',
} as const

/** ID 分隔符 */
export const ID_SEPARATOR = '_'

// ============================================
// 基础生成函数
// ============================================

/**
 * 生成时间戳部分
 * @returns 毫秒级时间戳字符串
 */
function generateTimestamp(): string {
  return Date.now().toString(36)
}

/**
 * 生成随机部分
 * @param length - 随机字符串长度，默认 6
 * @returns 随机字符串
 */
function generateRandomPart(length: number = 6): string {
  return Math.random()
    .toString(36)
    .substring(2, 2 + length)
}

/**
 * 生成唯一ID
 * @param prefix - ID前缀
 * @param identifier - 可选标识符（如群聊ID）
 * @returns 生成的唯一ID
 * 
 * @example
 * ```typescript
 * generateUniqueId('msg') // 'msg_k2j4h5_a3b9c2'
 * generateUniqueId('msg', 'group123') // 'msg_group123_k2j4h5'
 * ```
 */
export function generateUniqueId(prefix: string, identifier?: string): string {
  const parts = [prefix, generateTimestamp(), generateRandomPart()]
  
  if (identifier) {
    // 将 identifier 插入到 prefix 和时间戳之间
    parts.splice(1, 0, identifier)
  }
  
  return parts.join(ID_SEPARATOR)
}

// ============================================
// 专用 ID 生成函数
// ============================================

/**
 * 生成消息ID
 * @param groupId - 可选的群聊ID（用于关联）
 * @returns 消息ID
 * 
 * @example
 * ```typescript
 * generateMessageId() // 'msg_k2j4h5_a3b9c2'
 * generateMessageId('group123') // 'msg_group123_k2j4h5_a3b9c2'
 * ```
 */
export function generateMessageId(groupId?: string): string {
  return generateUniqueId(ID_PREFIX.MESSAGE, groupId)
}

/**
 * 生成群聊ID
 * @param type - 群聊类型：'local' | 'diap' | 'action'
 * @returns 群聊ID
 * 
 * @example
 * ```typescript
 * generateGroupId('local') // 'local_group_k2j4h5_a3b9c2'
 * generateGroupId('diap') // 'diap_group_k2j4h5_a3b9c2'
 * ```
 */
export function generateGroupId(type: 'local' | 'diap' | 'action' = 'local'): string {
  const prefixMap = {
    local: ID_PREFIX.LOCAL_GROUP,
    diap: ID_PREFIX.DIAP_GROUP,
    action: ID_PREFIX.ACTION,
  }
  
  return generateUniqueId(prefixMap[type])
}

/**
 * 生成欢迎消息ID
 * @param groupId - 群聊ID
 * @returns 欢迎消息ID
 * 
 * @example
 * ```typescript
 * generateWelcomeMessageId('group123') // 'welcome_group123_k2j4h5'
 * ```
 */
export function generateWelcomeMessageId(groupId: string): string {
  return `${ID_PREFIX.WELCOME}${ID_SEPARATOR}${groupId}`
}

/**
 * 生成系统消息ID
 * @param type - 系统消息类型
 * @param groupId - 群聊ID
 * @returns 系统消息ID
 * 
 * @example
 * ```typescript
 * generateSystemMessageId('join', 'group123') // 'system_join_group123_k2j4h5'
 * ```
 */
export function generateSystemMessageId(type: string, groupId: string): string {
  return generateUniqueId(`${ID_PREFIX.SYSTEM}${ID_SEPARATOR}${type}`, groupId)
}

/**
 * 生成任务ID
 * @returns 任务ID
 * 
 * @example
 * ```typescript
 * generateTaskId() // 'task_k2j4h5_a3b9c2'
 * ```
 */
export function generateTaskId(): string {
  return generateUniqueId(ID_PREFIX.TASK)
}

/**
 * 生成频道ID
 * @returns 频道ID
 * 
 * @example
 * ```typescript
 * generateChannelId() // 'channel_k2j4h5_a3b9c2'
 * ```
 */
export function generateChannelId(): string {
  return generateUniqueId(ID_PREFIX.CHANNEL)
}

/**
 * 生成智能体ID
 * @returns 智能体ID
 * 
 * @example
 * ```typescript
 * generateAgentId() // 'agent_k2j4h5_a3b9c2'
 * ```
 */
export function generateAgentId(): string {
  return generateUniqueId(ID_PREFIX.AGENT)
}

// ============================================
// 主题生成函数
// ============================================

/**
 * 生成群聊主题
 * @param groupId - 群聊ID
 * @param namespace - 命名空间，默认 'diap/group'
 * @returns 主题字符串
 * 
 * @example
 * ```typescript
 * generateGroupTopic('group123') // 'diap/group/group123'
 * ```
 */
export function generateGroupTopic(groupId: string, namespace: string = 'diap/group'): string {
  return `${namespace}/${groupId}`
}

/**
 * 生成集群行动主题
 * @param actionId - 行动ID
 * @returns 主题字符串
 * 
 * @example
 * ```typescript
 * generateClusterActionTopic('action123') // 'diap/cluster_action/action123'
 * ```
 */
export function generateClusterActionTopic(actionId: string): string {
  return `diap/cluster_action/${actionId}`
}

/**
 * 从主题提取ID
 * @param topic - 主题字符串
 * @returns 提取的ID或null
 * 
 * @example
 * ```typescript
 * extractIdFromTopic('diap/group/group123') // 'group123'
 * extractIdFromTopic('invalid') // null
 * ```
 */
export function extractIdFromTopic(topic: string): string | null {
  const parts = topic.split('/')
  return parts.length > 0 ? parts[parts.length - 1] : null
}

// ============================================
// ID 验证函数
// ============================================

/**
 * 检查字符串是否为有效的群聊ID
 * @param id - 要检查的字符串
 * @returns 是否为有效的群聊ID
 */
export function isValidGroupId(id: string): boolean {
  if (!id || typeof id !== 'string') return false
  
  const validPrefixes = [
    ID_PREFIX.LOCAL_GROUP,
    ID_PREFIX.DIAP_GROUP,
    ID_PREFIX.GROUP,
    ID_PREFIX.ACTION,
  ]
  
  return validPrefixes.some(prefix => id.startsWith(prefix + ID_SEPARATOR))
}

/**
 * 检查字符串是否为本地群聊ID
 * @param id - 要检查的字符串
 * @returns 是否为本地群聊ID
 */
export function isLocalGroupId(id: string): boolean {
  return typeof id === 'string' && id.startsWith(ID_PREFIX.LOCAL_GROUP + ID_SEPARATOR)
}

/**
 * 检查字符串是否为DIAP群聊ID
 * @param id - 要检查的字符串
 * @returns 是否为DIAP群聊ID
 */
export function isDiapGroupId(id: string): boolean {
  return typeof id === 'string' && id.startsWith(ID_PREFIX.DIAP_GROUP + ID_SEPARATOR)
}

/**
 * 检查字符串是否为行动ID
 * @param id - 要检查的字符串
 * @returns 是否为行动ID
 */
export function isActionId(id: string): boolean {
  return typeof id === 'string' && id.startsWith(ID_PREFIX.ACTION + ID_SEPARATOR)
}

/**
 * 检查字符串是否为任务ID
 * @param id - 要检查的字符串
 * @returns 是否为任务ID
 */
export function isTaskId(id: string): boolean {
  return typeof id === 'string' && id.startsWith(ID_PREFIX.TASK + ID_SEPARATOR)
}

// ============================================
// ID 转换函数
// ============================================

/**
 * 将行动ID转换为群聊ID
 * @param actionId - 行动ID
 * @returns 群聊ID
 * 
 * @example
 * ```typescript
 * actionIdToGroupId('action_abc123') // 'group_abc123'
 * ```
 */
export function actionIdToGroupId(actionId: string): string {
  if (isActionId(actionId)) {
    return actionId.replace(ID_PREFIX.ACTION + ID_SEPARATOR, ID_PREFIX.GROUP + ID_SEPARATOR)
  }
  return actionId
}

/**
 * 将群聊ID转换为行动ID
 * @param groupId - 群聊ID
 * @returns 行动ID
 * 
 * @example
 * ```typescript
 * groupIdToActionId('group_abc123') // 'action_abc123'
 * ```
 */
export function groupIdToActionId(groupId: string): string {
  if (groupId.startsWith(ID_PREFIX.GROUP + ID_SEPARATOR)) {
    return groupId.replace(ID_PREFIX.GROUP + ID_SEPARATOR, ID_PREFIX.ACTION + ID_SEPARATOR)
  }
  return groupId
}

// ============================================
// 工具函数
// ============================================

/**
 * 生成用于排序的时间戳ID
 * @returns 时间戳ID
 * 
 * @example
 * ```typescript
 * generateSortableId() // 'msg_1704067200000_a3b9c2'
 * ```
 */
export function generateSortableId(prefix: string = ID_PREFIX.MESSAGE): string {
  // 使用时间戳作为第一部分，确保排序正确
  const timestamp = Date.now().toString()
  const random = generateRandomPart(4)
  return `${prefix}${ID_SEPARATOR}${timestamp}${ID_SEPARATOR}${random}`
}

/**
 * 从ID中提取时间戳
 * @param id - ID字符串
 * @returns 时间戳或null
 * 
 * @example
 * ```typescript
 * extractTimestampFromId('msg_k2j4h5_a3b9c2') // 1704067200000
 * ```
 */
export function extractTimestampFromId(id: string): number | null {
  if (!id || typeof id !== 'string') return null
  
  const parts = id.split(ID_SEPARATOR)
  if (parts.length < 2) return null
  
  // 尝试解析时间戳部分（通常是第二部分）
  const timestampPart = parts[1]
  const timestamp = parseInt(timestampPart, 36)
  
  return isNaN(timestamp) ? null : timestamp
}

/**
 * 比较两个ID的时间顺序
 * @param id1 - 第一个ID
 * @param id2 - 第二个ID
 * @returns 比较结果：负数表示id1在前，正数表示id2在前，0表示相同
 */
export function compareIdsByTimestamp(id1: string, id2: string): number {
  const ts1 = extractTimestampFromId(id1)
  const ts2 = extractTimestampFromId(id2)
  
  if (ts1 === null && ts2 === null) return 0
  if (ts1 === null) return 1
  if (ts2 === null) return -1
  
  return ts1 - ts2
}
