import { invoke } from '@tauri-apps/api/core'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
const DEFAULT_IPFS_GATEWAY =
  import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

class DiapService {
  /**
   * 带重试的 IPFS 操作包装器
   */
  async withRetry(operation, operationName, maxRetries = 3, delayMs = 1500) {
    let lastError
    for (let i = 0; i < maxRetries; i++) {
      try {
        return await operation()
      } catch (error) {
        lastError = error
        const errorMsg = error?.message || error?.toString() || ''
        
        // 如果是 502 错误，等待后重试
        if (errorMsg.includes('502') || errorMsg.includes('Bad Gateway')) {
          if (i < maxRetries - 1) {
            console.log(
              `[DiapService] ${operationName} 遇到 502 错误，等待 ${delayMs}ms 后重试 (${i + 1}/${maxRetries})`
            )
            await new Promise((resolve) => setTimeout(resolve, delayMs))
            continue
          }
        }
        
        // 其他错误或重试次数用完，直接抛出
        throw error
      }
    }
    throw lastError
  }

  async createLocalIdentity({ name, description, sessionId, avatarCid, mcpConfigCid, customPrompt, ipfsApiUrl, ipfsGatewayUrl } = {}) {
    return this.withRetry(
      async () => {
        const response = await invoke('create_local_diap_identity', {
          params: {
            agent_name: name,
            agent_description: description,
            session_id: sessionId,
            avatar_cid: avatarCid,
            mcp_config_cid: mcpConfigCid,
            custom_prompt: customPrompt,
            ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
            ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
          },
        })
        return response
      },
      '创建 DIAP Identity',
      3,
      1500
    )
  }

  async getLocalIdentity(ipnsName, ipfsApiUrl, ipfsGatewayUrl) {
    return this.withRetry(
      async () => {
        const response = await invoke('get_local_diap_identity', {
          ipns_name: ipnsName,
          ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
          ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        })
        return response
      },
      '获取 DIAP Identity',
      3,
      1500
    )
  }

  async updateLocalIdentity(ipnsKey, cid, ipfsApiUrl, ipfsGatewayUrl) {
    return this.withRetry(
      async () => {
        const response = await invoke('update_local_diap_identity', {
          ipns_key: ipnsKey,
          cid: cid,
          ipfs_api_url: ipfsApiUrl || DEFAULT_IPFS_API,
          ipfs_gateway_url: ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY,
        })
        return response
      },
      '更新 DIAP Identity',
      3,
      1500
    )
  }
}

export default new DiapService()

