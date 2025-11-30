/**
 * Agent Assets Service - 前端版本（使用 HTTP API）
 * 前端版本不使用 Tauri，直接通过 IPFS HTTP API 上传文件
 */
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
  /**
   * 带重试的 IPFS 操作包装器
   */
  async withRetry(operation, operationName, maxRetries = 3, delayMs = 1000) {
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
              `[AgentAssetsService] ${operationName} 遇到 502 错误，等待 ${delayMs}ms 后重试 (${i + 1}/${maxRetries})`
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
   * 上传文件到 IPFS（使用 HTTP API）
   */
  async uploadToIpfs(file, fileName, ipfsApiUrl = DEFAULT_IPFS_API) {
    const formData = new FormData()
    formData.append('file', file)

    const response = await fetch(`${ipfsApiUrl}/api/v0/add`, {
      method: 'POST',
      body: formData,
    })

    if (!response.ok) {
      throw new Error(`IPFS 上传失败: ${response.status} ${response.statusText}`)
    }

    const result = await response.json()
    return {
      cid: result.Hash,
      name: result.Name || fileName,
      size: result.Size,
    }
  }

  /**
   * 上传 Base64 数据到 IPFS（使用 HTTP API）
   */
  async uploadBase64ToIpfs(dataBase64, fileName, ipfsApiUrl = DEFAULT_IPFS_API) {
    // 将 Base64 转换为 Blob
    const binaryString = atob(dataBase64)
    const bytes = new Uint8Array(binaryString.length)
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i)
    }
    const blob = new Blob([bytes])
    const file = new File([blob], fileName, { type: 'application/octet-stream' })

    return this.uploadToIpfs(file, fileName, ipfsApiUrl)
  }

  async uploadAvatar(file, options = {}) {
    if (!file) {
      return null
    }
    const ipfsApiUrl = options.ipfsApiUrl || DEFAULT_IPFS_API
    return this.withRetry(
      async () => {
        const result = await this.uploadToIpfs(file, file.name, ipfsApiUrl)
        return result
      },
      '上传头像',
      3,
      1500
    )
  }

  async uploadMcpConfig(mcpConfig, options = {}) {
    if (!mcpConfig) {
      return null
    }
    const dataBase64 = encodeJson(mcpConfig)
    const fileName = options.filename || 'mcp-config.json'
    const ipfsApiUrl = options.ipfsApiUrl || DEFAULT_IPFS_API
    return this.withRetry(
      async () => {
        const result = await this.uploadBase64ToIpfs(dataBase64, fileName, ipfsApiUrl)
        return result
      },
      '上传 MCP 配置',
      3,
      1500
    )
  }

  resolveIpfsUri(cid) {
    if (!cid) return null
    if (cid.startsWith('http')) return cid
    return `${DEFAULT_GATEWAY}/ipfs/${cid}`
  }
}

export default new AgentAssetsService()
