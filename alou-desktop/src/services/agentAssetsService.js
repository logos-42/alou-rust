import { invoke } from '@tauri-apps/api/core'

const DEFAULT_GATEWAY =
  import.meta.env.VITE_IPFS_GATEWAY_URL?.replace(/\/$/, '') || 'https://ipfs.io'
const DEFAULT_IPFS_API = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

const base64FromFile = (file) =>
  new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result
      if (typeof result === 'string') {
        const [, base64] = result.split(',')
        resolve(base64 || '')
      } else {
        reject(new Error('无法读取文件内容'))
      }
    }
    reader.onerror = () => reject(reader.error || new Error('文件读取失败'))
    reader.readAsDataURL(file)
  })

const encodeJson = (value) => {
  try {
    const json = typeof value === 'string' ? value : JSON.stringify(value)
    return window.btoa(unescape(encodeURIComponent(json)))
  } catch (error) {
    console.error('[AgentAssetsService] JSON 编码失败', error)
    throw new Error('配置编码失败')
  }
}

export class AgentAssetsService {
  async uploadAvatar(file, options = {}) {
    if (!file) {
      return null
    }
    const dataBase64 = await base64FromFile(file)
    const response = await invoke('ipfs_add_base64', {
      dataBase64,
      fileName: file.name,
      ipfsApiUrl: options.ipfsApiUrl || DEFAULT_IPFS_API,
    })
    return response
  }

  async uploadMcpConfig(mcpConfig, options = {}) {
    if (!mcpConfig) {
      return null
    }
    const dataBase64 = encodeJson(mcpConfig)
    const response = await invoke('ipfs_add_base64', {
      dataBase64,
      fileName: options.filename || 'mcp-config.json',
      ipfsApiUrl: options.ipfsApiUrl || DEFAULT_IPFS_API,
    })
    return response
  }

  resolveIpfsUri(cid) {
    if (!cid) return null
    if (cid.startsWith('http')) return cid
    return `${DEFAULT_GATEWAY}/ipfs/${cid}`
  }
}

export default new AgentAssetsService()

