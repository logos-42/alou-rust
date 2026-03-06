/**
 * Avatar Service - 统一的头像管理服务
 * 支持人类用户和智能体的头像上传、切换、保护
 */

import { invoke } from '@tauri-apps/api/core'
import ipfsContentService from './ipfsContentService'
import imageProxyService from './imageProxyService'

/**
 * 头像上传结果
 */
export interface AvatarUploadResult {
  success: boolean
  avatarUrl: string
  cid?: string
  base64?: string
  source: 'ipfs' | 'local'
  error?: string
}

/**
 * 头像切换选项
 */
export interface AvatarSwitchOptions {
  useDefault?: boolean
  useDicebear?: boolean
  seed?: string
}

/**
 * 头像验证结果
 */
export interface AvatarValidationResult {
  isValid: boolean
  isAccessible: boolean
  error?: string
}

class AvatarService {
  private readonly fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'
  private readonly dicebearBaseUrl = 'https://api.dicebear.com/7.x'
  private readonly ipfsGateway = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'
  private readonly ipfsApiUrl = import.meta.env.VITE_IPFS_API_URL || 'http://127.0.0.1:5001'

  /**
   * 上传头像文件
   * @param file - 头像文件
   * @param options - 上传选项
   * @returns 上传结果
   */
  async uploadAvatar(file: File, options?: { sessionId?: string }): Promise<AvatarUploadResult> {
    console.log('[AvatarService] 开始上传头像:', file.name)

    // 验证文件
    if (!file.type.startsWith('image/')) {
      return {
        success: false,
        avatarUrl: '',
        source: 'local',
        error: '请选择图片文件',
      }
    }

    const maxSize = 5 * 1024 * 1024 // 5MB
    if (file.size > maxSize) {
      return {
        success: false,
        avatarUrl: '',
        source: 'local',
        error: '头像文件不能超过 5MB',
      }
    }

    try {
      // 尝试上传到 IPFS
      const ipfsResult = await this.uploadToIpfs(file)
      if (ipfsResult.success && ipfsResult.cid) {
        console.log('[AvatarService] 头像上传到 IPFS 成功:', ipfsResult.cid)
        return ipfsResult
      }
    } catch (ipfsError) {
      console.warn('[AvatarService] IPFS 上传失败，将使用 base64:', ipfsError)
    }

    // IPFS 不可用，使用 base64 编码
    try {
      const base64Result = await this.convertToBase64(file)
      console.log('[AvatarService] 头像使用 base64 编码')
      return base64Result
    } catch (error) {
      console.error('[AvatarService] 头像处理失败:', error)
      return {
        success: false,
        avatarUrl: '',
        source: 'local',
        error: '头像文件处理失败',
      }
    }
  }

  /**
   * 上传到 IPFS
   * @param file - 文件
   * @returns 上传结果
   */
  private async uploadToIpfs(file: File): Promise<AvatarUploadResult> {
    try {
      // 使用 Tauri 命令上传（避免 CORS 问题）
      const arrayBuffer = await file.arrayBuffer()
      const uint8Array = new Uint8Array(arrayBuffer)
      const binaryString = uint8Array.reduce((data, byte) => {
        return data + String.fromCharCode(byte)
      }, '')
      const base64 = btoa(binaryString)

      const result: { cid: string; size?: number } = await invoke('ipfs_add_base64', {
        dataBase64: base64,
        fileName: file.name || 'avatar.png',
      })

      const avatarUrl = `${this.ipfsGateway}/ipfs/${result.cid}`

      // 验证头像是否可访问
      const isValid = await this.validateAvatar(avatarUrl)
      if (!isValid.isAccessible) {
        throw new Error('IPFS 头像无法访问')
      }

      return {
        success: true,
        avatarUrl,
        cid: result.cid,
        source: 'ipfs',
      }
    } catch (error) {
      console.error('[AvatarService] IPFS 上传失败:', error)
      throw error
    }
  }

