/**
 * 媒体消息查看器组件
 * 用于显示图片、音频、视频消息
 */

import React from 'react'
import './MediaMessageViewer.css'

interface MediaMessageViewerProps {
  type: 'image' | 'audio' | 'video'
  filePath?: string
  url?: string
  metadata?: {
    width?: number
    height?: number
    duration?: number
    format?: string
    prompt?: string
  }
  onLoad?: () => void
  onError?: (error: string) => void
}

const MediaMessageViewer: React.FC<MediaMessageViewerProps> = ({
  type,
  filePath,
  url,
  metadata,
  onLoad,
  onError,
}) => {
  // 获取媒体源（优先使用本地文件路径，其次使用 URL）
  const getMediaSource = () => {
    if (filePath) {
      // 本地文件路径转换为 file:// URL
      return `file://${filePath}`
    }
    return url
  }

  const handleError = (error: string) => {
    console.error(`[MediaMessageViewer] ${type} 加载失败:`, error)
    onError?.(error)
  }

  const handleLoad = () => {
    console.log(`[MediaMessageViewer] ${type} 加载成功`)
    onLoad?.()
  }

  // 渲染图片
  if (type === 'image') {
    return (
      <div className="media-viewer media-viewer-image">
        <img
          src={getMediaSource()}
          alt={metadata?.prompt || '生成的图片'}
          className="media-image"
          onLoad={handleLoad}
          onError={(e) => handleError('图片加载失败')}
          style={{
            maxWidth: '100%',
            maxHeight: '400px',
            objectFit: 'contain',
          }}
        />
        {metadata?.prompt && (
          <div className="media-caption">
            <span className="caption-label">提示词:</span>
            <span className="caption-text">{metadata.prompt}</span>
          </div>
        )}
      </div>
    )
  }

  // 渲染音频
  if (type === 'audio') {
    return (
      <div className="media-viewer media-viewer-audio">
        <div className="audio-player">
          <audio
            controls
            src={getMediaSource()}
            onLoadedMetadata={handleLoad}
            onError={(e) => handleError('音频加载失败')}
          >
            您的浏览器不支持音频播放
          </audio>
        </div>
        {metadata?.duration && (
          <div className="media-info">
            <span className="info-label">时长:</span>
            <span className="info-value">{metadata.duration.toFixed(1)} 秒</span>
          </div>
        )}
        {metadata?.format && (
          <div className="media-info">
            <span className="info-label">格式:</span>
            <span className="info-value">{metadata.format}</span>
          </div>
        )}
      </div>
    )
  }

  // 渲染视频
  if (type === 'video') {
    return (
      <div className="media-viewer media-viewer-video">
        <div className="video-player">
          <video
            controls
            src={getMediaSource()}
            width="100%"
            height="auto"
            onLoadedMetadata={handleLoad}
            onError={(e) => handleError('视频加载失败')}
            poster={metadata?.prompt ? undefined : undefined}
          >
            您的浏览器不支持视频播放
          </video>
        </div>
        {metadata?.prompt && (
          <div className="media-caption">
            <span className="caption-label">提示词:</span>
            <span className="caption-text">{metadata.prompt}</span>
          </div>
        )}
        {metadata?.duration && (
          <div className="media-info">
            <span className="info-label">时长:</span>
            <span className="info-value">{metadata.duration.toFixed(1)} 秒</span>
          </div>
        )}
        {metadata?.format && (
          <div className="media-info">
            <span className="info-label">格式:</span>
            <span className="info-value">{metadata.format}</span>
          </div>
        )}
      </div>
    )
  }

  // 未知类型
  return (
    <div className="media-viewer media-viewer-unknown">
      <div className="unknown-media">
        <span className="unknown-icon">📄</span>
        <span className="unknown-text">不支持的媒体类型：{type}</span>
      </div>
    </div>
  )
}

export default MediaMessageViewer
