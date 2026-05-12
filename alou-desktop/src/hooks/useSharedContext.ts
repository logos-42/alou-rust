/**
 * useSharedContext - 共享上下文 React Hook
 *
 * 提供共享上下文的 React 接口
 *
 * @module hooks/useSharedContext
 */

import { useState, useEffect, useCallback } from 'react'
import {
  SharedContextManager,
  sharedContextManager,
  SharedContextEvent,
} from '../services/sharedContextManager'
import {
  SwarmContext,
  SharedSession,
  ContextEntry,
  ContextEntryType,
  SharedContextStats,
  ContextQueryOptions,
  SharedContextResult,
  AgentInfo,
} from '../types/sharedContext'
import { GroupChatMessage } from '../types/groupchat'

/**
 * 共享上下文 Hook 返回类型
 */
export interface UseSharedContextReturn {
  /** 智囊团上下文列表 */
  swarms: SwarmContext[]
  /** 共享 Session 列表 */
  sessions: SharedSession[]
  /** 统计信息 */
  stats: SharedContextStats
  /** 加载状态 */
  loading: boolean
  /** 创建智囊团 */
  createSwarm: (
    name: string,
    members: AgentInfo[],
    description?: string
  ) => Promise<SharedContextResult<SwarmContext>>
  /** 获取智囊团 */
  getSwarm: (swarmId: string) => SwarmContext | null
  /** 更新智囊团 */
  updateSwarm: (
    swarmId: string,
    updates: Partial<SwarmContext>
  ) => Promise<SharedContextResult<SwarmContext>>
  /** 删除智囊团 */
  deleteSwarm: (swarmId: string) => Promise<SharedContextResult>
  /** 添加成员 */
  addMember: (
    swarmId: string,
    member: AgentInfo
  ) => Promise<SharedContextResult>
  /** 创建共享 Session */
  createSession: (
    name: string,
    creator: AgentInfo,
    swarmIds?: string[]
  ) => Promise<SharedContextResult<SharedSession>>
  /** 加入 Session */
  joinSession: (
    sessionId: string,
    agent: AgentInfo
  ) => Promise<SharedContextResult>
  /** 离开 Session */
  leaveSession: (
    sessionId: string,
    agentId: string
  ) => Promise<SharedContextResult>
  /** 同步 Session */
  syncSession: (sessionId: string) => Promise<SharedContextResult>
  /** 分享记忆 */
  shareMemory: (
    swarmId: string,
    content: string,
    creator: AgentInfo,
    tags?: string[]
  ) => Promise<SharedContextResult<ContextEntry>>
  /** 创建对话摘要 */
  createSummary: (
    swarmId: string,
    messages: GroupChatMessage[],
    creator: AgentInfo
  ) => Promise<SharedContextResult<ContextEntry>>
  /** 查询上下文 */
  queryEntries: (options?: ContextQueryOptions) => ContextEntry[]
  /** 获取上下文条目 */
  getEntry: (entryId: string) => ContextEntry | null
  /** 更新条目 */
  updateEntry: (
    entryId: string,
    updates: Partial<ContextEntry>
  ) => Promise<SharedContextResult<ContextEntry>>
  /** 删除条目 */
  deleteEntry: (entryId: string) => Promise<SharedContextResult>
  /** 获取智囊团记忆 */
  getSwarmMemories: (swarmId: string) => ContextEntry[]
  /** 刷新数据 */
  refresh: () => void
}

/**
 * 共享上下文 Hook
 */
