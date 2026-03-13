/**
 * DIAP 异步身份创建服务
 * 
 * 架构设计：
 * 1. 智能体创建时立即保存到 agentStore，确保 UI 及时显示
 * 2. DIAP 身份创建在后台异步执行，不阻塞创建流程
 * 3. DIAP 身份创建完成后，自动更新 agentStore 和 DiapIdentityPanel
 * 4. 进度显示在右上角 DIAP 身份面板内
 */

import { invoke } from '@tauri-apps/api/core'
import {
  saveDiapIdentityToFile,
  loadDiapIdentityFromFile,
  hasDiapIdentityFile
} from '../utils/diapAgentIdentityManager'
import useAgentStore from '../stores/agentStore'

const DEFAULT_IPFS_API = 'http://localhost:5001'
const DEFAULT_IPFS_GATEWAY = 'http://localhost:8080'

/**
 * DIAP 创建进度状态
 */
export interface DiapCreationProgress {
  stage: 'idle' | 'checking_ipfs' | 'creating_did' | 'uploading_to_ipfs' | 'publishing_ipns' | 'generating_zkp' | 'saving' | 'completed' | 'failed'
  message: string
  progress: number // 0-100
  error?: string
  identity?: DiapIdentityData
}

/**
 * DIAP 身份数据接口
 */
export interface DiapIdentityData {
  did: string
  cid: string
  ipns: string
  public_key?: string
  private_key?: string
  did_document?: Record<string, any>
  zkp_proof?: any
  is_registered?: boolean
  created_at?: number
}

/**
 * DIAP 创建任务
 */
export interface DiapCreationTask {
  sessionId: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  progress: DiapCreationProgress
  startedAt?: number
  completedAt?: number
  retryCount: number
  maxRetries: number
}

/**
 * DIAP 异步创建服务类
 */
class AsyncDiapCreationService {
  private tasks: Map<string, DiapCreationTask>
  private completionListeners: Set<(sessionId: string, identity: DiapIdentityData) => void>
  private progressListeners: Set<(sessionId: string, progress: DiapCreationProgress) => void>

  constructor() {
    this.tasks = new Map()
    this.completionListeners = new Set()
    this.progressListeners = new Set()
  }

  /**
   * 启动异步 DIAP 身份创建
   * 
   * @param sessionId - 智能体会话 ID
   * @param agentData - 智能体数据
   * @param options - 选项
   * @returns 任务 ID
   */
  async startDiapCreation(
    sessionId: string,
    agentData: {
      name: string
      roleDescription?: string
      avatarCid?: string | null
      mcpConfigCid?: string | null
      customPrompt?: string | null
    },
    options: {
      ipfsApiUrl?: string
      ipfsGatewayUrl?: string
      maxRetries?: number
    } = {}
  ): Promise<string> {
    // 检查是否已存在 DIAP 身份（基于 agentId）
    if (await hasDiapIdentityFile(sessionId)) {
      const existing = await loadDiapIdentityFromFile(sessionId)
      if (existing && existing.did && existing.cid && existing.ipns) {
        console.log('[AsyncDiapCreation] DIAP 身份已存在，跳过创建:', existing.did)
        // 触发完成回调
        this.completionListeners.forEach(listener => listener(sessionId, existing))
        return sessionId
      }
    }

    // 检查是否已在创建中
    if (this.tasks.has(sessionId)) {
      const existingTask = this.tasks.get(sessionId)!
      if (existingTask.status === 'running' || existingTask.status === 'pending') {
        console.log('[AsyncDiapCreation] DIAP 创建已在进行中，跳过:', sessionId)
        return sessionId
      }
    }

    // 创建任务
    const task: DiapCreationTask = {
      sessionId,
      status: 'pending',
      progress: {
        stage: 'idle',
        message: '等待启动...',
        progress: 0
      },
      startedAt: undefined,
      completedAt: undefined,
      retryCount: 0,
      maxRetries: options.maxRetries || 3
    }

    this.tasks.set(sessionId, task)
    console.log('[AsyncDiapCreation] 创建任务:', sessionId)

    // 异步执行创建（不阻塞）
    this.executeCreation(task, agentData, options).catch(err => {
      console.error('[AsyncDiapCreation] 后台创建失败:', err)
    })

    return sessionId
  }

