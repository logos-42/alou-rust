/**
 * DIAP身份集成服务
 * 统一管理DIAP身份的创建、存储和同步流程
 */

import { invoke } from '@tauri-apps/api/core'
import apiClient from './api'
import { 
  setDiapIdentity, 
  getDiapIdentity, 
  removeDiapIdentity,
  hasDiapIdentity 
} from '../utils/memoryStorage'

const DEFAULT_IPFS_API = 'http://localhost:5001'
const DEFAULT_IPFS_GATEWAY = 'http://localhost:8080'

class DiapIntegrationService {
  constructor() {
    this.isCreating = new Map() // 防止重复创建
  }

  /**
   * 统一的DIAP身份创建流程
   * 1. 后端创建DID文档模板
   * 2. 桌面端生成密钥对并上传到IPFS
   * 3. 同步到后端KV存储
   * 4. 保存到本地存储
   */
  async createDiapIdentity(sessionId, params = {}) {
    // 防止重复创建
    if (this.isCreating.has(sessionId)) {
      throw new Error('DIAP identity creation already in progress for this session')
    }

    // 检查是否已存在
    if (hasDiapIdentity(sessionId)) {
      const existing = getDiapIdentity(sessionId)
      if (existing && existing.did && existing.cid && existing.ipns) {
        console.log('[DiapIntegration] DIAP身份已存在，跳过创建:', existing)
        return { identity: existing }
      }
    }

    this.isCreating.set(sessionId, true)

    try {
      console.log('[DiapIntegration] 开始DIAP身份创建流程:', { sessionId, params })

      // 第1步：检查IPFS节点状态
      await this.ensureIpfsReady(params.ipfsApiUrl)

      // 第2步：从后端获取DID文档模板
      console.log('[DiapIntegration] 步骤1: 从后端获取DID文档模板')
      const didDocumentResponse = await this.createDidDocumentTemplate(sessionId, params)
      
      // 第3步：在桌面端创建真实的DIAP身份
      console.log('[DiapIntegration] 步骤2: 在桌面端创建真实的DIAP身份')
      const identity = await this.createRealDiapIdentity(sessionId, didDocumentResponse.did_document, params)

      // 第4步：保存到后端KV存储
      console.log('[DiapIntegration] 步骤3: 保存到后端KV存储')
      await this.saveToBackendKv(sessionId, identity)

      // 第5步：保存到本地存储
      console.log('[DiapIntegration] 步骤4: 保存到本地存储')
      this.saveToLocalStorage(sessionId, identity)

      console.log('[DiapIntegration] DIAP身份创建完成:', identity)
      return { identity }

    } catch (error) {
      console.error('[DiapIntegration] DIAP身份创建失败:', error)
      throw new Error(`DIAP身份创建失败: ${error.message}`)
    } finally {
      this.isCreating.delete(sessionId)
    }
  }

  /**
   * 检查IPFS节点是否就绪
   */
  async ensureIpfsReady(ipfsApiUrl) {
    const apiUrl = ipfsApiUrl || DEFAULT_IPFS_API
    
    try {
      // 检查IPFS守护进程状态
      const daemonStatus = await invoke('get_ipfs_daemon_status')
      console.log('[DiapIntegration] IPFS守护进程状态:', daemonStatus)

      if (!daemonStatus.process_running) {
        console.log('[DiapIntegration] IPFS守护进程未运行，尝试启动...')
        await invoke('start_ipfs_node')
        
        // 等待启动完成
        await this.waitForIpfsReady(apiUrl, 30000) // 30秒超时
      }

      // 测试API连接
      const apiTest = await invoke('test_ipfs_api', { ipfsApiUrl: apiUrl })
      if (!apiTest.success) {
        throw new Error(`IPFS API不可用: ${apiTest.error}`)
      }

      console.log('[DiapIntegration] IPFS节点就绪')
    } catch (error) {
      throw new Error(`IPFS节点检查失败: ${error.message}`)
    }
  }

  /**
   * 等待IPFS API就绪
   */
  async waitForIpfsReady(apiUrl, timeoutMs = 30000) {
    const startTime = Date.now()
    const checkInterval = 1000 // 1秒检查一次

    while (Date.now() - startTime < timeoutMs) {
      try {
        const result = await invoke('test_ipfs_api', { ipfsApiUrl: apiUrl })
        if (result.success) {
          console.log('[DiapIntegration] IPFS API就绪')
          return
        }
      } catch (error) {
        // 继续等待
      }

      await new Promise(resolve => setTimeout(resolve, checkInterval))
    }

    throw new Error('IPFS API启动超时')
  }

  /**
   * 从后端获取DID文档模板
   */
  async createDidDocumentTemplate(sessionId, params) {
    try {
      const response = await apiClient.post('/agent/diap/create-identity', {
        session_id: sessionId,
        agent_name: params.agentName,
        agent_description: params.agentDescription,
        ipfs_api_url: params.ipfsApiUrl,
        ipfs_gateway_url: params.ipfsGatewayUrl,
        ipns_key: params.ipnsKey,
        custom_prompt: params.customPrompt,
        avatar_cid: params.avatarCid,
        mcp_config_cid: params.mcpConfigCid,
      })

      if (!response.data.did_document) {
        throw new Error('后端返回的DID文档为空')
      }

      console.log('[DiapIntegration] 后端DID文档模板创建成功')
      return response.data
    } catch (error) {
      throw new Error(`获取DID文档模板失败: ${error.message}`)
    }
  }

