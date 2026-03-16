/**
 * Session 管理器 - 多页面并行的智能体协调器实例管理
 *
 * 实现 Session 实例隔离方案：
 * - 为每个 session 创建独立的 AgentCoordinator 实例
 * - 使用 Map<sessionId, Coordinator> 管理所有 session
 * - 提供统一的 API 获取/创建/销毁 session 协调器
 * - 使用 IndexedDB 进行持久化（大容量存储）
 *
 * @module services/sessionManager
 * @author Alou Team
 * @since 1.0.0
 */

import {
  UnifiedAgentCoordinator,
  AgentCoordinatorConfig,
  AgentMessageHandler,
} from './unifiedAgentCoordinator'

import { AgentInfo } from '../types/groupchat'

import * as sessionStorage from '@/utils/sessionStorage'
import { clearSessionData } from '@/stores/agentStore'

/**
 * Session 配置
 */
export interface SessionConfig extends AgentCoordinatorConfig {
  sessionId?: string
  description?: string
  createdAt?: number
  ttl?: number
}

/**
 * Session 信息
 */
export interface SessionInfo {
  sessionId: string
  description?: string
  createdAt: number
  lastActivityAt: number
  coordinator: UnifiedAgentCoordinator | null
  config: SessionConfig
  active: boolean
}

/**
 * Session 管理器单例类
 */
export class SessionManager {
  private static instance: SessionManager | null = null
  private sessions: Map<string, SessionInfo> = new Map()
  private cleanupTimer: NodeJS.Timeout | null = null
  private sessionEventListeners: Map<string, Array<() => void>> = new Map()

  private constructor() {
    this.startCleanupTimer()
    this.restoreSessions()
  }

  static getInstance(): SessionManager {
    if (!this.instance) {
      this.instance = new SessionManager()
    }
    return this.instance
  }

  static resetInstance(): void {
    if (this.instance) {
      this.instance.destroy()
      this.instance = null
    }
  }

  private startCleanupTimer(): void {
    this.cleanupTimer = setInterval(() => {
      this.cleanupInactiveSessions()
    }, 5 * 60 * 1000)

    if (typeof window !== 'undefined') {
      const handleBeforeUnload = () => this.destroy()
      window.addEventListener('beforeunload', handleBeforeUnload)
      this.sessionEventListeners.set('beforeunload', [handleBeforeUnload])
    }
  }

