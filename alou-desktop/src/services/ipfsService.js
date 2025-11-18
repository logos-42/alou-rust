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
      await this.getNodeInfo()
      return true
    } catch {
      return false
    }
  }
}

export default new IpfsService()

