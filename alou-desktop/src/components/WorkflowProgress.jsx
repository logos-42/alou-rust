import React, { useMemo } from 'react'
import './WorkflowProgress.css'

/**
 * WorkflowProgress - 工作流进度显示组件
 *
 * 显示工作流执行的实时进度，包括：
 * - 整体进度条
 * - 当前执行步骤
 * - 步骤状态列表
 * - 执行时间
 */
const WorkflowProgress = ({
  workflowId,
  progress = {},
  execution = null,
  onPause,
  onResume,
  onCancel,
  isDarkMode = false
}) => {
  const {
    status = 'idle',
    progress: progressValue = 0,
    currentStep = null,
    executionId = null
  } = progress

  // 计算执行时间
  const executionTime = useMemo(() => {
    if (!execution?.started_at) return null

    const startTime = new Date(execution.started_at * 1000)
    const endTime = execution.completed_at
      ? new Date(execution.completed_at * 1000)
      : new Date()

    const diffMs = endTime - startTime
    const seconds = Math.floor(diffMs / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)

    if (hours > 0) {
      return `${hours}:${(minutes % 60).toString().padStart(2, '0')}:${(seconds % 60).toString().padStart(2, '0')}`
    } else if (minutes > 0) {
      return `${minutes}:${(seconds % 60).toString().padStart(2, '0')}`
    } else {
      return `${seconds}s`
    }
  }, [execution])

  // 获取状态显示文本
  const getStatusText = (status) => {
    switch (status) {
      case 'starting': return '启动中'
      case 'running': return '执行中'
      case 'paused': return '已暂停'
      case 'completed': return '已完成'
      case 'failed': return '执行失败'
      case 'cancelled': return '已取消'
      default: return '等待中'
    }
  }

  // 获取状态颜色
  const getStatusColor = (status) => {
    switch (status) {
      case 'running': return '#007bff'
      case 'completed': return '#28a745'
      case 'failed': return '#dc3545'
      case 'paused': return '#ffc107'
      case 'cancelled': return '#6c757d'
      default: return '#6c757d'
    }
  }

  // 获取步骤状态图标
  const getStepIcon = (stepStatus) => {
    switch (stepStatus) {
      case 'completed': return '✅'
      case 'running': return '⚙️'
      case 'failed': return '❌'
      case 'pending': return '⏳'
      default: return '○'
    }
  }

  if (status === 'idle' && !executionId) {
    return null
  }

  return (
    <div className={`workflow-progress ${isDarkMode ? 'dark' : ''}`}>
      <div className="workflow-progress-header">
        <div className="workflow-progress-title">
          <span className="workflow-icon">🔄</span>
          <span>工作流执行</span>
          <span className="workflow-id">#{workflowId}</span>
        </div>

        <div className="workflow-progress-status">
          <span
            className="status-badge"
            style={{ backgroundColor: getStatusColor(status) }}
          >
            {getStatusText(status)}
          </span>

          {executionTime && (
            <span className="execution-time">
              {executionTime}
            </span>
          )}
        </div>
      </div>

      {/* 进度条 */}
      <div className="workflow-progress-bar">
        <div className="progress-container">
          <div
            className="progress-fill"
            style={{
              width: `${Math.max(0, Math.min(100, progressValue * 100))}%`,
              backgroundColor: getStatusColor(status)
            }}
          />
        </div>
        <span className="progress-text">
          {Math.round(progressValue * 100)}%
        </span>
      </div>

      {/* 当前步骤 */}
      {currentStep && (
        <div className="workflow-current-step">
          <span className="step-label">当前步骤:</span>
          <span className="step-name">{currentStep}</span>
        </div>
      )}

      {/* 控制按钮 */}
      {(status === 'running' || status === 'paused') && (
        <div className="workflow-controls">
          {status === 'running' && onPause && (
            <button
              type="button"
              className="control-btn pause"
              onClick={() => onPause(executionId)}
              title="暂停执行"
            >
              ⏸️ 暂停
            </button>
          )}

          {status === 'paused' && onResume && (
            <button
              type="button"
              className="control-btn resume"
              onClick={() => onResume(executionId)}
              title="恢复执行"
            >
              ▶️ 恢复
            </button>
          )}

          {onCancel && (
            <button
              type="button"
              className="control-btn cancel"
              onClick={() => onCancel(executionId)}
              title="取消执行"
            >
              ❌ 取消
            </button>
          )}
        </div>
      )}

      {/* 步骤详情 */}
      {execution?.step_results && execution.step_results.length > 0 && (
        <div className="workflow-steps">
          <div className="steps-header">执行步骤</div>
          <div className="steps-list">
            {execution.step_results.map((stepResult, index) => (
              <div key={stepResult.step_id} className="step-item">
                <div className="step-icon">
                  {getStepIcon(stepResult.status)}
                </div>
                <div className="step-info">
                  <div className="step-name">{stepResult.step_id}</div>
                  <div className="step-time">
                    {stepResult.execution_time_ms
                      ? `${stepResult.execution_time_ms}ms`
                      : '执行中...'
                    }
                  </div>
                </div>
                {stepResult.error && (
                  <div className="step-error" title={stepResult.error}>
                    ⚠️
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

export default WorkflowProgress