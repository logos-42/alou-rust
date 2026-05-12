/**
 * SharedContext Types - 共享上下文类型定义
 *
 * 设计目标：
 * 1. 多个智囊团(Agent Swarm)通过索引共享上下文
 * 2. 使用 IPFS 存储实现分布式持久化
 * 3. 使用共享 session 记录和分享记忆
 *
 * @module types/sharedContext
 */

import { AgentInfo, GroupChatMessage } from './groupchat'

/**
 * 上下文引用类型
 */
export enum ContextRefType {
  /** IPFS CID 引用 */
  CID = 'cid',
  /** IPNS 名称引用 */
  IPNS = 'ipns',
  /** 本地 Session ID 引用 */
  SESSION = 'session',
  /** 组合引用 */
  COMPOSITE = 'composite',
}

/**
 * 上下文引用
 */
export interface ContextRef {
  /** 引用类型 */
  type: ContextRefType
  /** 引用值 (CID/IPNS/SessionID) */
  value: string
  /** 引用名称/描述 */
  name?: string
  /** 创建时间戳 */
  createdAt: number
  /** 引用元数据 */
  metadata?: Record<string, unknown>
}

/**
 * 上下文条目
 */
export interface ContextEntry {
  /** 唯一标识 */
  id: string
  /** 上下文索引 (用于快速查找) */
  index: string
  /** 上下文类型 */
  type: ContextEntryType
  /** 上下文内容 */
  content: ContextContent
  /** 引用信息 */
  ref: ContextRef
  /** 所属智囊团/群组 */
  swarmId?: string
  /** 创建者 */
  creator: AgentInfo
  /** 创建时间 */
  createdAt: number
  /** 更新时间 */
  updatedAt: number
  /** 版本号 */
  version: number
  /** 标签 */
  tags: string[]
  /** 是否共享 */
  shared: boolean
  /** 共享权限 */
  permissions?: ContextPermissions
}

/**
 * 上下文条目类型
 */
export enum ContextEntryType {
  /** 记忆碎片 */
  MEMORY = 'memory',
  /** 对话摘要 */
  SUMMARY = 'summary',
  /** 任务结果 */
  TASK_RESULT = 'task_result',
  /** 知识片段 */
  KNOWLEDGE = 'knowledge',
  /** 决策记录 */
  DECISION = 'decision',
  /** 工具调用结果 */
  TOOL_RESULT = 'tool_result',
  /** 智囊团状态 */
  SWARM_STATE = 'swarm_state',
}

/**
 * 上下文内容
 */
export interface ContextContent {
  /** 文本内容 */
  text?: string
  /** 结构化数据 */
  data?: Record<string, unknown>
  /** 关联的消息 */
  messages?: GroupChatMessage[]
  /** 关联的实体 */
  entities?: ContextEntity[]
  /** 原始格式 */
  raw?: unknown
}

/**
 * 上下文实体
 */
export interface ContextEntity {
  /** 实体类型 */
  type: string
  /** 实体 ID */
  id: string
  /** 实体名称 */
  name: string
  /** 实体属性 */
  properties?: Record<string, unknown>
}

/**
 * 上下文权限
 */
export interface ContextPermissions {
  /** 是否可读 */
  readable: boolean
  /** 是否可写 */
  writable: boolean
  /** 是否可删除 */
  deletable: boolean
  /** 授权的智囊团列表 */
  authorizedSwarms: string[]
  /** 授权的 Agent 列表 */
  authorizedAgents: string[]
}

/**
 * 智囊团上下文
 */
export interface SwarmContext {
  /** 智囊团唯一标识 */
  id: string
  /** 智囊团名称 */
  name: string
  /** 智囊团描述 */
  description?: string
  /** 成员 Agent 列表 */
  members: AgentInfo[]
  /** 共享上下文索引 */
  contextIndexes: ContextIndex[]
  /** 共享记忆 */
  sharedMemories: ContextEntry[]
  /** 创建时间 */
  createdAt: number
  /** 活动时间 */
  lastActivityAt: number
  /** 根上下文 CID (IPFS) */
  rootCid?: string
  /** IPNS 名称 */
  ipnsName?: string
}

/**
 * 上下文索引
 * 用于快速查找和引用共享上下文
 */
export interface ContextIndex {
  /** 索引唯一标识 */
  id: string
  /** 索引名称 */
  name: string
  /** 索引类型 */
  type: ContextIndexType
  /** 索引键 (用于精确匹配) */
  key: string
  /** 索引值 */
  value: string
  /** 关联的上下文条目 */
  entryIds: string[]
  /** 创建时间 */
  createdAt: number
  /** 索引权重 (用于排序) */
  weight: number
}

/**
 * 上下文索引类型
 */
export enum ContextIndexType {
  /** 标签索引 */
  TAG = 'tag',
  /** 时间索引 */
  TIME = 'time',
  /** 类型索引 */
  TYPE = 'type',
  /** 创建者索引 */
  CREATOR = 'creator',
  /** 智囊团索引 */
  SWARM = 'swarm',
  /** 关键词索引 */
  KEYWORD = 'keyword',
  /** 语义向量索引 (预留) */
  VECTOR = 'vector',
}

/**
 * 共享 Session
 */
