/**
 * DIAP Service - 前端版本
 * 前端版本通过 HTTP API 与后端服务通信，不使用 Tauri
 */
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

  /**
   * 创建本地 DIAP Identity
   * 前端版本通过后端 API 创建
   */
  async createLocalIdentity({ name, description, sessionId, ipfsApiUrl, ipfsGatewayUrl } = {}) {
    // 前端版本：通过后端 API 创建 DIAP Identity
    // 这里需要调用后端 API，暂时返回错误提示
    console.warn('[DiapService] 前端版本暂不支持创建 DIAP Identity，请使用后端 API')
    
    // 返回一个模拟的 identity，实际应该通过后端 API 创建
    return this.withRetry(
      async () => {
        // TODO: 通过后端 API 创建 DIAP Identity
        // 暂时返回一个模拟的响应
        throw new Error('前端版本暂不支持创建 DIAP Identity，请使用桌面版或通过后端 API')
      },
      '创建 DIAP Identity',
      3,
      1500
    )
  }

  /**
   * 获取本地 DIAP Identity
   */
  async getLocalIdentity(ipnsName, ipfsApiUrl, ipfsGatewayUrl) {
    // 前端版本：通过 IPFS Gateway 获取
    return this.withRetry(
      async () => {
        const gateway = ipfsGatewayUrl || DEFAULT_IPFS_GATEWAY
        const url = `${gateway}/ipns/${ipnsName}`
        const response = await fetch(url)
        if (!response.ok) {
          throw new Error(`Failed to fetch DIAP Identity: ${response.statusText}`)
        }
        return await response.json()
      },
      '获取 DIAP Identity',
      3,
      1500
    )
  }

  /**
   * 更新本地 DIAP Identity
   */
  async updateLocalIdentity(ipnsKey, cid, ipfsApiUrl, ipfsGatewayUrl) {
    // 前端版本：通过后端 API 更新
    console.warn('[DiapService] 前端版本暂不支持更新 DIAP Identity，请使用后端 API')
    throw new Error('前端版本暂不支持更新 DIAP Identity，请使用桌面版或通过后端 API')
  }
}

export default new DiapService()
