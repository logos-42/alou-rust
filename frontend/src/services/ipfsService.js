/**
 * IPFS Service - 前端版本（使用 HTTP API）
 * 前端版本不使用 Tauri，直接通过 HTTP API 与 IPFS 通信
 */
const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

export class IpfsService {
  /**
   * 下载 Kubo 二进制文件（前端版本不支持）
   */
  async downloadKubo() {
    return { success: false, error: '前端版本不支持下载 Kubo 二进制文件' }
  }

  /**
   * 检查 Kubo 二进制是否存在（前端版本不支持）
   */
  async checkKuboInstalled() {
    // 前端版本假设 IPFS 节点已经在运行
    return true
  }

  /**
   * 启动本地 IPFS 节点（前端版本不支持）
   */
  async startNode(autoDownload = true) {
    return { success: false, error: '前端版本不支持启动 IPFS 节点。请确保 IPFS 节点已在运行。' }
  }

  /**
   * 停止本地 IPFS 节点（前端版本不支持）
   */
  async stopNode() {
    return { success: false, error: '前端版本不支持停止 IPFS 节点' }
  }

  /**
   * 获取 IPFS 节点信息（通过 HTTP API）
   */
  async getNodeInfo() {
    try {
      const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/id`, {
        method: 'POST',
      })
      if (!response.ok) {
        return { success: false, error: `HTTP ${response.status}: ${response.statusText}` }
      }
      const info = await response.json()
      return { success: true, info }
    } catch (error) {
      return { success: false, error: error.toString() }
    }
  }

  /**
   * 检查 IPFS 节点是否运行
   */
  async isNodeRunning() {
    try {
      const result = await this.getNodeInfo()
      return result.success
    } catch {
      return false
    }
  }

  /**
   * 从 IPFS 配置中获取 API 地址（前端版本不支持）
   */
  async getApiAddressFromConfig() {
    return { success: true, address: DEFAULT_IPFS_API }
  }

  /**
   * 诊断 IPFS API 配置问题（前端版本简化版）
   */
  async diagnoseApi() {
    const testResult = await this.testHttpApi()
    return {
      success: true,
      diagnosis: {
        api_accessible: testResult.success,
        api_address: DEFAULT_IPFS_API,
        recommendations: testResult.success
          ? []
          : ['请确保 IPFS 节点正在运行', '检查 API 地址是否正确', '检查防火墙设置'],
      },
    }
  }

  /**
   * 测试 IPFS HTTP API 是否就绪
   */
  async testHttpApi(ipfsApiUrl = null) {
    try {
      const apiUrl = ipfsApiUrl || DEFAULT_IPFS_API
      const response = await fetch(`${apiUrl}/api/v0/version`, {
        method: 'POST',
      })
      const success = response.ok
      return {
        success,
        apiUrl,
        error: success ? null : `HTTP ${response.status}: ${response.statusText}`,
      }
    } catch (error) {
      return {
        success: false,
        error: error.toString(),
        apiUrl: ipfsApiUrl || DEFAULT_IPFS_API,
      }
    }
  }

  /**
   * 等待 IPFS API 就绪（重试机制）
   */
  async waitForApiReady(maxRetries = 20, delayMs = 1000) {
    const apiUrl = DEFAULT_IPFS_API

    for (let i = 0; i < maxRetries; i++) {
      const httpTest = await this.testHttpApi(apiUrl)
      if (httpTest.success) {
        console.log(`[IPFS] HTTP API 已就绪 (尝试 ${i + 1}/${maxRetries}, 地址: ${httpTest.apiUrl})`)
        return { success: true, attempts: i + 1, method: 'http', apiUrl: httpTest.apiUrl }
      }

      if (i % 5 === 0 && httpTest.error) {
        console.warn(`[IPFS] HTTP API 测试失败 (尝试 ${i + 1}/${maxRetries}):`, httpTest.error)
      }

      if (i < maxRetries - 1) {
        await new Promise((resolve) => setTimeout(resolve, delayMs))
      }
    }

    return {
      success: false,
      error: `IPFS HTTP API 在 ${maxRetries} 次尝试后仍未就绪。请确保 IPFS 节点正在运行，API 地址: ${apiUrl}`,
      attempts: maxRetries,
    }
  }
}

export default new IpfsService()