  /**
   * 执行 DIAP 创建流程
   */
  private async executeCreation(
    task: DiapCreationTask,
    agentData: {
      name: string
      roleDescription?: string
      avatarCid?: string | null
      mcpConfigCid?: string | null
      customPrompt?: string | null
    },
    options: {
      ipfsApiUrl?: string
      ipfsGatewayUrl?: string
      maxRetries?: number
    }
  ): Promise<void> {
    const { sessionId } = task
    const ipfsApiUrl = options.ipfsApiUrl || DEFAULT_IPFS_API
    const ipfsGatewayUrl = options.ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY

    task.status = 'running'
    console.log('[AsyncDiapCreation] 🚀 开始执行 DIAP 创建流程:', sessionId)
    task.startedAt = Date.now()
    this.updateProgress(task, {
      stage: 'checking_ipfs',
      message: '检查 IPFS 节点...',
      progress: 5
    })

    try {
      // 第 1 步：检查 IPFS 节点
      const ipfsReady = await this.checkIpfsReady(ipfsApiUrl)
      if (!ipfsReady) {
        throw new Error('IPFS 节点不可用')
      }

      this.updateProgress(task, {
        stage: 'creating_did',
        message: '创建 DID 文档...',
        progress: 20
      })

      // 第 2 步：创建 DIAP 身份（带 ZKP）
      const identityResult = await this.createDiapIdentityWithZKP(
        sessionId,
        {
          agentName: agentData.name,
          agentDescription: agentData.roleDescription || '',
          ipfsApiUrl,
          ipfsGatewayUrl
        }
      )

      if (!identityResult.success || !identityResult.did) {
        throw new Error(identityResult.error || 'DIAP 身份创建失败')
      }

      const identity: DiapIdentityData = {
        did: identityResult.did,
        cid: identityResult.cid || '',
        ipns: identityResult.ipns || '',
        public_key: identityResult.public_key,
        private_key: identityResult.private_key,
        did_document: identityResult.did_document,
        zkp_proof: identityResult.zkp_proof,
        is_registered: false,
        created_at: Math.floor(Date.now() / 1000)
      }

      task.progress.identity = identity

      // 第 3 步：保存到本地存储（使用 agentId）
      this.updateProgress(task, {
        stage: 'saving',
        message: '保存身份...',
        progress: 90
      })

      // 使用 ipns 或 cid 作为 agentId
      const agentId = identity.ipns ? identity.ipns.replace(/^\/?ipns\//, '') : identity.cid
      await this.saveToLocalStorage(agentId, identity)

      // 第 4 步：更新 agentStore
      await this.updateAgentStore(sessionId, identity)

      // 完成
      task.status = 'completed'
      task.completedAt = Date.now()
      this.updateProgress(task, {
        stage: 'completed',
        message: '创建完成',
        progress: 100,
        identity
      })

      // 触发完成回调
      console.log('[AsyncDiapCreation] ✅ DIAP 身份创建完成:', sessionId, identity.did)
      this.completionListeners.forEach(listener => listener(sessionId, identity))

      // 发射自定义事件，通知 DiapIdentityPanel 刷新显示
      if (typeof window !== 'undefined') {
        const event = new CustomEvent('diap-identity-created', {
          detail: { sessionId, identity }
        })
        window.dispatchEvent(event)
        console.log('[AsyncDiapCreation] 📢 已发射 diap-identity-created 事件')
      }

    } catch (error) {
      const errorMsg = (error as Error).message
      console.error('[AsyncDiapCreation] ❌ DIAP 身份创建失败:', sessionId, errorMsg)

      // 重试逻辑
      if (task.retryCount < task.maxRetries) {
        task.retryCount++
        const retryDelay = Math.min(5000 * task.retryCount, 30000)
        console.log(`[AsyncDiapCreation] ${task.retryCount}/${task.maxRetries} 后重试...`)
        
        this.updateProgress(task, {
          stage: 'idle',
          message: `重试中 (${task.retryCount}/${task.maxRetries})...`,
          progress: 0,
          error: errorMsg
        })

        await new Promise(resolve => setTimeout(resolve, retryDelay))
        await this.executeCreation(task, agentData, options)
        return
      }

      // 重试次数用尽，标记为失败
      task.status = 'failed'
      task.completedAt = Date.now()
      this.updateProgress(task, {
        stage: 'failed',
        message: '创建失败',
        progress: 0,
        error: errorMsg
      })
    }
  }

  /**
   * 更新进度
   */
  private updateProgress(task: DiapCreationTask, progress: Partial<DiapCreationProgress>): void {
    task.progress = { ...task.progress, ...progress }
    
    // 通知进度监听器（DiapIdentityPanel 会监听）
    this.progressListeners.forEach(listener => {
      try {
        listener(task.sessionId, task.progress)
      } catch (error) {
        console.error('[AsyncDiapCreation] 进度监听器错误:', (error as Error).message)
      }
    })
  }

  /**
   * 检查 IPFS 节点是否就绪
   */
  private async checkIpfsReady(ipfsApiUrl: string): Promise<boolean> {
    try {
      const result = await invoke<any>('test_ipfs_api', { ipfsApiUrl })
      return result.success === true
    } catch (error) {
      console.warn('[AsyncDiapCreation] IPFS 检查失败:', (error as Error).message)
      return false
    }
  }

  /**
   * 创建 DIAP 身份（带 ZKP）
   */
  private async createDiapIdentityWithZKP(
    sessionId: string,
    params: {
      agentName: string
      agentDescription: string
      ipfsApiUrl: string
      ipfsGatewayUrl: string
    }
  ): Promise<{
    success: boolean
    did?: string
    cid?: string
    ipns?: string
    public_key?: string
    private_key?: string
    did_document?: Record<string, any>
    zkp_proof?: any
    error?: string
  }> {
    try {
      const result = await invoke<any>('create_diap_identity_with_zkp', {
        sessionId,
        agentName: params.agentName,
        agentDescription: params.agentDescription,
        ipfsApiUrl: params.ipfsApiUrl,
        ipfsGatewayUrl: params.ipfsGatewayUrl
      })

      return result
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message
      }
    }
  }

