/**
 * 群聊存储辅助工具
 * 
 * 提供统一的存储操作封装，支持内存存储和 localStorage 降级
 * 自动处理数据序列化、压缩和错误恢复
 * 
 * @module utils/groupchat/storageHelper
 * @author Alou Team
 * @since 1.0.0
 */

import {
  GroupChatAction,
  GroupChatMessage,
  GroupChatError,
  GroupChatErrorCode,
} from '@/types/groupchat'

// ============================================
// 常量定义
// ============================================

/** 存储键前缀 */
const STORAGE_PREFIX = {
  GROUP_CHATS: 'cluster_actions_group_chats',
  ACTIVE_ID: 'cluster_actions_active_id',
  MESSAGES: 'cluster_actions_messages',
  METADATA: 'cluster_actions_metadata',
} as const

/** 存储类型 */
export enum StorageType {
  /** 内存存储 */
  MEMORY = 'memory',
  /** LocalStorage */
  LOCAL_STORAGE = 'localStorage',
  /** SessionStorage */
  SESSION_STORAGE = 'sessionStorage',
}

/** 存储配置 */
export interface StorageConfig {
  /** 存储类型 */
  type: StorageType
  /** 命名空间 */
  namespace?: string
  /** 最大存储项数 */
  maxItems?: number
  /** 是否压缩 */
  compress?: boolean
}

// ============================================
// 内存存储实现
// ============================================

class MemoryStorage {
  private data: Map<string, string> = new Map()

  getItem(key: string): string | null {
    return this.data.get(key) || null
  }

  setItem(key: string, value: string): void {
    this.data.set(key, value)
  }

  removeItem(key: string): void {
    this.data.delete(key)
  }

  clear(): void {
    this.data.clear()
  }

  keys(): string[] {
    return Array.from(this.data.keys())
  }
}

// 全局内存存储实例
const globalMemoryStorage = new MemoryStorage()

// ============================================
// 存储适配器
// ============================================

/**
 * 获取存储实例
 * @param type - 存储类型
 * @returns 存储实例
 */
function getStorage(type: StorageType): Storage {
  switch (type) {
    case StorageType.LOCAL_STORAGE:
      return typeof window !== 'undefined' ? window.localStorage : globalMemoryStorage
    case StorageType.SESSION_STORAGE:
      return typeof window !== 'undefined' ? window.sessionStorage : globalMemoryStorage
    case StorageType.MEMORY:
    default:
      return globalMemoryStorage
  }
}

/**
 * 生成完整存储键
 * @param prefix - 键前缀
 * @param identifier - 标识符
 * @param namespace - 命名空间
 * @returns 完整键名
 */
function buildStorageKey(
  prefix: string,
  identifier?: string,
  namespace?: string
): string {
  const parts = [prefix]
  if (namespace) parts.push(namespace)
  if (identifier) parts.push(identifier)
  return parts.join('_')
}

// ============================================
// 基础存储操作
// ============================================

/**
 * 安全解析 JSON
 * @param jsonString - JSON 字符串
 * @param defaultValue - 解析失败时的默认值
 * @returns 解析结果
 */
function safeJsonParse<T>(jsonString: string | null, defaultValue: T): T {
  if (!jsonString) return defaultValue
  try {
    return JSON.parse(jsonString) as T
  } catch {
    return defaultValue
  }
}

/**
 * 安全序列化为 JSON
 * @param value - 要序列化的值
 * @returns JSON 字符串或 null
 */
function safeJsonStringify(value: unknown): string | null {
  try {
    return JSON.stringify(value)
  } catch {
    return null
  }
}

/**
 * 从存储读取数据
 * @param key - 存储键
 * @param storageType - 存储类型
 * @returns 存储的数据
 */
export function readFromStorage<T>(
  key: string,
  storageType: StorageType = StorageType.MEMORY
): T | null {
  try {
    const storage = getStorage(storageType)
    const data = storage.getItem(key)
    return safeJsonParse(data, null)
  } catch (error) {
    console.error(`[storageHelper] 读取存储失败 [${key}]:`, error)
    return null
  }
}

/**
 * 写入数据到存储
 * @param key - 存储键
 * @param value - 要存储的数据
 * @param storageType - 存储类型
 * @returns 是否成功
 */
export function writeToStorage<T>(
  key: string,
  value: T,
  storageType: StorageType = StorageType.MEMORY
): boolean {
  try {
    const storage = getStorage(storageType)
    const jsonString = safeJsonStringify(value)
    
    if (jsonString === null) {
      throw new GroupChatError(
        GroupChatErrorCode.STORAGE,
        '数据序列化失败'
      )
    }

    storage.setItem(key, jsonString)
    return true
  } catch (error) {
    console.error(`[storageHelper] 写入存储失败 [${key}]:`, error)
    return false
  }
}

/**
 * 从存储删除数据
 * @param key - 存储键
 * @param storageType - 存储类型
 */
