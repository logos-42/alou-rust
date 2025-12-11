/**
 * IPFS 内容服务 - 统一的 IPFS 内容获取和上传
 */
import { fetchJsonFromGateway } from './utils/gatewayUtils'

const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

export class IpfsContentService {
  /**
   * 从 IPFS CID 获取内容
   * @param {string} cid - IPFS CID
   * @returns {Promise<Object>} 内容数据
   */
  async getContent(cid) {
    console.log(`[IpfsContentService] 从 IPFS 获取内容: ${cid}`)
    
    try {
      const data = await fetchJsonFromGateway(`/ipfs/${cid}`)
      console.log(`[IpfsContentService] 获取内容成功: ${cid}`)
      return { success: true, data, source: 'ipfs' }
    } catch (error) {
      console.error(`[IpfsContentService] 获取内容失败: ${cid}`, error)
      return { success: false, error: error.message, source: 'ipfs' }
    }
  }
  
  /**
   * 从 IPNS 获取内容
   * @param {string} ipnsName - IPNS 名称
   * @returns {Promise<Object>} 内容数据
   */
  async getContentFromIpns(ipnsName) {
    console.log(`[IpfsContentService] 从 IPNS 获取内容: ${ipnsName}`)
    
    // 规范化 IPNS 名称
    const normalizedName = ipnsName.startsWith('/ipns/') ? ipnsName : `/ipns/${ipnsName}`
    
    try {
      const data = await fetchJsonFromGateway(normalizedName)
      console.log(`[IpfsContentService] 从 IPNS 获取内容成功: ${ipnsName}`)
      return { success: true, data, source: 'ipns-gateway' }
    } catch (error) {
      console.error(`[IpfsContentService] 从 IPNS 获取内容失败: ${ipnsName}`, error)
      return { success: false, error: error.message, source: 'ipns-gateway' }
    }
  }
  
  /**
   * 上传内容到 IPFS
   * @param {string} content - 内容（字符串）
   * @param {string} filename - 文件名
   * @returns {Promise<string>} CID
   */
  async uploadContent(content, filename) {
    console.log(`[IpfsContentService] 上传内容到 IPFS: ${filename}`)
    
    try {
      // 优先尝试使用 Tauri 命令（避免 CORS 问题）
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        
        // 将内容转换为 base64
        const base64Data = btoa(unescape(encodeURIComponent(content)))
        
        const result = await invoke('ipfs_add_base64', {
          dataBase64: base64Data,
          fileName: filename,
          ipfsApiUrl: DEFAULT_IPFS_API,
        })
        
        console.log(`[IpfsContentService] 通过 Tauri 上传成功: ${result.cid}`)
        return result.cid
      } catch (tauriError) {
        // Tauri 调用失败，回退到直接 fetch
        console.warn('[IpfsContentService] Tauri invoke 失败，回退到 fetch:', tauriError.message || tauriError)
        
        const response = await fetch(`${DEFAULT_IPFS_API}/api/v0/add`, {
          method: 'POST',
          body: new Blob([content], { type: 'application/json' }),
        })
        
        if (!response.ok) {
          throw new Error(`IPFS 上传失败: ${response.status}`)
        }
        
        const result = await response.json()
        console.log(`[IpfsContentService] 通过 fetch 上传成功: ${result.Hash}`)
        return result.Hash
      }
    } catch (error) {
      console.error('[IpfsContentService] 上传内容到 IPFS 失败:', error)
      throw error
    }
  }
}

export default new IpfsContentService()

