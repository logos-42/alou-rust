/**
 * IPFS Gateway 工具函数
 */

const DEFAULT_IPFS_GATEWAY = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'

/**
 * 公共 IPFS Gateway 列表
 */
export const PUBLIC_GATEWAYS = [
  'https://ipfs.io',
  'https://gateway.ipfs.io',
  'https://dweb.link',
]

/**
 * 从本地 Gateway 获取内容
 * @param {string} path - IPFS 路径 (/ipfs/... 或 /ipns/...)
 * @param {Object} options - 选项
 * @returns {Promise<Response>}
 */
export async function fetchFromLocalGateway(path, options = {}) {
  const gatewayUrl = `${DEFAULT_IPFS_GATEWAY}${path}`
  return fetch(gatewayUrl, {
    method: 'GET',
    headers: { 'Accept': 'application/json', ...options.headers },
    ...options,
  })
}

/**
 * 从公共 Gateway 列表依次尝试获取内容
 * @param {string} path - IPFS 路径 (/ipfs/... 或 /ipns/...)
 * @param {Object} options - 选项
 * @returns {Promise<Response>}
 */
export async function fetchFromPublicGateways(path, options = {}) {
  const errors = []
  
  for (const gateway of PUBLIC_GATEWAYS) {
    try {
      const gatewayUrl = `${gateway}${path}`
      const response = await fetch(gatewayUrl, {
        method: 'GET',
        headers: { 'Accept': 'application/json', ...options.headers },
        ...options,
      })
      
      if (response.ok) {
        const contentType = response.headers.get('content-type') || ''
        if (contentType.includes('application/json')) {
          return response
        }
      }
    } catch (error) {
      errors.push({ gateway, error: error.message })
      continue
    }
  }
  
  throw new Error(`All public gateways failed: ${errors.map(e => `${e.gateway}: ${e.error}`).join(', ')}`)
}

/**
 * 从 Gateway 获取 JSON 内容（带降级）
 * @param {string} path - IPFS 路径 (/ipfs/... 或 /ipns/...)
 * @param {Object} options - 选项
 * @returns {Promise<Object>} JSON 数据
 */
export async function fetchJsonFromGateway(path, options = {}) {
  // 先尝试本地 Gateway
  try {
    const response = await fetchFromLocalGateway(path, options)
    
    if (response.ok) {
      const contentType = response.headers.get('content-type') || ''
      if (contentType.includes('application/json')) {
        return await response.json()
      }
      
      // 如果返回 HTML，尝试公共 Gateway
      if (contentType.includes('text/html')) {
        const publicResponse = await fetchFromPublicGateways(path, options)
        return await publicResponse.json()
      }
      
      throw new Error(`Gateway returned non-JSON content: ${contentType}`)
    }
    
    throw new Error(`Gateway returned error: ${response.status}`)
  } catch (error) {
    // 本地 Gateway 失败，尝试公共 Gateway
    try {
      const publicResponse = await fetchFromPublicGateways(path, options)
      return await publicResponse.json()
    } catch (publicError) {
      throw new Error(`All gateways failed. Local: ${error.message}, Public: ${publicError.message}`)
    }
  }
}

