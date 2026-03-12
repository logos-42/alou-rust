/**
 * Session 隔离的消息队列管理器
 *
 * 为每个 session 创建独立的消息队列，实现：
 * - Session 隔离：不同 session 的消息互不影响
 * - 并发处理：同一个 session 内的消息可以并发处理
 * - 异步执行：fire-and-forget 模式，不阻塞 UI
 * - 优先级控制：支持消息优先级排序
 */

export interface QueueMessage {
  id: string
  agentId: string
  content: string
  groupId?: string
  isGroupChat?: boolean
  originalMessage?: string
  sessionId: string
  timestamp: number
  priority?: number
  retryCount?: number
  maxRetries?: number
}

export interface QueueConfig {
  maxConcurrent?: number
  maxQueueSize?: number
  retryDelay?: number
  messageTimeout?: number
}

export type MessageHandler = (message: QueueMessage) => Promise<void>

class SessionQueue {
  private sessionId: string
  private queue: QueueMessage[] = []
  private processing: Map<string, Promise<void>> = new Map()
  private handler: MessageHandler | null = null
  private config: QueueConfig
  private isProcessing: boolean = false

  constructor(sessionId: string, config: QueueConfig = {}) {
    this.sessionId = sessionId
    this.config = {
      maxConcurrent: 5,
      maxQueueSize: 100,
      retryDelay: 1000,
      messageTimeout: 60000,
      ...config,
    }
  }

  setHandler(handler: MessageHandler): void {
    this.handler = handler
  }

  enqueue(message: QueueMessage): boolean {
    if (this.queue.length >= this.config.maxQueueSize!) {
      console.warn('[SessionQueue] 队列已满:', { sessionId: this.sessionId, agentId: message.agentId })
      return false
    }

    const insertIndex = this.queue.findIndex(
      item => (item.priority ?? 10) > (message.priority ?? 10)
    )

    if (insertIndex === -1) {
      this.queue.push(message)
    } else {
      this.queue.splice(insertIndex, 0, message)
    }

    console.log('[SessionQueue] 消息入队:', { sessionId: this.sessionId, queueLength: this.queue.length })
    this.processQueue()
    return true
  }

  private async processQueue(): Promise<void> {
    if (this.isProcessing || !this.handler) return
    this.isProcessing = true

    try {
      while (this.queue.length > 0 && this.processing.size < this.config.maxConcurrent!) {
        const message = this.queue.shift()
        if (!message) break

        const promise = this.processMessage(message)
        this.processing.set(message.id, promise)

        promise.finally(() => {
          this.processing.delete(message.id)
          this.processQueue()
        })
      }
    } catch (error) {
      console.error('[SessionQueue] 队列处理错误:', error)
    } finally {
      this.isProcessing = false
    }
  }

  private async processMessage(message: QueueMessage): Promise<void> {
    if (!this.handler) return

    console.log('[SessionQueue] 处理消息:', { sessionId: this.sessionId, agentId: message.agentId })

    try {
      await this.handler(message)
    } catch (error) {
      console.error('[SessionQueue] 消息处理失败:', error)
      const retryCount = message.retryCount ?? 0
      const maxRetries = message.maxRetries ?? 3
      if (retryCount < maxRetries) {
        await new Promise(resolve => setTimeout(resolve, this.config.retryDelay))
        message.retryCount = retryCount + 1
        this.enqueue(message)
      }
    }
  }

  getStatus() {
    return {
      queueLength: this.queue.length,
      processingCount: this.processing.size,
      isProcessing: this.isProcessing,
    }
  }

  clear(): void {
    this.queue = []
  }

  destroy(): void {
    this.clear()
    this.handler = null
  }
}

class SessionMessageQueueManager {
  private static instance: SessionMessageQueueManager | null = null
  private sessionQueues: Map<string, SessionQueue> = new Map()
  private defaultConfig: QueueConfig

  private constructor(config: QueueConfig = {}) {
    this.defaultConfig = config
  }

  static getInstance(config?: QueueConfig): SessionMessageQueueManager {
    if (!this.instance) {
      this.instance = new SessionMessageQueueManager(config)
    }
    return this.instance
  }

  static resetInstance(): void {
    if (this.instance) {
      this.instance.destroy()
      this.instance = null
    }
  }

  getOrCreateQueue(sessionId: string, config?: QueueConfig): SessionQueue {
    let queue = this.sessionQueues.get(sessionId)
    if (!queue) {
      queue = new SessionQueue(sessionId, { ...this.defaultConfig, ...config })
      this.sessionQueues.set(sessionId, queue)
    }
    return queue
  }

  setHandler(sessionId: string, handler: MessageHandler): void {
    this.getOrCreateQueue(sessionId).setHandler(handler)
  }

  enqueue(message: QueueMessage): boolean {
    return this.getOrCreateQueue(message.sessionId).enqueue(message)
  }

  getQueueStatus(sessionId: string) {
    const queue = this.sessionQueues.get(sessionId)
    return queue ? queue.getStatus() : null
  }

  getAllQueueStatus(): Map<string, any> {
    const status = new Map()
    for (const [sessionId, queue] of this.sessionQueues.entries()) {
      status.set(sessionId, queue.getStatus())
    }
    return status
  }

  destroyQueue(sessionId: string): void {
    const queue = this.sessionQueues.get(sessionId)
    if (queue) {
      queue.destroy()
      this.sessionQueues.delete(sessionId)
    }
  }

  destroy(): void {
    for (const queue of this.sessionQueues.values()) {
      queue.destroy()
    }
    this.sessionQueues.clear()
  }
}

export const sessionMessageQueue = SessionMessageQueueManager.getInstance()

export function enqueueMessage(
  agentId: string,
  content: string,
  sessionId: string,
  options?: { groupId?: string; isGroupChat?: boolean; originalMessage?: string; priority?: number }
): boolean {
  return sessionMessageQueue.enqueue({
    id: `msg_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
    agentId,
    content,
    sessionId,
    timestamp: Date.now(),
    priority: options?.priority ?? 10,
    groupId: options?.groupId,
    isGroupChat: options?.isGroupChat,
    originalMessage: options?.originalMessage,
  })
}

export function setSessionHandler(
  sessionId: string,
  handler: (message: QueueMessage) => Promise<void>
): void {
  sessionMessageQueue.setHandler(sessionId, handler)
}

export function getQueueStatus(sessionId?: string): any {
  if (sessionId) {
    return sessionMessageQueue.getQueueStatus(sessionId)
  }
  return Object.fromEntries(sessionMessageQueue.getAllQueueStatus())
}