  /**
   * 在桌面端创建真实的DIAP身份
   */
  async createRealDiapIdentity(sessionId, didDocumentTemplate, params) {
    try {
      console.log('[DiapIntegration] 调用桌面端创建DIAP身份...')
      
      const result = await invoke('create_diap_identity_from_did_document', {
        session_id: sessionId,
        did_document: didDocumentTemplate,
        ipfs_api_url: params.ipfsApiUrl || DEFAULT_IPFS_API,
        ipfs_gateway_url: params.ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        ipns_key: params.ipnsKey,
      })

      // 验证返回的身份信息
      if (!result.did || !result.cid || !result.ipns) {
        throw new Error(`桌面端返回的身份信息不完整: ${JSON.stringify(result)}`)
      }

      console.log('[DiapIntegration] 桌面端DIAP身份创建成功:', {
        did: result.did,
        cid: result.cid,
        ipns: result.ipns,
        public_key: result.public_key ? result.public_key.substring(0, 20) + '...' : 'N/A'
      })

      return result
    } catch (error) {
      console.error('[DiapIntegration] 桌面端身份创建详细错误:', error)
      
      // 提供更详细的错误信息
      if (error.message.includes('IPFS API不可用')) {
        throw new Error('IPFS节点未运行或API不可访问，请检查IPFS服务状态')
      } else if (error.message.includes('IPFS节点正在启动')) {
        throw new Error('IPFS节点正在启动中，请稍等片刻后重试')
      } else if (error.message.includes('上传DID文档到IPFS失败')) {
        throw new Error('DID文档上传到IPFS失败，请检查IPFS节点状态和网络连接')
      } else if (error.message.includes('发布到IPNS失败')) {
        throw new Error('IPNS发布失败，请检查IPNS密钥配置和IPFS节点状态')
      } else {
        throw new Error(`桌面端身份创建失败: ${error.message}`)
      }
    }
  }

  /**
   * 保存到后端KV存储
   */
  async saveToBackendKv(sessionId, identity) {
    try {
      await apiClient.post('/agent/diap/save-complete-identity', {
        session_id: sessionId,
        diap_identity: identity,
      })
      console.log('[DiapIntegration] 后端KV存储保存成功')
    } catch (error) {
      console.warn('[DiapIntegration] 后端KV存储保存失败:', error.message)
      // 不抛出错误，因为本地存储仍然可用
    }
  }

  /**
   * 保存到本地存储
   */
  saveToLocalStorage(sessionId, identity) {
    try {
      setDiapIdentity(sessionId, identity)
      console.log('[DiapIntegration] 本地存储保存成功')
    } catch (error) {
      console.warn('[DiapIntegration] 本地存储保存失败:', error.message)
    }
  }

  /**
   * 获取DIAP身份（优先从本地，然后从后端）
   */
  async getDiapIdentity(sessionId) {
    try {
      // 优先从本地存储获取
      if (hasDiapIdentity(sessionId)) {
        const identity = getDiapIdentity(sessionId)
        if (identity && identity.did && identity.cid && identity.ipns) {
          console.log('[DiapIntegration] 从本地存储获取DIAP身份成功')
          return { identity }
        }
      }

      // 从后端获取
      console.log('[DiapIntegration] 从后端获取DIAP身份...')
      const response = await apiClient.post('/agent/diap/get-identity-by-session', {
        session_id: sessionId,
      })

      if (response.data && response.data.identity) {
        const identity = response.data.identity
        // 保存到本地存储
        this.saveToLocalStorage(sessionId, identity)
        console.log('[DiapIntegration] 从后端获取DIAP身份成功')
        return { identity }
      }

      throw new Error('未找到DIAP身份')
    } catch (error) {
      throw new Error(`获取DIAP身份失败: ${error.message}`)
    }
  }

  /**
   * 删除DIAP身份
   */
  async removeDiapIdentity(sessionId) {
    try {
      // 从本地存储删除
      removeDiapIdentity(sessionId)
      console.log('[DiapIntegration] 本地DIAP身份删除成功')
    } catch (error) {
      console.warn('[DiapIntegration] 删除本地DIAP身份失败:', error.message)
    }
  }

  /**
   * 测试IPNS在公共网关的可访问性
   */
  async testIpnsAccessibility(ipnsName) {
    try {
      const result = await invoke('test_ipns_on_public_gateway', { ipnsName })
      return result
    } catch (error) {
      throw new Error(`测试IPNS可访问性失败: ${error.message}`)
    }
  }

  /**
   * 注册到链上
   */
  async registerOnChain(identity, network, stakeAmount, useAa = false, salt = 0) {
    try {
      const response = await apiClient.post('/agent/diap/register-onchain', {
        ipns: identity.ipns,
        did: identity.did,
        cid: identity.cid,
        public_key: identity.public_key,
        network,
        stake_amount: stakeAmount,
        use_aa: useAa,
        salt,
      })
      return response.data
    } catch (error) {
      throw new Error(`链上注册失败: ${error.message}`)
    }
  }
}

// 创建单例实例
const diapIntegrationService = new DiapIntegrationService()

export default diapIntegrationService