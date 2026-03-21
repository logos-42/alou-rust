/**
 * 异步任务状态指示器组件
 * 
 * 显示异步任务（如视频生成）的实时状态
 */

import React from 'react'
import { Loader2, CheckCircle2, XCircle, Clock, Video } from 'lucide-react'
import type { AsyncTask } from '../hooks/useAsyncTaskEvents'
import './AsyncTaskIndicator.css'

interface AsyncTaskIndicatorProps {
  task: AsyncTask
  onCancel?: (taskId: string) => void
}

// 获取工具图标
const getToolIcon = (toolName: string) => {
  switch (toolName) {
    case 'generate_video':
      return <Video size={20} />
    default:
      return <Loader2 size={20} />
  }
}

// 获取工具显示名称
const getToolDisplayName = (toolName: string): string => {
  const nameMap: Record<string, string> = {
    'generate_video': '视频生成',
    'generate_image': '图片生成',
    'generate_audio': '音频生成',
  }
  return nameMap[toolName] || toolName
}

// 获取状态图标
const getStatusIcon = (status: string) => {
  switch (status) {
    case 'completed':
      return <CheckCircle2 size={18} className="status-icon-success" />
    case 'failed':
    case 'timeout':
      return <XCircle size={18} className="status-icon-error" />
    case 'cancelled':
      return <XCircle size={18} className="status-icon-cancelled" />
    case 'pending':
      return <Clock size={18} className="status-icon-pending" />
    default:
      return <Loader2 size={18} className="status-icon-loading animate-spin" />
  }
}

// 获取状态文本
const getStatusText = (status: string): string => {
  const statusMap: Record<string, string> = {
    'processing': '处理中...',
    'pending': '排队中...',
    'completed': '已完成',
    'failed': '失败',
    'timeout': '超时',
    'cancelled': '已取消',
  }
  return statusMap[status] || status
}

// 格式化时间
const formatElapsed = (seconds: number): string => {
  if (seconds < 60) {
    return `${seconds}秒`
  }
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}分${remainingSeconds}秒`
}

export const AsyncTaskIndicator: React.FC<AsyncTaskIndicatorProps> = ({
  task,
  onCancel,
}) => {
  const isActive = task.status === 'processing' || task.status === 'pending'
  const isCompleted = task.status === 'completed'
  const isFailed = task.status === 'failed' || task.status === 'timeout'

  return (
    <div className={`async-task-card ${isCompleted ? 'completed' : ''} ${isFailed ? 'failed' : ''}`}>
      <div className="async-task-header">
        <div className="async-task-icon">
          {getToolIcon(task.toolName)}
        </div>
        <div className="async-task-info">
          <div className="async-task-title">
            {getToolDisplayName(task.toolName)}
            {isActive && <span className="pulse-dot" />}
          </div>
          <div className="async-task-subtitle">
            <span className="async-task-status">
              {getStatusIcon(task.status)}
              <span className="status-text">{getStatusText(task.status)}</span>
            </span>
            {isActive && (
              <span className="async-task-elapsed">
                已等待 {formatElapsed(task.elapsed)}
              </span>
            )}
            {task.pollCount !== undefined && task.pollCount > 0 && (
              <span className="async-task-poll-count">
                查询 {task.pollCount} 次
              </span>
            )}
          </div>
        </div>
        {isActive && onCancel && (
          <button
            className="async-task-cancel-btn"
            onClick={() => onCancel(task.taskId)}
            title="取消任务"
          >
            <XCircle size={16} />
          </button>
        )}
      </div>

      {/* 进度条 */}
      {isActive && (
        <div className="async-task-progress-container">
          <div className="async-task-progress-bar">
            <div
              className="async-task-progress-fill"
              style={{
                width: `${task.progress || 0}%`,
                animationDuration: task.status === 'pending' ? '3s' : '2s',
              }}
            />
          </div>
          {task.progress !== undefined && task.progress > 0 && (
            <span className="async-task-progress-text">{Math.round(task.progress)}%</span>
          )}
        </div>
      )}

      {/* 结果展示 */}
      {isCompleted && task.result && (
        <div className="async-task-result">
          {task.result.url && (
            <div className="async-task-media">
              {task.toolName === 'generate_video' && (
                <video
                  src={task.result.url}
                  controls
                  className="async-task-video"
                  poster={task.result.thumbnail_url}
                />
              )}
              {task.toolName === 'generate_image' && (
                <img
                  src={task.result.url}
                  alt="生成的图片"
                  className="async-task-image"
                />
              )}
              {task.toolName === 'generate_audio' && (
                <audio
                  src={task.result.url}
                  controls
                  className="async-task-audio"
                />
              )}
            </div>
          )}
          {task.result.file_path && (
            <div className="async-task-file-path">
              文件已保存: {task.result.file_path}
            </div>
          )}
        </div>
      )}

      {/* 错误信息 */}
      {isFailed && task.error && (
        <div className="async-task-error">
          <XCircle size={16} />
          <span>{task.error}</span>
        </div>
      )}
    </div>
  )
}

export default AsyncTaskIndicator