  private stopCleanupTimer(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer)
      this.cleanupTimer = null
    }
  }

  private async restoreSessions(): Promise<void> {
    try {
      // 1. 先尝试恢复最后一个 session（即使不是 active 状态）
      const lastSessionId = await sessionStorage.get<string>('last_session_id')
      if (lastSessionId) {
        const meta = await sessionStorage.get<SessionInfo>(`session_meta_${lastSessionId}`)
        if (meta) {
          console.log('[SessionManager] 恢复最后一个 session:', lastSessionId)
          this.sessions.set(lastSessionId, {
            ...meta,
            coordinator: null,
            active: true, // 强制标记为 active，以便恢复
          })
          return // 只恢复最后一个，不恢复其他的
        }
      }

      // 2. 如果没有 last_session_id，尝试恢复所有 active sessions（向后兼容）
      const sessionKeys = await sessionStorage.getAllKeys()
      const sessionIds = sessionKeys
        .filter(key => key.startsWith('session_meta_'))
        .map(key => key.replace('session_meta_', ''))

      for (const sessionId of sessionIds) {
        const meta = await sessionStorage.get<SessionInfo>(`session_meta_${sessionId}`)
        if (meta && meta.active) {
          const ttl = meta.config?.ttl || 30 * 60 * 1000
          if (Date.now() - meta.lastActivityAt < ttl) {
            console.log('[SessionManager] 恢复 session:', sessionId)
            this.sessions.set(sessionId, {
              ...meta,
              coordinator: null,
              active: true,
            })
          }
        }
      }
    } catch (error) {
      console.error('[SessionManager] 恢复 session 失败:', error)
    }
  }

  private async saveSessionMeta(sessionInfo: SessionInfo): Promise<void> {
    try {
      await sessionStorage.set(`session_meta_${sessionInfo.sessionId}`, {
        sessionId: sessionInfo.sessionId,
        description: sessionInfo.description,
        createdAt: sessionInfo.createdAt,
        lastActivityAt: sessionInfo.lastActivityAt,
        config: sessionInfo.config,
        active: sessionInfo.active,
      }, {
        sessionId: sessionInfo.sessionId,
        expiresAt: sessionInfo.lastActivityAt + (sessionInfo.config?.ttl || 30 * 60 * 1000),
      })

      // 保存最后一个 session 的 ID，用于重启后恢复
      await sessionStorage.set('last_session_id', sessionInfo.sessionId, {
        sessionId: sessionInfo.sessionId,
      })
    } catch (error) {
      console.error('[SessionManager] 保存 session 元数据失败:', error)
    }
  }

  private cleanupInactiveSessions(): void {
    const now = Date.now()
    
    for (const [sessionId, sessionInfo] of this.sessions.entries()) {
      const ttl = sessionInfo.config?.ttl || 30 * 60 * 1000
      
      if (now - sessionInfo.lastActivityAt > ttl) {
        console.log('[SessionManager] 清理过期 session:', sessionId)
        this.destroySession(sessionId, true)
      }
    }
    
    sessionStorage.cleanupExpired().then(count => {
      if (count > 0) {
        console.log('[SessionManager] 清理了', count, '条过期数据')
      }
    }).catch(console.error)
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
  }

  getOrCreateSession(config: SessionConfig = {}): SessionInfo {
    if (config.sessionId && this.sessions.has(config.sessionId)) {
      const sessionInfo = this.sessions.get(config.sessionId)!
      
      if (!sessionInfo.coordinator) {
        sessionInfo.coordinator = new UnifiedAgentCoordinator({
          ...config,
          sessionId: sessionInfo.sessionId,
        })
      }
      
      sessionInfo.lastActivityAt = Date.now()
      sessionInfo.active = true
      this.saveSessionMeta(sessionInfo)
      return sessionInfo
    }

    const sessionId = config.sessionId || this.generateSessionId()
    const now = Date.now()

    const coordinator = new UnifiedAgentCoordinator({
      autoReply: config.autoReply,
      replyDelay: config.replyDelay,
      debug: config.debug,
      autoSubscribe: config.autoSubscribe,
      onMessage: config.onMessage,
      restoreFromStore: config.restoreFromStore,
      sessionId,
    })

    const sessionInfo: SessionInfo = {
      sessionId,
      description: config.description,
      createdAt: config.createdAt || now,
      lastActivityAt: now,
      coordinator,
      config,
      active: true,
    }

    this.sessions.set(sessionId, sessionInfo)
    this.saveSessionMeta(sessionInfo)

    console.log('[SessionManager] 创建新 session:', {
      sessionId,
      description: config.description,
      totalSessions: this.sessions.size,
    })

    return sessionInfo
  }

  getSessionCoordinator(sessionId: string): UnifiedAgentCoordinator | null {
    const sessionInfo = this.sessions.get(sessionId)
    if (!sessionInfo || !sessionInfo.active) {
      return null
    }

    if (!sessionInfo.coordinator) {
      sessionInfo.coordinator = new UnifiedAgentCoordinator({
        sessionId,
      })
    }

    sessionInfo.lastActivityAt = Date.now()
    this.saveSessionMeta(sessionInfo)
    return sessionInfo.coordinator
  }

  getActiveSessions(): SessionInfo[] {
    return Array.from(this.sessions.values()).filter(s => s.active && s.coordinator !== null)
  }

  getSessionInfo(sessionId: string): SessionInfo | null {
    return this.sessions.get(sessionId) || null
  }

  async destroySession(sessionId: string, clearStorage: boolean = true): Promise<boolean> {
    const sessionInfo = this.sessions.get(sessionId)

    if (!sessionInfo) {
      console.warn('[SessionManager] Session 不存在:', sessionId)
      return false
    }

    console.log('[SessionManager] 销毁 session:', sessionId)

    if (sessionInfo.coordinator) {
      sessionInfo.coordinator.destroy()
    }

    const listeners = this.sessionEventListeners.get(sessionId)
    if (listeners) {
      listeners.forEach(cleanup => cleanup())
      this.sessionEventListeners.delete(sessionId)
    }

    this.sessions.delete(sessionId)

    if (clearStorage) {
      try {
        await clearSessionData(sessionId)
        await sessionStorage.deleteByKey(`session_meta_${sessionId}`)
      } catch (error) {
        console.error('[SessionManager] 清除 session 存储失败:', error)
      }
    }

    console.log('[SessionManager] Session 已销毁，剩余 session 数:', this.sessions.size)
    return true
  }

  async destroyAllSessions(): Promise<void> {
    console.log('[SessionManager] 销毁所有 session，总数:', this.sessions.size)
    
    // 保留最后一个 session 的元数据，不清理
    const sessionIds = Array.from(this.sessions.keys())
    const lastSessionId = sessionIds.length > 0 ? sessionIds[sessionIds.length - 1] : null
    
    const destroyPromises = sessionIds.map(sessionId =>
      // 最后一个 session 不清理存储，以便重启后恢复
      this.destroySession(sessionId, sessionId !== lastSessionId)
    )
    await Promise.all(destroyPromises)
    
    // 注意：this.sessions 不清空，保留最后一个 session 的引用
    if (lastSessionId) {
      const lastSession = this.sessions.get(lastSessionId)
      this.sessions.clear()
      if (lastSession) {
        this.sessions.set(lastSessionId, lastSession)
      }
    } else {
      this.sessions.clear()
    }
  }

  touchSession(sessionId: string): void {
    const sessionInfo = this.sessions.get(sessionId)
    if (sessionInfo) {
      sessionInfo.lastActivityAt = Date.now()
      this.saveSessionMeta(sessionInfo)
    }
  }

  getSessionCount(): number {
    return this.getActiveSessions().length
  }

  isSessionActive(sessionId: string): boolean {
    const sessionInfo = this.sessions.get(sessionId)
    return !!sessionInfo && sessionInfo.active && sessionInfo.coordinator !== null
  }

  async getSessionStats(): Promise<{
    totalSessions: number
    activeSessions: number
    totalAgents: number
    totalSubscribedGroups: number
    storageUsed: number
  }> {
    const activeSessions = this.getActiveSessions()
    let totalAgents = 0
    let totalSubscribedGroups = 0

    for (const session of activeSessions) {
      if (session.coordinator) {
        const status = session.coordinator.getStatus()
        totalAgents += status.registeredAgents
        totalSubscribedGroups += status.subscribedGroups
      }
    }

    let storageUsed = 0
    try {
      const stats = await sessionStorage.getStats()
      storageUsed = stats.totalSize
    } catch (error) {
      console.error('[SessionManager] 获取存储统计失败:', error)
    }

    return {
      totalSessions: this.sessions.size,
      activeSessions: activeSessions.length,
      totalAgents,
      totalSubscribedGroups,
      storageUsed,
    }
  }

  async registerAgentForSession(
    sessionId: string,
    agent: AgentInfo,
    handler?: AgentMessageHandler
  ): Promise<boolean> {
    const coordinator = this.getSessionCoordinator(sessionId)
    if (!coordinator) {
      console.error('[SessionManager] Session 不存在:', sessionId)
      return false
    }

    await coordinator.registerAgent(agent, handler)
    this.touchSession(sessionId)
    return true
  }

  async subscribeToGroupForSession(
    sessionId: string,
    groupId: string,
    mode?: GroupChatMode
  ): Promise<boolean> {
    const coordinator = this.getSessionCoordinator(sessionId)
    if (!coordinator) {
      console.error('[SessionManager] Session 不存在:', sessionId)
      return false
    }

    await coordinator.subscribeToGroup(groupId, mode)
    this.touchSession(sessionId)
    return true
  }

  unsubscribeFromGroupForSession(
    sessionId: string,
    groupId: string
  ): void {
    const coordinator = this.getSessionCoordinator(sessionId)
    if (!coordinator) {
      console.error('[SessionManager] Session 不存在:', sessionId)
      return
    }

    coordinator.unsubscribeFromGroup(groupId)
    this.touchSession(sessionId)
  }

  async destroy(): Promise<void> {
    console.log('[SessionManager] 销毁 SessionManager')
    this.stopCleanupTimer()
    await this.destroyAllSessions()

    const listeners = this.sessionEventListeners.get('beforeunload')
    if (listeners) {
      listeners.forEach(cleanup => {
        if (typeof window !== 'undefined') {
          window.removeEventListener('beforeunload', cleanup)
        }
      })
      this.sessionEventListeners.delete('beforeunload')
    }
  }
}

export const sessionManager = SessionManager.getInstance()

export function getOrCreateSession(config: SessionConfig = {}): SessionInfo {
  return sessionManager.getOrCreateSession(config)
}

export function getSessionCoordinator(sessionId: string): UnifiedAgentCoordinator | null {
  return sessionManager.getSessionCoordinator(sessionId)
}

export function getActiveSessions(): SessionInfo[] {
  return sessionManager.getActiveSessions()
}

export function destroySession(sessionId: string, clearStorage?: boolean): Promise<boolean> {
  return sessionManager.destroySession(sessionId, clearStorage)
}

export function getSessionStats(): Promise<ReturnType<typeof sessionManager.getSessionStats>> {
  return sessionManager.getSessionStats()
}