export function useSharedContext(): UseSharedContextReturn {
  const [swarms, setSwarms] = useState<SwarmContext[]>([])
  const [sessions, setSessions] = useState<SharedSession[]>([])
  const [stats, setStats] = useState<SharedContextStats>({
    totalEntries: 0,
    sharedEntries: 0,
    swarmCount: 0,
    ipfsEntries: 0,
    totalSize: 0,
    lastSyncAt: 0,
  })
  const [loading, setLoading] = useState(false)

  /**
   * 刷新数据
   */
  const refresh = useCallback(() => {
    setSwarms(sharedContextManager.getAllSwarmContexts())
    setSessions(sharedContextManager.getAllSharedSessions())
    setStats(sharedContextManager.getStats())
  }, [])

  /**
   * 处理事件
   */
  const handleEvent = useCallback((event: SharedContextEvent) => {
    console.log('[useSharedContext] 收到事件:', event.type)
    refresh()
  }, [refresh])

  useEffect(() => {
    refresh()

    sharedContextManager.addEventListener('*', handleEvent)

    return () => {
      sharedContextManager.removeEventListener('*', handleEvent)
    }
  }, [handleEvent, refresh])

  /**
   * 创建智囊团
   */
  const createSwarm = useCallback(
    async (
      name: string,
      members: AgentInfo[],
      description?: string
    ): Promise<SharedContextResult<SwarmContext>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.createSwarmContext(
          name,
          members,
          description
        )
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 获取智囊团
   */
  const getSwarm = useCallback((swarmId: string): SwarmContext | null => {
    return sharedContextManager.getSwarmContext(swarmId)
  }, [])

  /**
   * 更新智囊团
   */
  const updateSwarm = useCallback(
    async (
      swarmId: string,
      updates: Partial<SwarmContext>
    ): Promise<SharedContextResult<SwarmContext>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.updateSwarmContext(
          swarmId,
          updates
        )
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 删除智囊团
   */
  const deleteSwarm = useCallback(
    async (swarmId: string): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.deleteSwarmContext(swarmId)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 添加成员
   */
  const addMember = useCallback(
    async (swarmId: string, member: AgentInfo): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.addMemberToSwarm(swarmId, member)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 创建 Session
   */
  const createSession = useCallback(
    async (
      name: string,
      creator: AgentInfo,
      swarmIds: string[] = []
    ): Promise<SharedContextResult<SharedSession>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.createSharedSession(
          name,
          creator,
          swarmIds
        )
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 加入 Session
   */
  const joinSession = useCallback(
    async (sessionId: string, agent: AgentInfo): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.joinSharedSession(sessionId, agent)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 离开 Session
   */
  const leaveSession = useCallback(
    async (sessionId: string, agentId: string): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.leaveSharedSession(sessionId, agentId)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 同步 Session
   */
  const syncSession = useCallback(
    async (sessionId: string): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.syncSession(sessionId)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 分享记忆
   */
  const shareMemory = useCallback(
    async (
      swarmId: string,
      content: string,
      creator: AgentInfo,
      tags: string[] = []
    ): Promise<SharedContextResult<ContextEntry>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.shareMemoryToSwarm(
          swarmId,
          content,
          creator,
          tags
        )
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 创建对话摘要
   */
  const createSummary = useCallback(
    async (
      swarmId: string,
      messages: GroupChatMessage[],
      creator: AgentInfo
    ): Promise<SharedContextResult<ContextEntry>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.createConversationSummary(
          swarmId,
          messages,
          creator
        )
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 查询上下文
   */
  const queryEntries = useCallback(
    (options?: ContextQueryOptions): ContextEntry[] => {
      return sharedContextManager.queryContextEntries(options)
    },
    []
  )

  /**
   * 获取上下文条目
   */
  const getEntry = useCallback((entryId: string): ContextEntry | null => {
    return sharedContextManager.getContextEntry(entryId)
  }, [])

  /**
   * 更新条目
   */
  const updateEntry = useCallback(
    async (
      entryId: string,
      updates: Partial<ContextEntry>
    ): Promise<SharedContextResult<ContextEntry>> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.updateContextEntry(entryId, updates)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 删除条目
   */
  const deleteEntry = useCallback(
    async (entryId: string): Promise<SharedContextResult> => {
      setLoading(true)
      try {
        const result = await sharedContextManager.deleteContextEntry(entryId)
        refresh()
        return result
      } finally {
        setLoading(false)
      }
    },
    [refresh]
  )

  /**
   * 获取智囊团记忆
   */
  const getSwarmMemories = useCallback((swarmId: string): ContextEntry[] => {
    return sharedContextManager.getSwarmSharedMemories(swarmId)
  }, [])

  return {
    swarms,
    sessions,
    stats,
    loading,
    createSwarm,
    getSwarm,
    updateSwarm,
    deleteSwarm,
    addMember,
    createSession,
    joinSession,
    leaveSession,
    syncSession,
    shareMemory,
    createSummary,
    queryEntries,
    getEntry,
    updateEntry,
    deleteEntry,
    getSwarmMemories,
    refresh,
  }
}

/**
 * 单个智囊团上下文 Hook
 */
export function useSwarmContext(swarmId: string | null) {
  const { swarms, ...rest } = useSharedContext()

  const swarm = swarmId ? swarms.find(s => s.id === swarmId) || null : null

  return {
    swarm,
    ...rest,
  }
}

/**
 * 单个 Session Hook
 */
export function useSharedSession(sessionId: string | null) {
  const { sessions, ...rest } = useSharedContext()

  const session = sessionId
    ? sessions.find(s => s.id === sessionId) || null
    : null

  return {
    session,
    ...rest,
  }
}

export default useSharedContext