  /**
   * 转换为 Base64
   * @param file - 文件
   * @returns Base64 结果
   */
  private async convertToBase64(file: File): Promise<AvatarUploadResult> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const base64 = reader.result as string
        resolve({
          success: true,
          avatarUrl: base64,
          base64,
          source: 'local',
        })
      }
      reader.onerror = () => {
        reject(new Error('文件读取失败'))
      }
      reader.readAsDataURL(file)
    })
  }

  /**
   * 切换头像（支持多种模式）
   * @param currentAvatar - 当前头像
   * @param options - 切换选项
   * @returns 新头像 URL
   */
  async switchAvatar(currentAvatar: string | null, options?: AvatarSwitchOptions): Promise<string> {
    const opts = options || {}

    // 使用默认头像
    if (opts.useDefault) {
      return this.fallbackAvatar
    }

    // 使用 Dicebear 生成头像
    if (opts.useDicebear) {
      const seed = opts.seed || `user_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
      return `${this.dicebearBaseUrl}/identicon/svg?seed=${seed}`
    }

    // 轮换预设头像
    const presetAvatars = [
      this.fallbackAvatar,
      `${this.dicebearBaseUrl}/avataaars/svg?seed=user`,
      `${this.dicebearBaseUrl}/bottts/svg?seed=user`,
      `${this.dicebearBaseUrl}/lorelei/svg?seed=user`,
      `${this.dicebearBaseUrl}/notionists/svg?seed=user`,
    ]

    const currentIndex = presetAvatars.indexOf(currentAvatar || '')
    const nextIndex = (currentIndex + 1) % presetAvatars.length
    return presetAvatars[nextIndex]
  }

  /**
   * 验证头像是否有效
   * @param avatarUrl - 头像 URL
   * @returns 验证结果
   */
  async validateAvatar(avatarUrl: string): Promise<AvatarValidationResult> {
    if (!avatarUrl) {
      return {
        isValid: false,
        isAccessible: false,
        error: '头像 URL 为空',
      }
    }

    // Base64 头像直接有效
    if (avatarUrl.startsWith('data:')) {
      try {
        // 验证 base64 格式
        const response = await fetch(avatarUrl)
        const blob = await response.blob()
        return {
          isValid: blob.size > 0,
          isAccessible: true,
        }
      } catch (error) {
        return {
          isValid: false,
          isAccessible: false,
          error: 'Base64 头像格式错误',
        }
      }
    }

    // HTTP/HTTPS 头像
    try {
      const proxiedUrl = imageProxyService.getProxiedUrl(avatarUrl)
      const response = await fetch(proxiedUrl, { method: 'HEAD' })
      return {
        isValid: response.ok,
        isAccessible: response.ok,
      }
    } catch (error) {
      return {
        isValid: false,
        isAccessible: false,
        error: `头像无法访问：${(error as Error).message}`,
      }
    }
  }

  /**
   * 预加载头像（避免加载延迟）
   * @param avatarUrls - 头像 URL 列表
   */
  async preloadAvatars(avatarUrls: string[]): Promise<void> {
    const preloadPromises = avatarUrls.map(async (url) => {
      try {
        const proxiedUrl = imageProxyService.getProxiedUrl(url)
        await fetch(proxiedUrl, { method: 'HEAD' })
      } catch (error) {
        console.warn('[AvatarService] 头像预加载失败:', url, error)
      }
    })

    await Promise.allSettled(preloadPromises)
    console.log(`[AvatarService] 预加载了 ${avatarUrls.length} 个头像`)
  }

  /**
   * 获取默认头像
   * @returns 默认头像 URL
   */
  getFallbackAvatar(): string {
    return this.fallbackAvatar
  }

  /**
   * 生成 Dicebear 头像
   * @param seed - 种子值
   * @param style - 头像风格
   * @returns 头像 URL
   */
  generateDicebearAvatar(seed: string, style: string = 'identicon'): string {
    return `${this.dicebearBaseUrl}/${style}/svg?seed=${seed}`
  }

  /**
   * 从 IPFS CID 构建头像 URL
   * @param cid - IPFS CID
   * @returns 头像 URL
   */
  buildIpfsAvatar(cid: string): string {
    if (cid.startsWith('http')) {
      return cid
    }
    if (cid.startsWith('Qm') || cid.startsWith('bafy') || cid.startsWith('bafk')) {
      return `${this.ipfsGateway}/ipfs/${cid}`
    }
    return cid
  }

  /**
   * 清理无效头像
   * @param avatarUrl - 头像 URL
   * @returns 清理后的头像 URL
   */
  sanitizeAvatar(avatarUrl: string | null | undefined): string {
    if (!avatarUrl) {
      return this.fallbackAvatar
    }

    // 检查是否包含无效字符串
    if (
      avatarUrl.includes('undefined') ||
      avatarUrl.includes('null') ||
      avatarUrl.includes('[object Object]')
    ) {
      return this.fallbackAvatar
    }

    return avatarUrl
  }

  /**
   * 批量上传头像（用于测试）
   * @param files - 文件列表
   * @returns 上传结果数组
   */
  async batchUploadAvatars(files: File[]): Promise<AvatarUploadResult[]> {
    const results: AvatarUploadResult[] = []

    for (const file of files) {
      try {
        const result = await this.uploadAvatar(file)
        results.push(result)
      } catch (error) {
        results.push({
          success: false,
          avatarUrl: '',
          source: 'local',
          error: (error as Error).message,
        })
      }
    }

    return results
  }

  /**
   * 获取服务统计信息
   */
  getStats(): {
    ipfsGateway: string
    dicebearBaseUrl: string
    supportedStyles: string[]
  } {
    return {
      ipfsGateway: this.ipfsGateway,
      dicebearBaseUrl: this.dicebearBaseUrl,
      supportedStyles: [
        'identicon',
        'avataaars',
        'bottts',
        'lorelei',
        'notionists',
        'fun-emoji',
        'shapes',
      ],
    }
  }
}

// 创建单例实例
const avatarService = new AvatarService()

export default avatarService
export { AvatarService }
