/**
 * IPFS Service - 与本地 Kubo 节点通信
 * 通过 Tauri 命令与 Rust 后端交互
 */
import { invoke } from '@tauri-apps/api/core'

export class IpfsService {
  /**
   * 下载 Kubo 二进制文件（首次安装或更新时）
   */
  async downloadKubo() {
    try {
      const result = await invoke('download_kubo_binary')
      return { success: true, message: result }
    } catch (error) {
      console.error('Failed to download Kubo:', error)
      return { success: false, error: error.toString() }
    }
  }

  /**
   * 检查 Kubo 二进制是否存在
   */
  async checkKuboInstalled() {
    try {
      // Try to get IPFS info - if it fails, Kubo might not be installed
      await invoke('get_ipfs_info')
      return true
    } catch {
      // Try to start node - this will fail if binary doesn't exist
      try {
        await invoke('start_ipfs_node')
        return true
      } catch {
        return false
      }
    }
  }

  /**
   * 启动本地 IPFS 节点（如果 Kubo 未安装会自动下载）
   */
  async startNode(autoDownload = true) {
    try {
      const result = await invoke('start_ipfs_node')
      return { success: true, message: result }
    } catch (error) {
      const errorMsg = error.toString()
      
      // If binary not found and autoDownload is enabled, try to download
      if (autoDownload && errorMsg.includes('not found')) {
        console.log('Kubo binary not found, downloading...')
        const downloadResult = await this.downloadKubo()
        if (downloadResult.success) {
          // Retry starting after download
          return this.startNode(false)
        }
        return { success: false, error: 'Failed to download Kubo binary' }
      }
      
      // 检查是否是端口占用错误
      if (errorMsg.includes('端口') && errorMsg.includes('5001') || errorMsg.includes('port') && errorMsg.includes('5001')) {
        console.warn('IPFS 端口 5001 已被占用，将尝试使用现有的 IPFS 实例')
        // 不返回错误，允许使用已存在的 IPFS 实例
        return { 
          success: true, 
          message: '使用已存在的 IPFS 实例',
          warning: '检测到另一个 IPFS 实例正在运行，将使用现有实例'
        }
      }
      
      console.error('Failed to start IPFS node:', error)
      return { success: false, error: errorMsg }
    }
  }

  /**
   * 停止本地 IPFS 节点
   */
  async stopNode() {
    try {
      const result = await invoke('stop_ipfs_node')
      return { success: true, message: result }
    } catch (error) {
      console.error('Failed to stop IPFS node:', error)
      return { success: false, error: error.toString() }
    }
  }

