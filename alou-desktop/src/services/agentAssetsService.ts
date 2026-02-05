import ipfsService from './ipfsContentService'

class AgentAssetsService {
  /**
   * 上传头像文件到 IPFS
   * @param {File} file - 头像文件
   * @param {Object} options - 上传选项
   * @param {string} options.sessionId - 会话ID
   * @returns {Promise<Object>} 上传结果，包含 CID
   */
  async uploadAvatar(file, options = {}) {
    try {
      console.log('[AgentAssetsService] 开始上传头像:', file.name)
      
      // 使用 ipfsService 上传文件
      const result = await ipfsService.uploadFile(file)
      
      console.log('[AgentAssetsService] 头像上传成功:', result.cid)
      return {
        cid: result.cid,
        name: file.name,
        size: file.size,
        type: file.type
      }
    } catch (error) {
      console.error('[AgentAssetsService] 头像上传失败:', error)
      throw new Error(`头像上传失败: ${error.message}`)
    }
  }

  /**
   * 上传 MCP 配置到 IPFS
   * @param {Object} config - MCP 配置对象
   * @param {Array} config.ports - 端口配置
   * @param {number} config.generatedAt - 生成时间
   * @returns {Promise<Object>} 上传结果，包含 CID
   */
  async uploadMcpConfig(config) {
    try {
      console.log('[AgentAssetsService] 开始上传 MCP 配置:', config)
      
      // 将配置转换为 JSON 字符串
      const configJson = JSON.stringify(config, null, 2)
      const blob = new Blob([configJson], { type: 'application/json' })
      const file = new File([blob], 'mcp-config.json', { type: 'application/json' })
      
      // 使用 ipfsService 上传文件
      const result = await ipfsService.uploadFile(file)
      
      console.log('[AgentAssetsService] MCP 配置上传成功:', result.cid)
      return {
        cid: result.cid,
        config: config,
        uploadedAt: Date.now()
      }
    } catch (error) {
      console.error('[AgentAssetsService] MCP 配置上传失败:', error)
      throw new Error(`MCP 配置上传失败: ${error.message}`)
    }
  }

  /**
   * 从 IPFS 获取头像
   * @param {string} cid - 头像文件的 CID
   * @returns {Promise<string>} 头像的 URL
   */
  async getAvatar(cid) {
    try {
      console.log('[AgentAssetsService] 获取头像:', cid)
      const url = await ipfsService.getFileUrl(cid)
      return url
    } catch (error) {
      console.error('[AgentAssetsService] 获取头像失败:', error)
      throw new Error(`获取头像失败: ${error.message}`)
    }
  }

  /**
   * 从 IPFS 获取 MCP 配置
   * @param {string} cid - MCP 配置文件的 CID
   * @returns {Promise<Object>} MCP 配置对象
   */
  async getMcpConfig(cid) {
    try {
      console.log('[AgentAssetsService] 获取 MCP 配置:', cid)
      const content = await ipfsService.getFileContent(cid)
      return JSON.parse(content)
    } catch (error) {
      console.error('[AgentAssetsService] 获取 MCP 配置失败:', error)
      throw new Error(`获取 MCP 配置失败: ${error.message}`)
    }
  }
}

const agentAssetsService = new AgentAssetsService()
export default agentAssetsService