export function removeFromStorage(
  key: string,
  storageType: StorageType = StorageType.MEMORY
): void {
  try {
    const storage = getStorage(storageType)
    storage.removeItem(key)
  } catch (error) {
    console.error(`[storageHelper] 删除存储失败 [${key}]:`, error)
  }
}

/**
 * 清空存储
 * @param storageType - 存储类型
 * @param prefix - 只删除指定前缀的键
 */
export function clearStorage(
  storageType: StorageType = StorageType.MEMORY,
  prefix?: string
): void {
  try {
    const storage = getStorage(storageType)
    
    if (prefix) {
      // 只删除指定前缀的键
      const keys = storage instanceof MemoryStorage 
        ? storage.keys()
        : Object.keys(storage)
      
      keys.forEach(key => {
        if (key.startsWith(prefix)) {
          storage.removeItem(key)
        }
      })
    } else {
      storage.clear()
    }
  } catch (error) {
    console.error('[storageHelper] 清空存储失败:', error)
  }
}

// ============================================
// 群聊专用存储函数
// ============================================

/**
 * 存储优化：只保留必要的数据
 * @param action - 群聊行动
 * @returns 优化后的行动数据
 */
export function optimizeGroupChatAction(action: GroupChatAction): Partial<GroupChatAction> {
  return {
    action_id: action.action_id,
    description: action.description,
    status: action.status,
    created_at: action.created_at,
    // 只保留前3个智能体的信息，减少数据量
    agents: action.agents?.slice(0, 3).map(agent => ({
      id: agent.id,
      name: agent.name,
      avatar: agent.avatar,
      mode: agent.mode,
    })),
    // 只保留metadata的关键信息
    metadata: action.metadata ? {
      type: action.metadata.type,
      channel_id: action.metadata.channel_id,
      channel_name: action.metadata.channel_name,
    } : {},
  }
}

/**
 * 保存群聊列表到存储
 * @param channelId - 频道ID
 * @param actions - 群聊行动列表
 * @param storageType - 存储类型
 * @returns 是否成功
 */
export function saveGroupChats(
  channelId: string,
  actions: GroupChatAction[],
  storageType: StorageType = StorageType.MEMORY
): boolean {
  const key = buildStorageKey(STORAGE_PREFIX.GROUP_CHATS, channelId)
  
  // 优化数据
  const optimizedActions = actions.map(optimizeGroupChatAction)
  
  const success = writeToStorage(key, optimizedActions, storageType)
  
  if (success) {
    console.log(`[storageHelper] 保存群聊 (${channelId}):`, optimizedActions.length, '个')
  }
  
  return success
}

/**
 * 从存储加载群聊列表
 * @param channelId - 频道ID
 * @param storageType - 存储类型
 * @returns 群聊行动列表
 */
export function loadGroupChats(
  channelId: string,
  storageType: StorageType = StorageType.MEMORY
): GroupChatAction[] {
  const key = buildStorageKey(STORAGE_PREFIX.GROUP_CHATS, channelId)
  const actions = readFromStorage<GroupChatAction[]>(key, storageType)
  
  if (actions && actions.length > 0) {
    console.log(`[storageHelper] 加载群聊 (${channelId}):`, actions.length, '个')
  }
  
  return actions || []
}

/**
 * 保存活跃群聊ID
 * @param channelId - 频道ID
 * @param actionId - 行动ID
 * @param storageType - 存储类型
 * @returns 是否成功
 */
export function saveActiveGroupId(
  channelId: string,
  actionId: string | null,
  storageType: StorageType = StorageType.MEMORY
): boolean {
  const key = buildStorageKey(STORAGE_PREFIX.ACTIVE_ID, channelId)
  return writeToStorage(key, actionId, storageType)
}

/**
 * 加载活跃群聊ID
 * @param channelId - 频道ID
 * @param storageType - 存储类型
 * @returns 活跃行动ID
 */
export function loadActiveGroupId(
  channelId: string,
  storageType: StorageType = StorageType.MEMORY
): string | null {
  const key = buildStorageKey(STORAGE_PREFIX.ACTIVE_ID, channelId)
  return readFromStorage<string | null>(key, storageType)
}

/**
 * 保存群聊消息
 * @param actionId - 行动ID
 * @param messages - 消息列表
 * @param storageType - 存储类型
 * @returns 是否成功
 */
export function saveGroupChatMessages(
  actionId: string,
  messages: GroupChatMessage[],
  storageType: StorageType = StorageType.MEMORY
): boolean {
  const key = buildStorageKey(STORAGE_PREFIX.MESSAGES, actionId)
  
  // 限制存储的消息数量
  const MAX_STORED_MESSAGES = 500
  const messagesToStore = messages.slice(-MAX_STORED_MESSAGES)
  
  return writeToStorage(key, messagesToStore, storageType)
}

/**
 * 加载群聊消息
 * @param actionId - 行动ID
 * @param storageType - 存储类型
 * @returns 消息列表
 */
