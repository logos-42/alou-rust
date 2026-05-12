/**
 * SharedContextManager - 共享上下文管理器
 *
 * 核心功能：
 * 1. 管理多个智囊团的共享上下文
 * 2. 通过索引实现快速查找和引用
 * 3. 使用 IPFS 存储实现分布式持久化
 * 4. 使用共享 session 记录和分享记忆
 *
 * @module services/sharedContextManager
 */

import { invoke } from '@tauri-apps/api/core'
import {
  ContextEntry,
  ContextEntryType,
  ContextRef,
  ContextRefType,
  SwarmContext,
  SharedSession,
  SharedSessionConfig,
  SharedSessionState,
  SharedContextResult,
  SharedContextStats,
  ContextQueryOptions,
  ContextIndex,
  ContextIndexType,
  ContextContent,
  SyncStatus,
  ConflictType,
  IpfsStoredEntry,
  isContextEntry,
  isSwarmContext,
  isSharedSession,
  ContextPermissions,
  AgentInfo,
} from '../types/sharedContext'
import { GroupChatMessage } from '../types/groupchat'
import ipfsService from './ipfsService'

/**
 * 生成唯一 ID
 */
function generateId(prefix: string): string {
  return `${prefix}_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
}

/**
 * 生成上下文索引
 */
function generateContextIndex(entry: ContextEntry): string {
  const parts = [
    entry.type,
    entry.swarmId || 'global',
    entry.tags.join(':'),
    entry.creator.id || 'unknown',
  ]
  return parts.join('|')
}

/**
 * 共享上下文管理器类
 */
export class SharedContextManager {
  private static instance: SharedContextManager | null = null

  /** 本地缓存 */
  private localCache: Map<string, ContextEntry> = new Map()
  /** 智囊团上下文 */
  private swarmContexts: Map<string, SwarmContext> = new Map()
  /** 共享 Sessions */
  private sharedSessions: Map<string, SharedSession> = new Map()
  /** 上下文索引 */
  private contextIndexes: Map<string, ContextIndex[]> = new Map()
  /** IPFS 存储映射 */
  private ipfsStorage: Map<string, IpfsStoredEntry> = new Map()
  /** 事件监听器 */
  private listeners: Map<string, Set<(event: SharedContextEvent) => void>> = new Map()

  /** 默认配置 */
  private defaultConfig: SharedSessionConfig = {
    autoSyncInterval: 30000,
    syncStrategy: 'periodic',
    conflictResolution: 'latest',
    maxEntries: 1000,
    contextTtl: 7 * 24 * 60 * 60 * 1000,
    useIpfsStorage: true,
  }

  private constructor() {
    this.restoreFromStorage()
  }

  /**
   * 获取单例实例
   */
  static getInstance(): SharedContextManager {
    if (!this.instance) {
      this.instance = new SharedContextManager()
    }
    return this.instance
  }

  /**
   * 重置单例
   */
  static resetInstance(): void {
    if (this.instance) {
      this.instance.destroy()
      this.instance = null
    }
  }

  /**
   * 从本地存储恢复
   */
  private async restoreFromStorage(): Promise<void> {
    try {
      const stored = localStorage.getItem('shared_context_manager')
      if (stored) {
        const data = JSON.parse(stored)
        if (data.swarmContexts) {
          for (const [id, context] of Object.entries(data.swarmContexts)) {
            this.swarmContexts.set(id, context as SwarmContext)
          }
        }
        if (data.sharedSessions) {
          for (const [id, session] of Object.entries(data.sharedSessions)) {
            this.sharedSessions.set(id, session as SharedSession)
          }
        }
        console.log('[SharedContextManager] 已从本地存储恢复')
      }
    } catch (error) {
      console.error('[SharedContextManager] 恢复失败:', error)
    }
  }

  /**
   * 保存到本地存储
   */
  private async saveToStorage(): Promise<void> {
    try {
      const data = {
        swarmContexts: Object.fromEntries(this.swarmContexts),
        sharedSessions: Object.fromEntries(this.sharedSessions),
      }
      localStorage.setItem('shared_context_manager', JSON.stringify(data))
    } catch (error) {
      console.error('[SharedContextManager] 保存失败:', error)
    }
  }

  // ============================================================================
  // 智囊团上下文管理
  // ============================================================================

  /**
   * 创建智囊团上下文
   */
  async createSwarmContext(
    name: string,
    members: AgentInfo[],
    description?: string
  ): Promise<SharedContextResult<SwarmContext>> {
    const startTime = Date.now()
    try {
      const swarmId = generateId('swarm')
      const now = Date.now()

      const swarmContext: SwarmContext = {
        id: swarmId,
        name,
        description,
        members,
        contextIndexes: [],
        sharedMemories: [],
        createdAt: now,
        lastActivityAt: now,
      }

      this.swarmContexts.set(swarmId, swarmContext)
      await this.saveToStorage()

      this.emit('swarm_created', { swarmContext })

      return {
        success: true,
        data: swarmContext,
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 获取智囊团上下文
   */
  getSwarmContext(swarmId: string): SwarmContext | null {
    return this.swarmContexts.get(swarmId) || null
  }

  /**
   * 获取所有智囊团上下文
   */
  getAllSwarmContexts(): SwarmContext[] {
    return Array.from(this.swarmContexts.values())
  }

  /**
   * 更新智囊团上下文
   */
  async updateSwarmContext(
    swarmId: string,
    updates: Partial<SwarmContext>
  ): Promise<SharedContextResult<SwarmContext>> {
    const startTime = Date.now()
    try {
      const swarmContext = this.swarmContexts.get(swarmId)
      if (!swarmContext) {
        return { success: false, error: '智囊团不存在' }
      }

      const updated: SwarmContext = {
        ...swarmContext,
        ...updates,
        id: swarmId,
        lastActivityAt: Date.now(),
      }

      this.swarmContexts.set(swarmId, updated)
      await this.saveToStorage()

      this.emit('swarm_updated', { swarmContext: updated })

      return {
        success: true,
        data: updated,
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 删除智囊团上下文
   */
  async deleteSwarmContext(swarmId: string): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const deleted = this.swarmContexts.delete(swarmId)
      if (deleted) {
        this.contextIndexes.delete(swarmId)
        await this.saveToStorage()
        this.emit('swarm_deleted', { swarmId })
      }

      return {
        success: deleted,
        error: deleted ? undefined : '智囊团不存在',
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 添加成员到智囊团
   */
  async addMemberToSwarm(
    swarmId: string,
    member: AgentInfo
  ): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const swarmContext = this.swarmContexts.get(swarmId)
      if (!swarmContext) {
        return { success: false, error: '智囊团不存在' }
      }

      if (!swarmContext.members.find(m => m.id === member.id)) {
        swarmContext.members.push(member)
        swarmContext.lastActivityAt = Date.now()
        await this.saveToStorage()
        this.emit('member_added', { swarmId, member })
      }

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  // ============================================================================
  // 上下文条目管理
  // ============================================================================

  /**
   * 创建上下文条目
   */
  async createContextEntry(
    type: ContextEntryType,
    content: ContextContent,
    creator: AgentInfo,
    options: {
      swarmId?: string
      tags?: string[]
      shared?: boolean
      permissions?: ContextPermissions
      ipfsCid?: string
    } = {}
  ): Promise<SharedContextResult<ContextEntry>> {
    const startTime = Date.now()
    try {
      const entryId = generateId('ctx')
      const now = Date.now()

      const entry: ContextEntry = {
        id: entryId,
        index: '',
        type,
        content,
        ref: {
          type: options.ipfsCid ? ContextRefType.CID : ContextRefType.SESSION,
          value: options.ipfsCid || generateId('ref'),
          createdAt: now,
        },
        swarmId: options.swarmId,
        creator,
        createdAt: now,
        updatedAt: now,
        version: 1,
        tags: options.tags || [],
        shared: options.shared || false,
        permissions: options.permissions,
      }

      entry.index = generateContextIndex(entry)

      this.localCache.set(entryId, entry)

      if (options.swarmId) {
        const swarmContext = this.swarmContexts.get(options.swarmId)
        if (swarmContext) {
          swarmContext.sharedMemories.push(entry)
          swarmContext.lastActivityAt = now
        }
      }

      this.updateIndexes(entry)
      await this.saveToStorage()

      if (options.ipfsCid) {
        await this.storeToIpfs(entry)
      }

      this.emit('entry_created', { entry })

      return {
        success: true,
        data: entry,
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 获取上下文条目
   */
  getContextEntry(entryId: string): ContextEntry | null {
    return this.localCache.get(entryId) || null
  }

  /**
   * 更新上下文条目
   */
  async updateContextEntry(
    entryId: string,
    updates: Partial<ContextEntry>
  ): Promise<SharedContextResult<ContextEntry>> {
    const startTime = Date.now()
    try {
      const entry = this.localCache.get(entryId)
      if (!entry) {
        return { success: false, error: '上下文条目不存在' }
      }

      const updated: ContextEntry = {
        ...entry,
        ...updates,
        id: entryId,
        updatedAt: Date.now(),
        version: entry.version + 1,
      }

      updated.index = generateContextIndex(updated)
      updated.ref = {
        ...entry.ref,
        value: updates.content ? generateId('ref') : entry.ref.value,
        createdAt: Date.now(),
      }

      this.localCache.set(entryId, updated)

      this.updateIndexes(updated)
      await this.saveToStorage()

      this.emit('entry_updated', { entry: updated })

      return {
        success: true,
        data: updated,
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 删除上下文条目
   */
  async deleteContextEntry(entryId: string): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const entry = this.localCache.get(entryId)
      if (!entry) {
        return { success: false, error: '上下文条目不存在' }
      }

      this.localCache.delete(entryId)

      if (entry.swarmId) {
        const swarmContext = this.swarmContexts.get(entry.swarmId)
        if (swarmContext) {
          swarmContext.sharedMemories = swarmContext.sharedMemories.filter(
            e => e.id !== entryId
          )
        }
      }

      this.removeFromIndexes(entryId)
      await this.ipfsStorage.delete(entry.cid)

      this.emit('entry_deleted', { entryId })

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 查询上下文条目
   */
  queryContextEntries(options: ContextQueryOptions = {}): ContextEntry[] {
    let results = Array.from(this.localCache.values())

    if (options.swarmId) {
      results = results.filter(e => e.swarmId === options.swarmId)
    }

    if (options.type) {
      results = results.filter(e => e.type === options.type)
    }

    if (options.tags && options.tags.length > 0) {
      results = results.filter(e =>
        options.tags!.some(tag => e.tags.includes(tag))
      )
    }

    if (options.creatorId) {
      results = results.filter(e => e.creator.id === options.creatorId)
    }

    if (options.timeRange) {
      results = results.filter(
        e =>
          e.createdAt >= options.timeRange!.start &&
          e.createdAt <= options.timeRange!.end
      )
    }

    if (options.keyword) {
      const keyword = options.keyword.toLowerCase()
      results = results.filter(e => {
        const text = e.content.text?.toLowerCase() || ''
        return text.includes(keyword)
      })
    }

    if (options.sort) {
      results.sort((a, b) => {
        const aVal = (a as any)[options.sort!.field]
        const bVal = (b as any)[options.sort!.field]
        return options.sort!.order === 'asc'
          ? aVal > bVal ? 1 : -1
          : aVal < bVal ? 1 : -1
      })
    }

    if (options.pagination) {
      const { offset, limit } = options.pagination
      results = results.slice(offset, offset + limit)
    }

    return results
  }

  // ============================================================================
  // 索引管理
  // ============================================================================

  /**
   * 更新索引
   */
  private updateIndexes(entry: ContextEntry): void {
    const indexes: ContextIndex[] = []

    for (const tag of entry.tags) {
      indexes.push({
        id: generateId('idx'),
        name: `tag_${tag}`,
        type: ContextIndexType.TAG,
        key: 'tag',
        value: tag,
        entryIds: [entry.id],
        createdAt: Date.now(),
        weight: 1,
      })
    }

    indexes.push({
      id: generateId('idx'),
      name: `type_${entry.type}`,
      type: ContextIndexType.TYPE,
      key: 'type',
      value: entry.type,
      entryIds: [entry.id],
      createdAt: Date.now(),
      weight: 1,
    })

    indexes.push({
      id: generateId('idx'),
      name: `time_${Math.floor(entry.createdAt / 60000)}`,
      type: ContextIndexType.TIME,
      key: 'timestamp',
      value: String(entry.createdAt),
      entryIds: [entry.id],
      createdAt: Date.now(),
      weight: 1,
    })

    if (entry.creator.id) {
      indexes.push({
        id: generateId('idx'),
        name: `creator_${entry.creator.id}`,
        type: ContextIndexType.CREATOR,
        key: 'creator',
        value: entry.creator.id,
        entryIds: [entry.id],
        createdAt: Date.now(),
        weight: 1,
      })
    }

    if (entry.swarmId) {
      indexes.push({
        id: generateId('idx'),
        name: `swarm_${entry.swarmId}`,
        type: ContextIndexType.SWARM,
        key: 'swarmId',
        value: entry.swarmId,
        entryIds: [entry.id],
        createdAt: Date.now(),
        weight: 1,
      })
    }

    const existingIndexes = this.contextIndexes.get(entry.index) || []
    this.contextIndexes.set(entry.index, [...existingIndexes, ...indexes])
  }

  /**
   * 从索引中移除
   */
  private removeFromIndexes(entryId: string): void {
    for (const [indexKey, indexes] of this.contextIndexes.entries()) {
      const filtered = indexes.filter(idx => !idx.entryIds.includes(entryId))
      if (filtered.length > 0) {
        this.contextIndexes.set(indexKey, filtered)
      } else {
        this.contextIndexes.delete(indexKey)
      }
    }
  }

  /**
   * 通过索引查找
   */
  findByIndex(type: ContextIndexType, value: string): ContextEntry[] {
    const entries: ContextEntry[] = []

    for (const [indexKey, indexes] of this.contextIndexes.entries()) {
      const matchingIndex = indexes.find(
        idx => idx.type === type && idx.value === value
      )
      if (matchingIndex) {
        for (const entryId of matchingIndex.entryIds) {
          const entry = this.localCache.get(entryId)
          if (entry) {
            entries.push(entry)
          }
        }
      }
    }

    return entries
  }

  /**
   * 获取相关上下文
   */
  getRelatedContexts(entryId: string, limit: number = 5): ContextEntry[] {
    const entry = this.localCache.get(entryId)
    if (!entry) return []

    const related: ContextEntry[] = []

    for (const tag of entry.tags) {
      const tagged = this.findByIndex(ContextIndexType.TAG, tag)
      related.push(...tagged.filter(e => e.id !== entryId))
    }

    if (entry.swarmId) {
      const swarmEntries = this.queryContextEntries({
        swarmId: entry.swarmId,
      })
      related.push(...swarmEntries.filter(e => e.id !== entryId))
    }

    return related.slice(0, limit)
  }

  // ============================================================================
  // IPFS 存储
  // ============================================================================

  /**
   * 存储到 IPFS
   */
  private async storeToIpfs(entry: ContextEntry): Promise<string | null> {
    try {
      const data = JSON.stringify(entry)
      const result = await invoke<string>('add_ipfs_data', { data })

      if (result) {
        const storedEntry: IpfsStoredEntry = {
          cid: result,
          entry,
          storedAt: Date.now(),
          pinned: false,
        }
        this.ipfsStorage.set(entry.id, storedEntry)

        await invoke('pin_cid', { cid: result })

        return result
      }

      return null
    } catch (error) {
      console.error('[SharedContextManager] IPFS 存储失败:', error)
      return null
    }
  }

  /**
   * 从 IPFS 加载
   */
  async loadFromIpfs(cid: string): Promise<ContextEntry | null> {
    try {
      const cached = Array.from(this.ipfsStorage.values()).find(
        s => s.cid === cid
      )
      if (cached) {
        return cached.entry
      }

      const data = await invoke<string>('get_ipfs_data', { cid })
      if (data) {
        const entry = JSON.parse(data) as ContextEntry
        if (isContextEntry(entry)) {
          this.localCache.set(entry.id, entry)
          return entry
        }
      }

      return null
    } catch (error) {
      console.error('[SharedContextManager] IPFS 加载失败:', error)
      return null
    }
  }

  /**
   * 发布到 IPNS
   */
  async publishToIpns(entryId: string): Promise<string | null> {
    try {
      const entry = this.localCache.get(entryId)
      if (!entry || !entry.ref.value) return null

      const ipnsName = await invoke<string>('publish_ipns', {
        cid: entry.ref.value,
        key: `context_${entryId}`,
      })

      if (ipnsName) {
        entry.ref.type = ContextRefType.IPNS
        entry.ref.value = ipnsName
        await this.saveToStorage()
        return ipnsName
      }

      return null
    } catch (error) {
      console.error('[SharedContextManager] IPNS 发布失败:', error)
      return null
    }
  }

  // ============================================================================
  // 共享 Session 管理
  // ============================================================================

  /**
   * 创建共享 Session
   */
  async createSharedSession(
    name: string,
    creator: AgentInfo,
    swarmIds: string[] = [],
    config?: Partial<SharedSessionConfig>
  ): Promise<SharedContextResult<SharedSession>> {
    const startTime = Date.now()
    try {
      const sessionId = generateId('session')
      const now = Date.now()

      const session: SharedSession = {
        id: sessionId,
        name,
        swarms: swarmIds,
        contextRefs: [],
        state: {
          active: true,
          participants: [creator],
          syncStatus: SyncStatus.SYNCED,
          conflicts: [],
        },
        creator,
        createdAt: now,
        lastSyncAt: now,
        config: {
          ...this.defaultConfig,
          ...config,
        },
      }

      this.sharedSessions.set(sessionId, session)

      for (const swarmId of swarmIds) {
        const swarm = this.swarmContexts.get(swarmId)
        if (swarm) {
          swarm.lastActivityAt = now
        }
      }

      await this.saveToStorage()
      this.emit('session_created', { session })

      return {
        success: true,
        data: session,
        duration: Date.now() - startTime,
      }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 获取共享 Session
   */
  getSharedSession(sessionId: string): SharedSession | null {
    return this.sharedSessions.get(sessionId) || null
  }

  /**
   * 获取所有共享 Session
   */
  getAllSharedSessions(): SharedSession[] {
    return Array.from(this.sharedSessions.values())
  }

  /**
   * 加入共享 Session
   */
  async joinSharedSession(
    sessionId: string,
    agent: AgentInfo
  ): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const session = this.sharedSessions.get(sessionId)
      if (!session) {
        return { success: false, error: 'Session 不存在' }
      }

      if (!session.state.participants.find(p => p.id === agent.id)) {
        session.state.participants.push(agent)
        session.state.active = true
        await this.saveToStorage()
        this.emit('participant_joined', { sessionId, agent })
      }

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 离开共享 Session
   */
  async leaveSharedSession(
    sessionId: string,
    agentId: string
  ): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const session = this.sharedSessions.get(sessionId)
      if (!session) {
        return { success: false, error: 'Session 不存在' }
      }

      session.state.participants = session.state.participants.filter(
        p => p.id !== agentId
      )

      if (session.state.participants.length === 0) {
        session.state.active = false
      }

      await this.saveToStorage()
      this.emit('participant_left', { sessionId, agentId })

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 添加上下文引用到 Session
   */
  async addContextRefToSession(
    sessionId: string,
    ref: ContextRef
  ): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const session = this.sharedSessions.get(sessionId)
      if (!session) {
        return { success: false, error: 'Session 不存在' }
      }

      session.contextRefs.push(ref)
      session.lastSyncAt = Date.now()
      await this.saveToStorage()

      this.emit('context_ref_added', { sessionId, ref })

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  /**
   * 同步 Session
   */
  async syncSession(sessionId: string): Promise<SharedContextResult> {
    const startTime = Date.now()
    try {
      const session = this.sharedSessions.get(sessionId)
      if (!session) {
        return { success: false, error: 'Session 不存在' }
      }

      session.state.syncStatus = SyncStatus.SYNCING

      for (const ref of session.contextRefs) {
        if (ref.type === ContextRefType.CID && ref.value) {
          await this.loadFromIpfs(ref.value)
        }
      }

      session.state.syncStatus = SyncStatus.SYNCED
      session.lastSyncAt = Date.now()
      await this.saveToStorage()

      this.emit('session_synced', { sessionId })

      return { success: true, duration: Date.now() - startTime }
    } catch (error) {
      return {
        success: false,
        error: String(error),
        duration: Date.now() - startTime,
      }
    }
  }

  // ============================================================================
  // 记忆共享
  // ============================================================================

  /**
   * 分享记忆到智囊团
   */
  async shareMemoryToSwarm(
    swarmId: string,
    memoryContent: string,
    creator: AgentInfo,
    tags: string[] = []
  ): Promise<SharedContextResult<ContextEntry>> {
    return this.createContextEntry(
      ContextEntryType.MEMORY,
      { text: memoryContent },
      creator,
      { swarmId, tags, shared: true }
    )
  }

  /**
   * 获取智囊团共享记忆
   */
  getSwarmSharedMemories(swarmId: string): ContextEntry[] {
    const swarm = this.swarmContexts.get(swarmId)
    return swarm?.sharedMemories.filter(e => e.type === ContextEntryType.MEMORY) || []
  }

  /**
   * 创建对话摘要
   */
  async createConversationSummary(
    swarmId: string,
    messages: GroupChatMessage[],
    creator: AgentInfo
  ): Promise<SharedContextResult<ContextEntry>> {
    const summaryText = messages
      .map(m => `[${m.fromName}]: ${m.content}`)
      .join('\n')

    return this.createContextEntry(
      ContextEntryType.SUMMARY,
      {
        text: summaryText,
        messages,
      },
      creator,
      { swarmId, tags: ['summary'] }
    )
  }

  // ============================================================================
  // 事件系统
  // ============================================================================

  /**
   * 添加事件监听器
   */
  addEventListener(
    event: string,
    callback: (event: SharedContextEvent) => void
  ): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set())
    }
    this.listeners.get(event)!.add(callback)
  }

  /**
   * 移除事件监听器
   */
  removeEventListener(
    event: string,
    callback: (event: SharedContextEvent) => void
  ): void {
    this.listeners.get(event)?.delete(callback)
  }

  /**
   * 触发事件
   */
  private emit(eventType: string, data: Record<string, unknown>): void {
    const event: SharedContextEvent = {
      type: eventType,
      data,
      timestamp: Date.now(),
    }

    this.listeners.get(eventType)?.forEach(cb => {
      try {
        cb(event)
      } catch (error) {
        console.error('[SharedContextManager] 事件处理错误:', error)
      }
    })

    this.listeners.get('*')?.forEach(cb => {
      try {
        cb(event)
      } catch (error) {
        console.error('[SharedContextManager] 全局事件处理错误:', error)
      }
    })
  }

  // ============================================================================
  // 统计和状态
  // ============================================================================

  /**
   * 获取统计信息
   */
  getStats(): SharedContextStats {
    let ipfsEntries = 0
    let totalSize = 0

    for (const stored of this.ipfsStorage.values()) {
      if (stored.pinned) ipfsEntries++
      totalSize += JSON.stringify(stored.entry).length
    }

    return {
      totalEntries: this.localCache.size,
      sharedEntries: Array.from(this.localCache.values()).filter(e => e.shared)
        .length,
      swarmCount: this.swarmContexts.size,
      ipfsEntries,
      totalSize,
      lastSyncAt: Math.max(
        ...Array.from(this.sharedSessions.values()).map(s => s.lastSyncAt),
        0
      ),
    }
  }

  /**
   * 销毁管理器
   */
  destroy(): void {
    this.localCache.clear()
    this.swarmContexts.clear()
    this.sharedSessions.clear()
    this.contextIndexes.clear()
    this.ipfsStorage.clear()
    this.listeners.clear()
  }
}

/**
 * 共享上下文事件
 */
export interface SharedContextEvent {
  type: string
  data: Record<string, unknown>
  timestamp: number
}

// 导出单例
export const sharedContextManager = SharedContextManager.getInstance()

// 导出便捷函数
export function getSharedContextManager(): SharedContextManager {
  return sharedContextManager
}

export function getSwarmContext(swarmId: string): SwarmContext | null {
  return sharedContextManager.getSwarmContext(swarmId)
}

export function getAllSwarmContexts(): SwarmContext[] {
  return sharedContextManager.getAllSwarmContexts()
}

export function getSharedSession(sessionId: string): SharedSession | null {
  return sharedContextManager.getSharedSession(sessionId)
}

export function getAllSharedSessions(): SharedSession[] {
  return sharedContextManager.getAllSharedSessions()
}

export function createSwarmContext(
  name: string,
  members: AgentInfo[],
  description?: string
): Promise<SharedContextResult<SwarmContext>> {
  return sharedContextManager.createSwarmContext(name, members, description)
}

export function shareMemoryToSwarm(
  swarmId: string,
  memoryContent: string,
  creator: AgentInfo,
  tags?: string[]
): Promise<SharedContextResult<ContextEntry>> {
  return sharedContextManager.shareMemoryToSwarm(swarmId, memoryContent, creator, tags)
}

export function queryContextEntries(
  options?: ContextQueryOptions
): ContextEntry[] {
  return sharedContextManager.queryContextEntries(options)
}