export interface SharedSession {
  /** Session 唯一标识 */
  id: string
  /** Session 名称 */
  name: string
  /** 参与智囊团列表 */
  swarms: string[]
  /** 上下文引用 */
  contextRefs: ContextRef[]
  /** 共享状态 */
  state: SharedSessionState
  /** 创建者 */
  creator: AgentInfo
  /** 创建时间 */
  createdAt: number
  /** 最后同步时间 */
  lastSyncAt: number
  /** Session 配置 */
  config: SharedSessionConfig
}

/**
 * 共享 Session 状态
 */
export interface SharedSessionState {
  /** 是否活跃 */
  active: boolean
  /** 当前参与者 */
  participants: AgentInfo[]
  /** 同步状态 */
  syncStatus: SyncStatus
  /** 冲突列表 */
  conflicts: ContextConflict[]
}

/**
 * 同步状态
 */
export enum SyncStatus {
  /** 已同步 */
  SYNCED = 'synced',
  /** 同步中 */
  SYNCING = 'syncing',
  /** 需要同步 */
  PENDING = 'pending',
  /** 冲突 */
  CONFLICT = 'conflict',
  /** 离线 */
  OFFLINE = 'offline',
}

/**
 * 上下文冲突
 */
export interface ContextConflict {
  /** 冲突 ID */
  id: string
  /** 冲突的上下文条目 */
  entries: ContextEntry[]
  /** 冲突类型 */
  type: ConflictType
  /** 冲突描述 */
  description: string
  /** 解决状态 */
  resolved: boolean
  /** 解决方案 */
  resolution?: ContextEntry
}

/**
 * 冲突类型
 */
export enum ConflictType {
  /** 版本冲突 */
  VERSION = 'version',
  /** 并发修改 */
  CONCURRENT = 'concurrent',
  /** 权限冲突 */
  PERMISSION = 'permission',
  /** 语义冲突 */
  SEMANTIC = 'semantic',
}

/**
 * 共享 Session 配置
 */
export interface SharedSessionConfig {
  /** 自动同步间隔 (毫秒) */
  autoSyncInterval: number
  /** 同步策略 */
  syncStrategy: SyncStrategy
  /** 冲突解决策略 */
  conflictResolution: ConflictResolutionStrategy
  /** 最大上下文条目数 */
  maxEntries: number
  /** 上下文过期时间 (毫秒) */
  contextTtl: number
  /** 是否启用 IPFS 存储 */
  useIpfsStorage: boolean
}

/**
 * 同步策略
 */
export enum SyncStrategy {
  /** 实时同步 */
  REALTIME = 'realtime',
  /** 按需同步 */
  ON_DEMAND = 'on_demand',
  /** 定期同步 */
  PERIODIC = 'periodic',
}

/**
 * 冲突解决策略
 */
export enum ConflictResolutionStrategy {
  /** 最新优先 */
  LATEST = 'latest',
  /** 创建者优先 */
  CREATOR = 'creator',
  /** 手动解决 */
  MANUAL = 'manual',
  /** 合并策略 */
  MERGE = 'merge',
}

/**
 * IPFS 存储条目
 */
export interface IpfsStoredEntry {
  /** IPFS CID */
  cid: string
  /** 条目数据 */
  entry: ContextEntry
  /** 存储时间 */
  storedAt: number
  /** 是否固定 */
  pinned: boolean
  /** 固定者 */
  pinnedBy?: string
}

/**
 * 共享上下文操作结果
 */
export interface SharedContextResult<T = void> {
  /** 是否成功 */
  success: boolean
  /** 结果数据 */
  data?: T
  /** 错误信息 */
  error?: string
  /** 操作耗时 */
  duration?: number
}

/**
 * 共享上下文查询选项
 */
export interface ContextQueryOptions {
  /** 智囊团 ID */
  swarmId?: string
  /** 上下文类型 */
  type?: ContextEntryType
  /** 标签 */
  tags?: string[]
  /** 创建者 */
  creatorId?: string
  /** 时间范围 */
  timeRange?: {
    start: number
    end: number
  }
  /** 关键词 */
  keyword?: string
  /** 分页 */
  pagination?: {
    offset: number
    limit: number
  }
  /** 排序 */
  sort?: {
    field: string
    order: 'asc' | 'desc'
  }
}

/**
 * 共享上下文统计
 */
export interface SharedContextStats {
  /** 总条目数 */
  totalEntries: number
  /** 共享条目数 */
  sharedEntries: number
  /** 智囊团数 */
  swarmCount: number
  /** IPFS 存储条目数 */
  ipfsEntries: number
  /** 总存储大小 (字节) */
  totalSize: number
  /** 最后同步时间 */
  lastSyncAt: number
}

/**
 * 类型守卫函数
 */
export function isContextRef(value: unknown): value is ContextRef {
  return (
    typeof value === 'object' &&
    value !== null &&
    'type' in value &&
    'value' in value &&
    'createdAt' in value
  )
}

export function isContextEntry(value: unknown): value is ContextEntry {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'index' in value &&
    'type' in value &&
    'content' in value &&
    'ref' in value
  )
}

export function isSwarmContext(value: unknown): value is SwarmContext {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'name' in value &&
    'members' in value &&
    'contextIndexes' in value
  )
}

export function isSharedSession(value: unknown): value is SharedSession {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'name' in value &&
    'swarms' in value &&
    'contextRefs' in value
  )
}