export function loadGroupChatMessages(
  actionId: string,
  storageType: StorageType = StorageType.MEMORY
): GroupChatMessage[] {
  const key = buildStorageKey(STORAGE_PREFIX.MESSAGES, actionId)
  return readFromStorage<GroupChatMessage[]>(key, storageType) || []
}

/**
 * 删除群聊相关数据
 * @param actionId - 行动ID
 * @param channelId - 频道ID（可选）
 * @param storageType - 存储类型
 */
export function deleteGroupChatData(
  actionId: string,
  channelId?: string,
  storageType: StorageType = StorageType.MEMORY
): void {
  // 删除消息
  const messagesKey = buildStorageKey(STORAGE_PREFIX.MESSAGES, actionId)
  removeFromStorage(messagesKey, storageType)
  
  // 删除元数据
  const metadataKey = buildStorageKey(STORAGE_PREFIX.METADATA, actionId)
  removeFromStorage(metadataKey, storageType)
  
  // 如果提供了 channelId，从群聊列表中移除
  if (channelId) {
    const actions = loadGroupChats(channelId, storageType)
    const filteredActions = actions.filter(a => a.action_id !== actionId)
    saveGroupChats(channelId, filteredActions, storageType)
  }
}

/**
 * 清除所有群聊数据
 * @param storageType - 存储类型
 */
export function clearAllGroupChatData(storageType: StorageType = StorageType.MEMORY): void {
  clearStorage(storageType, STORAGE_PREFIX.GROUP_CHATS)
  clearStorage(storageType, STORAGE_PREFIX.ACTIVE_ID)
  clearStorage(storageType, STORAGE_PREFIX.MESSAGES)
  clearStorage(storageType, STORAGE_PREFIX.METADATA)
  
  console.log('[storageHelper] 已清除所有群聊数据')
}

// ============================================
// 存储管理器类
// ============================================

/**
 * 群聊存储管理器
 * 提供更高级的存储管理功能
 */
export class GroupChatStorageManager {
  private storageType: StorageType
  private namespace?: string

  constructor(config: StorageConfig) {
    this.storageType = config.type
    this.namespace = config.namespace
  }

  /**
   * 保存群聊列表
   */
  saveGroupChats(channelId: string, actions: GroupChatAction[]): boolean {
    const key = buildStorageKey(STORAGE_PREFIX.GROUP_CHATS, channelId, this.namespace)
    const optimizedActions = actions.map(optimizeGroupChatAction)
    return writeToStorage(key, optimizedActions, this.storageType)
  }

  /**
   * 加载群聊列表
   */
  loadGroupChats(channelId: string): GroupChatAction[] {
    const key = buildStorageKey(STORAGE_PREFIX.GROUP_CHATS, channelId, this.namespace)
    return readFromStorage<GroupChatAction[]>(key, this.storageType) || []
  }

  /**
   * 保存活跃群聊ID
   */
  saveActiveGroupId(channelId: string, actionId: string | null): boolean {
    const key = buildStorageKey(STORAGE_PREFIX.ACTIVE_ID, channelId, this.namespace)
    return writeToStorage(key, actionId, this.storageType)
  }

  /**
   * 加载活跃群聊ID
   */
  loadActiveGroupId(channelId: string): string | null {
    const key = buildStorageKey(STORAGE_PREFIX.ACTIVE_ID, channelId, this.namespace)
    return readFromStorage<string | null>(key, this.storageType)
  }

  /**
   * 保存消息
   */
  saveMessages(actionId: string, messages: GroupChatMessage[]): boolean {
    const key = buildStorageKey(STORAGE_PREFIX.MESSAGES, actionId, this.namespace)
    const MAX_STORED_MESSAGES = 500
    const messagesToStore = messages.slice(-MAX_STORED_MESSAGES)
    return writeToStorage(key, messagesToStore, this.storageType)
  }

  /**
   * 加载消息
   */
  loadMessages(actionId: string): GroupChatMessage[] {
    const key = buildStorageKey(STORAGE_PREFIX.MESSAGES, actionId, this.namespace)
    return readFromStorage<GroupChatMessage[]>(key, this.storageType) || []
  }

  /**
   * 清除所有数据
   */
  clearAll(): void {
    const prefixes = [
      STORAGE_PREFIX.GROUP_CHATS,
      STORAGE_PREFIX.ACTIVE_ID,
      STORAGE_PREFIX.MESSAGES,
      STORAGE_PREFIX.METADATA,
    ]
    
    prefixes.forEach(prefix => {
      const fullPrefix = buildStorageKey(prefix, undefined, this.namespace)
      clearStorage(this.storageType, fullPrefix)
    })
  }
}

// ============================================
// 导出默认实例
// ============================================

/** 默认存储管理器实例（内存存储） */
export const defaultStorageManager = new GroupChatStorageManager({
  type: StorageType.MEMORY,
})