  /**
   * 获取 IPFS 节点信息（包括 Peer ID）
   */
  async getNodeInfo() {
    try {
      const info = await invoke('get_ipfs_info')
      return { success: true, info }
    } catch (error) {
      console.error('Failed to get IPFS info:', error)
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
   * 从 IPFS 配置中获取 API 地址
   */
  async getApiAddressFromConfig() {
    try {
      const address = await invoke('get_ipfs_api_address')
      return { success: true, address }
    } catch (error) {
      return { success: false, error: error.toString() }
    }
  }

  /**
   * 诊断 IPFS API 配置问题
   */
  async diagnoseApi() {
    try {
      const diagnosis = await invoke('diagnose_ipfs_api')
      return { success: true, diagnosis }
    } catch (error) {
      return { success: false, error: error.toString() }
    }
  }

  /**
   * 测试 IPFS HTTP API 是否就绪
   */
  async testHttpApi(ipfsApiUrl = null, useConfig = false) {
    try {
      const apiUrl = ipfsApiUrl || import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
      
      // 如果需要从配置读取，使用带配置的函数
      const command = useConfig ? 'test_ipfs_api_with_config' : 'test_ipfs_api'
      const params = useConfig 
        ? { ipfsApiUrl: apiUrl }
        : { ipfsApiUrl: apiUrl }
      
      const result = await invoke(command, params)
      
      // 返回 JSON 对象
      if (typeof result === 'object' && result !== null) {
        return {
          success: result.success || false,
          apiUrl: result.api_url || apiUrl,
          error: result.error || null
        }
      }
      
      // 向后兼容：如果返回布尔值
      return { success: result === true, apiUrl }
    } catch (error) {
      return { success: false, error: error.toString(), apiUrl: ipfsApiUrl || 'http://127.0.0.1:5001' }
    }
  }

  /**
   * 等待 IPFS API 就绪（重试机制）
   * 先测试 HTTP API，如果失败则回退到命令行接口检查
   * @param {number} maxRetries - 最大重试次数
   * @param {number} delayMs - 每次重试的延迟（毫秒）
   */
  async waitForApiReady(maxRetries = 20, delayMs = 1000) {
    // 首先尝试从 IPFS 配置中获取实际的 API 地址
    let apiUrl = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'
    const configAddress = await this.getApiAddressFromConfig()
    if (configAddress.success) {
      apiUrl = configAddress.address
      console.log(`[IPFS] 从配置中读取 API 地址: ${apiUrl}`)
    } else {
      console.log(`[IPFS] 使用默认 API 地址: ${apiUrl}`)
    }
    
    for (let i = 0; i < maxRetries; i++) {
      // 优先测试 HTTP API（这是实际使用的接口）
      // 前几次尝试使用默认地址，之后尝试从配置读取
      const useConfig = i >= 3 && i % 5 === 0
      const httpTest = await this.testHttpApi(apiUrl, useConfig)
      if (httpTest.success) {
        console.log(`[IPFS] HTTP API 已就绪 (尝试 ${i + 1}/${maxRetries}, 地址: ${httpTest.apiUrl})`)
        return { success: true, attempts: i + 1, method: 'http', apiUrl: httpTest.apiUrl }
      }
      
      // 显示详细的错误信息（每 5 次尝试显示一次，避免日志过多）
      if (i % 5 === 0 && httpTest.error) {
        console.warn(`[IPFS] HTTP API 测试失败 (尝试 ${i + 1}/${maxRetries}):`, httpTest.error)
      }
      
      // 如果 HTTP API 不可用，也检查命令行接口（作为备用检查）
      try {
        const result = await this.getNodeInfo()
        if (result.success) {
          // 命令行接口可用，但 HTTP API 还不可用，继续等待
          if (i % 5 === 0) {
            console.log(`[IPFS] 命令行接口可用，但 HTTP API 尚未就绪 (尝试 ${i + 1}/${maxRetries}, 地址: ${apiUrl})`)
          }
        }
      } catch (error) {
        // 命令行接口也不可用
      }
      
      if (i < maxRetries - 1) {
        // 等待一段时间后重试
        await new Promise((resolve) => setTimeout(resolve, delayMs))
      }
    }
    
    // 最后尝试从配置中获取地址并测试
    const finalConfigAddress = await this.getApiAddressFromConfig()
    if (finalConfigAddress.success && finalConfigAddress.address !== apiUrl) {
      console.log(`[IPFS] 最后尝试使用配置中的地址: ${finalConfigAddress.address}`)
      const finalTest = await this.testHttpApi(finalConfigAddress.address)
      if (finalTest.success) {
        return { success: true, attempts: maxRetries + 1, method: 'http', apiUrl: finalTest.apiUrl }
      }
    }
    
    // 如果所有尝试都失败，进行诊断
    console.warn('[IPFS] 所有尝试都失败，开始诊断...')
    const diagnosis = await this.diagnoseApi()
    let errorMsg = `IPFS HTTP API 在 ${maxRetries} 次尝试后仍未就绪。\n`
    
    if (diagnosis.success && diagnosis.diagnosis) {
      const diag = diagnosis.diagnosis
      errorMsg += `\n诊断结果：\n`
      errorMsg += `- 配置文件存在: ${diag.config_exists}\n`
      errorMsg += `- API 已启用: ${diag.api_enabled}\n`
      errorMsg += `- API 地址: ${diag.api_address || diag.api_address_http || '未设置'}\n`
      errorMsg += `- API 可访问: ${diag.api_accessible !== undefined ? diag.api_accessible : '未知'}\n`
      
      if (diag.recommendations && Array.isArray(diag.recommendations) && diag.recommendations.length > 0) {
        errorMsg += `\n建议：\n`
        diag.recommendations.forEach((rec, idx) => {
          errorMsg += `${idx + 1}. ${rec}\n`
        })
      }
    } else {
      errorMsg += `\n请检查：\n` +
                  `1. IPFS 节点是否正在运行\n` +
                  `2. API 地址是否正确: ${apiUrl}\n` +
                  `3. 防火墙是否阻止了连接\n` +
                  `4. 是否有多个 IPFS 实例在运行（可能导致端口冲突）\n` +
                  `5. IPFS API 是否在配置中被禁用（运行诊断命令查看详情）`
    }
    
    return {
      success: false,
      error: errorMsg,
      diagnosis: diagnosis.success ? diagnosis.diagnosis : null
    }
  }
}

export default new IpfsService()

