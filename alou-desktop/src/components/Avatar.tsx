/**
 * Avatar Component - 统一的头像显示组件
 * 支持人类用户和智能体，自动处理 IPFS 头像加载和错误恢复
 */

import React, { useState, useEffect } from 'react'
import avatarService from '@/services/avatarService'
import imageProxyService from '@/services/imageProxyService'
import './Avatar.css'

/**
 * Avatar 组件属性
 */
interface AvatarProps {
  /** 头像 URL */
  src?: string
  /** 备用名称（用于生成 Dicebear 头像） */
  name?: string
  /** 头像尺寸 */
  size?: 'small' | 'medium' | 'large' | number
  /** 点击回调 */
  onClick?: () => void
  /** 加载失败时的备用头像 */
  fallback?: string
  /** 是否圆形 */
  rounded?: boolean
  /** 自定义类名 */
  className?: string
  /** 是否使用代理 */
  useProxy?: boolean
  /** 是否显示加载状态 */
  showLoading?: boolean
}

/**
 * Avatar 组件
 */
const Avatar: React.FC<AvatarProps> = ({
  src,
  name,
  size = 'medium',
  onClick,
  fallback,
  rounded = true,
  className = '',
  useProxy = true,
  showLoading = false,
}) => {
  const [isLoading, setIsLoading] = useState(true)
  const [hasError, setHasError] = useState(false)
  const [currentSrc, setCurrentSrc] = useState<string>('')

  // 处理头像 URL
  useEffect(() => {
    let avatarUrl = src

    // 清理无效头像
    if (!avatarUrl || avatarUrl.includes('undefined') || avatarUrl.includes('null')) {
      avatarUrl = fallback || avatarService.getFallbackAvatar()
    }

    // 使用代理（避免 CORS 问题）
    if (useProxy && avatarUrl && !avatarUrl.startsWith('data:')) {
      try {
        avatarUrl = imageProxyService.getProxiedUrl(avatarUrl)
      } catch (error) {
        console.warn('[Avatar] 代理处理失败，使用原图:', error)
      }
    }

    setCurrentSrc(avatarUrl)
    setIsLoading(true)
    setHasError(false)
  }, [src, fallback, useProxy])

  // 处理加载完成
  const handleLoad = () => {
    setIsLoading(false)
  }

  // 处理加载失败
  const handleError = () => {
    console.warn('[Avatar] 头像加载失败:', currentSrc)
    setIsLoading(false)
    setHasError(true)

    // 使用备用头像
    if (!hasError) {
      const fallbackUrl = fallback || (name ? avatarService.generateDicebearAvatar(name) : avatarService.getFallbackAvatar())
      setCurrentSrc(fallbackUrl)
    }
  }

  // 计算尺寸
  const getSize = () => {
    if (typeof size === 'number') return size
    switch (size) {
      case 'small':
        return 32
      case 'medium':
        return 40
      case 'large':
        return 64
      default:
        return 40
    }
  }

  const avatarSize = getSize()

  return (
    <div
      className={`avatar-container ${rounded ? 'avatar-rounded' : ''} ${className}`}
      style={{
        width: avatarSize,
        height: avatarSize,
        minWidth: avatarSize,
        minHeight: avatarSize,
      }}
      onClick={onClick}
    >
      {isLoading && showLoading && (
        <div className="avatar-loading">
          <div className="avatar-loading-spinner"></div>
        </div>
      )}

      <img
        src={currentSrc}
        alt={name || 'avatar'}
        className={`avatar-image ${isLoading ? 'avatar-loading-image' : ''}`}
        onLoad={handleLoad}
        onError={handleError}
        loading="lazy"
        crossOrigin="anonymous"
      />

      {hasError && (
        <div className="avatar-error-indicator">⚠️</div>
      )}
    </div>
  )
}

/**
 * AvatarGroup 组件 - 头像组
 */
interface AvatarGroupProps {
  /** 头像列表 */
  avatars: Array<{
    src?: string
    name?: string
    id?: string
  }>
  /** 最大显示数量 */
  max?: number
  /** 头像尺寸 */
  size?: 'small' | 'medium' | 'large'
  /** 是否重叠显示 */
  overlap?: boolean
}

const AvatarGroup: React.FC<AvatarGroupProps> = ({
  avatars,
  max = 5,
  size = 'medium',
  overlap = true,
}) => {
  const displayAvatars = avatars.slice(0, max)
  const remainingCount = avatars.length - max

  return (
    <div className={`avatar-group ${overlap ? 'avatar-group-overlap' : ''}`}>
      {displayAvatars.map((avatar, index) => (
        <Avatar
          key={avatar.id || index}
          src={avatar.src}
          name={avatar.name}
          size={size}
          className="avatar-group-item"
        />
      ))}

      {remainingCount > 0 && (
        <div
          className={`avatar-group-remaining avatar-size-${size}`}
          style={{
            width: typeof size === 'number' ? size : undefined,
            height: typeof size === 'number' ? size : undefined,
          }}
        >
          +{remainingCount}
        </div>
      )}
    </div>
  )
}

export { Avatar, AvatarGroup }
export default Avatar
