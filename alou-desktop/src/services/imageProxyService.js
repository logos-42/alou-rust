/**
 * 图片代理服务
 * 在Tauri环境中将外部图片URL转换为data URL
 */

class ImageProxyService {
  constructor() {
    this.cache = new Map()
    this.loadingPromises = new Map()
  }

  /**
   * 获取图片的data URL
   * @param {string} imageUrl - 图片URL
   * @returns {Promise<string>} data URL
   */
  async getImageDataUrl(imageUrl) {
    if (!imageUrl) {
      return null
    }

    // 如果已经是data URL，直接返回
    if (imageUrl.startsWith('data:')) {
      return imageUrl
    }

    // 检查缓存
    if (this.cache.has(imageUrl)) {
      return this.cache.get(imageUrl)
    }

    // 检查是否正在加载
    if (this.loadingPromises.has(imageUrl)) {
      return this.loadingPromises.get(imageUrl)
    }

    // 开始加载
    const loadPromise = this._loadImageAsDataUrl(imageUrl)
    this.loadingPromises.set(imageUrl, loadPromise)

    try {
      const dataUrl = await loadPromise
      if (dataUrl) {
        this.cache.set(imageUrl, dataUrl)
      }
      return dataUrl
    } catch (error) {
      console.error('[ImageProxyService] 加载图片失败:', imageUrl, error)
      return null
    } finally {
      this.loadingPromises.delete(imageUrl)
    }
  }

  /**
   * 将图片URL转换为data URL
   * @param {string} imageUrl - 图片URL
   * @returns {Promise<string>} data URL
   */
  async _loadImageAsDataUrl(imageUrl) {
    // 检查是否在Tauri环境中
    if (typeof window !== 'undefined' && window.__TAURI__) {
      return this._loadImageInTauri(imageUrl)
    } else {
      return this._loadImageInBrowser(imageUrl)
    }
  }

  /**
   * 在Tauri环境中加载图片
   * @param {string} imageUrl - 图片URL
   * @returns {Promise<string>} data URL
   */
  async _loadImageInTauri(imageUrl) {
    try {
      console.log('[ImageProxyService] 在Tauri中加载图片:', imageUrl)
      
      // 检查Tauri HTTP插件是否可用
      if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.http) {
        console.log('[ImageProxyService] 使用Tauri HTTP插件')
        
        // 使用Tauri HTTP插件
        const { fetch: tauriFetch } = window.__TAURI__.http
        const response = await tauriFetch(imageUrl, {
          method: 'GET',
          headers: {
            'User-Agent': 'Alou-Desktop/1.0'
          }
        })

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`)
        }

        // 获取图片数据
        const arrayBuffer = await response.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)
        
        // 转换为base64
        let binary = ''
        const chunkSize = 8192
        for (let i = 0; i < uint8Array.length; i += chunkSize) {
          const chunk = uint8Array.subarray(i, i + chunkSize)
          binary += String.fromCharCode.apply(null, chunk)
        }
        const base64 = btoa(binary)
        
        // 检测图片类型
        const contentType = response.headers.get('content-type') || this._detectImageType(uint8Array) || 'image/png'
        
        const dataUrl = `data:${contentType};base64,${base64}`
        console.log('[ImageProxyService] Tauri HTTP插件加载成功，大小:', dataUrl.length)
        
        return dataUrl
      } else {
        console.log('[ImageProxyService] Tauri HTTP插件不可用，使用原生fetch')
        
        // 使用原生fetch（Tauri 2.0支持）
        const response = await fetch(imageUrl, {
          method: 'GET',
          headers: {
            'User-Agent': 'Alou-Desktop/1.0'
          }
        })

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`)
        }

        // 获取图片数据
        const arrayBuffer = await response.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)
        
        // 转换为base64
        let binary = ''
        const chunkSize = 8192
        for (let i = 0; i < uint8Array.length; i += chunkSize) {
          const chunk = uint8Array.subarray(i, i + chunkSize)
          binary += String.fromCharCode.apply(null, chunk)
        }
        const base64 = btoa(binary)
        
        // 检测图片类型
        const contentType = response.headers.get('content-type') || this._detectImageType(uint8Array) || 'image/png'
        
        const dataUrl = `data:${contentType};base64,${base64}`
        console.log('[ImageProxyService] 原生fetch加载成功，大小:', dataUrl.length)
        
        return dataUrl
      }
    } catch (error) {
      console.error('[ImageProxyService] Tauri加载失败:', error)
      // 尝试浏览器方式作为fallback
      return this._loadImageInBrowser(imageUrl)
    }
  }

  /**
   * 在浏览器环境中加载图片
   * @param {string} imageUrl - 图片URL
   * @returns {Promise<string>} data URL
   */
  async _loadImageInBrowser(imageUrl) {
    return new Promise((resolve, reject) => {
      const img = new Image()
      
      // 设置跨域
      img.crossOrigin = 'anonymous'
      
      img.onload = () => {
        try {
          // 创建canvas
          const canvas = document.createElement('canvas')
          const ctx = canvas.getContext('2d')
          
          // 设置canvas尺寸
          canvas.width = img.width
          canvas.height = img.height
          
          // 绘制图片
          ctx.drawImage(img, 0, 0)
          
          // 转换为data URL
          const dataUrl = canvas.toDataURL('image/png')
          console.log('[ImageProxyService] 浏览器加载成功，大小:', dataUrl.length)
          resolve(dataUrl)
        } catch (error) {
          reject(error)
        }
      }
      
      img.onerror = () => {
        reject(new Error(`Failed to load image: ${imageUrl}`))
      }
      
      // 开始加载
      img.src = imageUrl
    })
  }

  /**
   * 检测图片类型
   * @param {Uint8Array} bytes - 图片字节数据
   * @returns {string} MIME类型
   */
  _detectImageType(bytes) {
    if (bytes.length < 4) return null
    
    // PNG: 89 50 4E 47
    if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4E && bytes[3] === 0x47) {
      return 'image/png'
    }
    
    // JPEG: FF D8 FF
    if (bytes[0] === 0xFF && bytes[1] === 0xD8 && bytes[2] === 0xFF) {
      return 'image/jpeg'
    }
    
    // GIF: 47 49 46 38
    if (bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x38) {
      return 'image/gif'
    }
    
    // WebP: 52 49 46 46 ... 57 45 42 50
    if (bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
        bytes.length >= 12 && bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50) {
      return 'image/webp'
    }
    
    return 'image/png' // 默认
  }

  /**
   * 预加载图片
   * @param {Array<string>} imageUrls - 图片URL数组
   */
  async preloadImages(imageUrls) {
    const promises = imageUrls
      .filter(url => url && !url.startsWith('data:'))
      .map(url => this.getImageDataUrl(url).catch(error => {
        console.warn('[ImageProxyService] 预加载失败:', url, error)
        return null
      }))
    
    await Promise.all(promises)
  }

  /**
   * 清除缓存
   * @param {string} imageUrl - 可选，指定要清除的图片URL
   */
  clearCache(imageUrl = null) {
    if (imageUrl) {
      this.cache.delete(imageUrl)
    } else {
      this.cache.clear()
    }
  }

  /**
   * 获取缓存大小
   * @returns {number} 缓存条目数量
   */
  getCacheSize() {
    return this.cache.size
  }
}

// 创建单例
const imageProxyService = new ImageProxyService()

export default imageProxyService