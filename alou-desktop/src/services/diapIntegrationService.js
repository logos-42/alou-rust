/**
 * DIAP身份集成服务
 * 
 * 正确的架构分工：
 * 1. 后端 (alou-edge): 创建DID文档和身份基础信息
 * 2. 桌面端 (alou-desktop): 处理IPFS操作 - 上传文档和发布IPNS
 * 3. 前端 (DiapIntegrationService): 协调整个流程
 * 
 * 注意：桌面端不创建DIAP身份，只处理IPFS存储操作
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
   * 创建完整的DIAP身份（新架构）
   * 1. 桌面端本地创建完整DIAP身份（包含DID、CID、IPNS、ZKP）
   * 2. 桌面端处理所有IPFS操作
   * 3. 桌面端将完整身份保存到后端KV
   * 4. 保存到本地存储
   */
  async createCompleteDiapIdentity(sessionId, params = {}) {
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
      console.log('[DiapIntegration] 开始完整DIAP身份创建流程:', { sessionId, params })

      // 第1步：检查IPFS节点状态
      await this.ensureIpfsReady(params.ipfsApiUrl)

      // 第2步：桌面端本地创建完整DIAP身份（包含DID、CID、IPNS、ZKP）
      console.log('[DiapIntegration] 步骤1: 桌面端本地创建完整DIAP身份')
      const completeIdentity = await this.createDiapIdentityWithZKPCommand(sessionId, params)
      
      if (!completeIdentity.success) {
        throw new Error(`完整DIAP身份创建失败: ${completeIdentity.error || 'Unknown error'}`)
      }

      console.log('[DiapIntegration] ✅ 完整DIAP身份创建成功:', {
        did: completeIdentity.did,
        cid: completeIdentity.cid,
        ipns: completeIdentity.ipns,
        zkp_generated: completeIdentity.zkp_proof?.generated
      })

      // 第3步：保存到本地存储（跳过后端KV存储）
      console.log('[DiapIntegration] 步骤2: 保存到本地存储')
      this.saveToLocalStorage(sessionId, completeIdentity)

      console.log('[DiapIntegration] 🎉 完整DIAP身份创建完成:', completeIdentity)
      return { identity: completeIdentity }

    } catch (error) {
      console.error('[DiapIntegration] 完整DIAP身份创建失败:', error)
      throw new Error(`完整DIAP身份创建失败: ${error.message}`)
    } finally {
      this.isCreating.delete(sessionId)
    }
  }

  /**
   * 调用Tauri命令创建带ZKP的DIAP身份
   */
  async createDiapIdentityWithZKPCommand(sessionId, params) {
    try {
      const result = await invoke('create_diap_identity_with_zkp', {
        agentName: params.agentName || 'Unnamed',
        agentDescription: params.agentDescription || '',
        ipfsApiUrl: params.ipfsApiUrl || DEFAULT_IPFS_API,
        ipfsGatewayUrl: params.ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        sessionId: sessionId
      })
      
      console.log('[DiapIntegration] 本地ZKP命令结果:', result)
      return result
    } catch (error) {
      console.error('[DiapIntegration] 本地ZKP命令调用失败:', error)
      throw error
    }
  }

  /**
   * 检查IPFS节点是否就绪
   */
  async ensureIpfsReady(ipfsApiUrl) {
    const apiUrl = ipfsApiUrl || DEFAULT_IPFS_API
    
    try {
      console.log('[DiapIntegration] 检查IPFS API连接...')
      
      // 直接测试API连接，不依赖进程状态检查
      const apiTest = await invoke('test_ipfs_api', { ipfsApiUrl: apiUrl })
      console.log('[DiapIntegration] IPFS API测试结果:', apiTest)
      
      if (!apiTest.success) {
        console.log('[DiapIntegration] IPFS API不可用，尝试启动守护进程...')
        
        // 检查守护进程状态
        const daemonStatus = await invoke('get_ipfs_daemon_status')
        console.log('[DiapIntegration] IPFS守护进程状态:', daemonStatus)

        if (!daemonStatus.process_running) {
          console.log('[DiapIntegration] IPFS守护进程未运行，尝试启动...')
          await invoke('start_ipfs_node')
          
          // 等待启动完成
          await this.waitForIpfsReady(apiUrl, 30000) // 30秒超时
        }

        // 再次测试API连接
        const retryApiTest = await invoke('test_ipfs_api', { ipfsApiUrl: apiUrl })
        if (!retryApiTest.success) {
          throw new Error(`IPFS API不可用: ${retryApiTest.error}`)
        }
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
   * 后端创建基础DIAP身份（不含CID/IPNS）
   */
  async createBasicIdentityOnBackend(sessionId, params) {
    try {
      const response = await apiClient.post('/agent/diap/create-identity', {
        session_id: sessionId,
        agent_name: params.agentName,
        agent_description: params.agentDescription,
        ipfs_api_url: params.ipfsApiUrl,
        ipfs_gateway_url: params.ipfsGatewayUrl,
        custom_prompt: params.customPrompt,
        avatar_cid: params.avatarCid,
        mcp_config_cid: params.mcpConfigCid,
      })

      if (!response.data.did || !response.data.did_document || !response.data.public_key || !response.data.ipns_key) {
        throw new Error('后端返回的DIAP身份信息不完整')
      }

      console.log('[DiapIntegration] 后端基础DIAP身份创建成功')
      return response.data
    } catch (error) {
      throw new Error(`后端创建基础DIAP身份失败: ${error.message}`)
    }
  }

  /**
   * 保存完整身份到后端KV存储
   */
  async saveCompleteIdentityToBackend(sessionId, completeIdentity) {
    try {
      const response = await apiClient.post('/agent/diap/save-complete-identity', {
        session_id: sessionId,
        diap_identity: {
          did: completeIdentity.did,
          cid: completeIdentity.cid,
          ipns: completeIdentity.ipns,
          public_key: completeIdentity.public_key,
          private_key: completeIdentity.private_key,
          did_document: completeIdentity.did_document,
          zkp_proof: completeIdentity.zkp_proof,
          is_registered: false,
          created_at: Math.floor(Date.now() / 1000),
        }
      })

      console.log('[DiapIntegration] 完整身份保存到后端成功')
      return response.data
    } catch (error) {
      console.warn('[DiapIntegration] 后端KV存储保存失败:', error.message)
      // 不抛出错误，因为本地存储仍然可用
    }
  }

  /**
   * 桌面端上传DID文档到IPFS
   */
  async uploadDidDocumentToIpfs(didDocument, params) {
    try {
      console.log('[DiapIntegration] 上传DID文档到IPFS...')
      
      const result = await invoke('ipfs_add_json', {
        json_data: didDocument,
        file_name: 'did_document.json',
        ipfs_api_url: params.ipfsApiUrl || DEFAULT_IPFS_API,
      })

      if (!result.cid) {
        throw new Error('IPFS上传失败，未返回CID')
      }

      console.log('[DiapIntegration] DID文档上传成功:', result.cid)
      return result
    } catch (error) {
      console.error('[DiapIntegration] IPFS上传详细错误:', error)
      throw new Error(`DID文档上传到IPFS失败: ${error.message}`)
    }
  }

  /**
   * 桌面端发布到IPNS
   */
  async publishToIpns(cid, ipnsKey, params) {
    try {
      console.log('[DiapIntegration] 发布到IPNS...')
      
      const result = await invoke('ipfs_publish_ipns', {
        cid: cid,
        ipns_key: ipnsKey,
        ipfs_api_url: params.ipfsApiUrl || DEFAULT_IPFS_API,
      })

      if (!result.ipns) {
        throw new Error('IPNS发布失败，未返回IPNS名称')
      }

      console.log('[DiapIntegration] IPNS发布成功:', result.ipns)
      return result
    } catch (error) {
      console.error('[DiapIntegration] IPNS发布详细错误:', error)
      throw new Error(`IPNS发布失败: ${error.message}`)
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
