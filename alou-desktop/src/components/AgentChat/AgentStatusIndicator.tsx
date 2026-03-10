/**
 * Agent 执行状态指示器
 * 显示 Agent 当前执行状态和待处理指令数量
 */

import React from 'react'
import { useAgentHookIntegration } from './useAgentHookIntegration'
import './AgentStatusIndicator.css'

interface AgentStatusIndicatorProps {
  activeChannelId: string | null
  sessionId: string
}

export const AgentStatusIndicator: React.FC<AgentStatusIndicatorProps> = ({
  activeChannelId,
  sessionId,
}) => {
  const {
    agentStatus,
    hasPendingInstructions,
    isExecuting,
    pauseAgent,
    resumeAgent,
    cancelAgent,
  } = useAgentHookIntegration({
    activeChannelId,
    sessionId,
    isLoading: false, // 由 Hook 内部判断
  })

  if (!activeChannelId || !agentStatus) {
    return null
  }

  const getStatusText = () => {
    switch (agentStatus.status) {
      case 'executing':
        return `执行中 ${Math.round(agentStatus.progress)}%`
      case 'paused':
        return '已暂停'
      case 'completed':
        return '已完成'
      case 'cancelled':
        return '已取消'
      case 'failed':
        return '执行失败'
      case 'idle':
        return '空闲'
      default:
        return agentStatus.status
    }
  }

  const getStatusClass = () => {
    return `status-dot ${agentStatus.status}`
  }

  const handlePauseResume = async () => {
    if (agentStatus.status === 'paused') {
      await resumeAgent()
    } else {
      await pauseAgent('用户要求暂停')
    }
  }

  const handleCancel = async () => {
    if (window.confirm('确定要取消当前执行吗？')) {
      await cancelAgent('用户取消')
    }
  }

  return (
    <div className="agent-status-indicator">
      <div className="status-left">
        <div className={getStatusClass()} title={agentStatus.status}></div>
        <span className="status-text">{getStatusText()}</span>
      </div>

      {hasPendingInstructions && (
        <div className="pending-badge" title="待处理指令">
          💬 {agentStatus.pending_instructions}
        </div>
      )}

      {isExecuting && (
        <div className="status-actions">
          <button
            type="button"
            className="action-btn pause-btn"
            onClick={handlePauseResume}
            title={agentStatus.status === 'paused' ? '恢复执行' : '暂停执行'}
          >
            {agentStatus.status === 'paused' ? '▶️' : '⏸️'}
          </button>
          <button
            type="button"
            className="action-btn cancel-btn"
            onClick={handleCancel}
            title="取消执行"
          >
            ⏹️
          </button>
        </div>
      )}
    </div>
  )
}