  /**
   * 保存到本地存储（基于 agentId）
   */
  private async saveToLocalStorage(agentId: string, identity: DiapIdentityData): Promise<void> {
    try {
      await saveDiapIdentityToFile(agentId, identity)
      console.log('[AsyncDiapCreation] 本地存储保存成功:', agentId)
    } catch (error) {
      console.warn('[AsyncDiapCreation] 本地存储保存失败:', (error as Error).message)
    }
  }

  /**
   * 更新 agentStore
   */
  private async updateAgentStore(sessionId: string, identity: DiapIdentityData): Promise<void> {
    try {
      const agent = useAgentStore.getState().getAgent(sessionId)
      if (agent) {
        useAgentStore.getState().updateAgent(sessionId, {
          did: identity.did,
          cid: identity.cid,
          ipns: identity.ipns,
          diapIdentity: identity
        })
        console.log('[AsyncDiapCreation] agentStore 更新成功:', sessionId)
      } else {
        console.warn('[AsyncDiapCreation] agentStore 中未找到智能体:', sessionId)
      }
    } catch (error) {
      console.warn('[AsyncDiapCreation] agentStore 更新失败:', (error as Error).message)
    }
  }

  /**
   * 订阅进度更新
   */
  subscribeProgress(listener: (sessionId: string, progress: DiapCreationProgress) => void): () => void {
    this.progressListeners.add(listener)
    return () => {
      this.progressListeners.delete(listener)
    }
  }

  /**
   * 订阅完成事件
   */
  onCompletion(listener: (sessionId: string, identity: DiapIdentityData) => void): () => void {
    this.completionListeners.add(listener)
    return () => {
      this.completionListeners.delete(listener)
    }
  }

  /**
   * 获取任务状态
   */
  getTaskStatus(sessionId: string): DiapCreationTask | undefined {
    return this.tasks.get(sessionId)
  }

  /**
   * 获取 DIAP 身份
   */
  async getDiapIdentity(sessionId: string): Promise<DiapIdentityData | null> {
    if (await hasDiapIdentityFile(sessionId)) {
      const identity = await loadDiapIdentityFromFile(sessionId)
      // DID 和 CID 是必须的，IPNS 可以为空
      if (identity && identity.did && identity.cid) {
        return identity
      }
    }
    return null
  }

  /**
   * 检查是否正在创建
   */
  isCreating(sessionId: string): boolean {
    const task = this.tasks.get(sessionId)
    return !!task && (task.status === 'running' || task.status === 'pending')
  }

  /**
   * 清理已完成的任务
   */
  cleanupCompletedTasks(maxAgeMs: number = 300000): number {
    const now = Date.now()
    let cleaned = 0

    this.tasks.forEach((task, sessionId) => {
      if ((task.status === 'completed' || task.status === 'failed') &&
          task.completedAt &&
          (now - task.completedAt > maxAgeMs)) {
        this.tasks.delete(sessionId)
        cleaned++
      }
    })

    return cleaned
  }
}

// 创建单例实例
const asyncDiapCreationService = new AsyncDiapCreationService()

// 定期清理已完成的任务（每 5 分钟）
setInterval(() => {
  asyncDiapCreationService.cleanupCompletedTasks()
}, 300000)

export default asyncDiapCreationService
export { AsyncDiapCreationService }
export type { DiapCreationProgress, DiapIdentityData, DiapCreationTask }
