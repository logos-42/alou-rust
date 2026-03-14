/**
 * Alou 自主循环控制面板
 *
 * 提供自主智能体主循环的启动、停止、暂停、恢复等控制功能
 */

import React, { useEffect, useState, useCallback } from 'react'
import autonomousLoopService from '@/services/autonomousLoopService'

/**
 * 自主循环状态接口
 */
interface AutonomousLoopState {
  is_running: boolean
  is_paused: boolean
  tasks_completed: number
  tasks_failed: number
  total_iterations: number
  current_task_id: string | null
  config: {
    heartbeat_interval_seconds: number
    task_check_interval_seconds: number
    memory_save_interval_seconds: number
    progress_report_interval_seconds: number
  }
  last_heartbeat: number
}

/**
 * 自主循环配置接口
 */
interface AutonomousLoopConfig {
  heartbeat_interval_seconds: number
  task_check_interval_seconds: number
  memory_save_interval_seconds: number
  progress_report_interval_seconds: number
}

// 图标组件
interface StatusIconProps {
  isRunning: boolean
  isPaused: boolean
}

const StatusIcon: React.FC<StatusIconProps> = ({ isRunning, isPaused }) => {
  if (!isRunning) {
    return (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ff4444" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="15" y1="9" x2="9" y2="15" />
        <line x1="9" y1="9" x2="15" y2="15" />
      </svg>
    )
  }
  if (isPaused) {
    return (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffaa00" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="10" y1="15" x2="10" y2="9" />
        <line x1="14" y1="15" x2="14" y2="9" />
      </svg>
    )
  }
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#44ff44" strokeWidth="2">
      <circle cx="12" cy="12" r="10" />
      <polyline points="12 6 12 12 16 14" />
    </svg>
  )
}

// 按钮组件
interface ControlButtonProps {
  onClick: () => void
  disabled?: boolean
  children: React.ReactNode
  variant?: 'primary' | 'danger' | 'warning' | 'success'
}

const ControlButton: React.FC<ControlButtonProps> = ({
  onClick,
  disabled = false,
  children,
  variant = 'primary'
}) => {
  const colors: Record<string, string> = {
    primary: '#3b82f6',
    danger: '#ef4444',
    warning: '#f59e0b',
    success: '#10b981',
  }

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: '8px 16px',
        borderRadius: '6px',
        border: 'none',
        backgroundColor: disabled ? '#6b7280' : colors[variant],
        color: 'white',
        cursor: disabled ? 'not-allowed' : 'pointer',
        fontSize: '14px',
        fontWeight: 500,
        transition: 'all 0.2s',
      }}
    >
      {children}
    </button>
  )
}

// 统计卡片组件
interface StatCardProps {
  label: string
  value: number | string
  color: string
}

const StatCard: React.FC<StatCardProps> = ({ label, value, color }) => {
  return (
    <div style={{
      backgroundColor: '#374151',
      borderRadius: '8px',
      padding: '12px',
      textAlign: 'center',
    }}>
      <div style={{ fontSize: '24px', fontWeight: 700, color }}>
        {value}
      </div>
      <div style={{ fontSize: '12px', color: '#9ca3af', marginTop: '4px' }}>
        {label}
      </div>
    </div>
  )
}

// 配置项组件
interface ConfigItemProps {
  label: string
  value: string
}

const ConfigItem: React.FC<ConfigItemProps> = ({ label, value }) => {
  return (
    <div style={{
      display: 'flex',
      justifyContent: 'space-between',
      padding: '4px 0',
    }}>
      <span style={{ color: '#9ca3af' }}>{label}</span>
      <span style={{ color: '#e5e7eb' }}>{value}</span>
    </div>
  )
}

/**
 * AutonomousLoopPanel 组件
 * 自主循环控制面板，用于管理 AI 自动执行任务
 */
const AutonomousLoopPanel: React.FC = () => {
  const [state, setState] = useState<AutonomousLoopState | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // 刷新状态
  const refreshState = useCallback(async () => {
    try {
      const newState = await autonomousLoopService.getState() as AutonomousLoopState
      setState(newState)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取状态失败')
    } finally {
      setLoading(false)
    }
  }, [])

  // 启动
  const handleStart = async () => {
    setLoading(true)
    const result = await autonomousLoopService.start() as { success: boolean; message?: string }
    if (!result.success) {
      setError(result.message || '启动失败')
    }
    await refreshState()
    setLoading(false)
  }

  // 停止
  const handleStop = async () => {
    setLoading(true)
    const result = await autonomousLoopService.stop() as { success: boolean; message?: string }
    if (!result.success) {
      setError(result.message || '停止失败')
    }
    await refreshState()
    setLoading(false)
  }

  // 暂停
  const handlePause = async () => {
    setLoading(true)
    const result = await autonomousLoopService.pause() as { success: boolean; message?: string }
    if (!result.success) {
      setError(result.message || '暂停失败')
    }
    await refreshState()
    setLoading(false)
  }

  // 恢复
  const handleResume = async () => {
    setLoading(true)
    const result = await autonomousLoopService.resume() as { success: boolean; message?: string }
    if (!result.success) {
      setError(result.message || '恢复失败')
    }
    await refreshState()
    setLoading(false)
  }

  // 初始加载
  useEffect(() => {
    refreshState()
    // 每3秒刷新状态
    const interval = setInterval(refreshState, 3000)
    return () => clearInterval(interval)
  }, [refreshState])

  // 渲染
  if (loading && !state) {
    return (
      <div style={{
        padding: '20px',
        textAlign: 'center',
        color: '#9ca3af'
      }}>
        加载中...
      </div>
    )
  }

  if (!state) {
    return (
      <div style={{
        padding: '20px',
        textAlign: 'center',
        color: '#ef4444'
      }}>
        {error || '未知错误'}
      </div>
    )
  }

  const statusText = autonomousLoopService.getStatusText(state)
  const statusColor = autonomousLoopService.getStatusColor(state)

  return (
    <div style={{
      padding: '20px',
      backgroundColor: '#1f2937',
      borderRadius: '12px',
      color: 'white',
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    }}>
      {/* 标题 */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        marginBottom: '20px',
      }}>
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" strokeWidth="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z" />
          <path d="M2 17l10 5 10-5" />
          <path d="M2 12l10 5 10-5" />
        </svg>
        <h2 style={{ margin: 0, fontSize: '20px', fontWeight: 600 }}>
          🤖 自主循环控制
        </h2>
        <StatusIcon isRunning={state.is_running} isPaused={state.is_paused} />
        <span style={{
          color: statusColor,
          fontWeight: 600,
          fontSize: '14px',
        }}>
          {statusText}
        </span>
      </div>

      {/* 统计信息 */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(3, 1fr)',
        gap: '16px',
        marginBottom: '20px',
      }}>
        <StatCard
          label="完成任务"
          value={state.tasks_completed}
          color="#10b981"
        />
        <StatCard
          label="失败任务"
          value={state.tasks_failed}
          color="#ef4444"
        />
        <StatCard
          label="循环次数"
          value={state.total_iterations}
          color="#3b82f6"
        />
      </div>

      {/* 当前任务 */}
      {state.current_task_id && (
        <div style={{
          backgroundColor: '#374151',
          borderRadius: '8px',
          padding: '12px',
          marginBottom: '20px',
        }}>
          <div style={{ fontSize: '12px', color: '#9ca3af', marginBottom: '4px' }}>
            当前任务
          </div>
          <div style={{ fontSize: '14px', fontFamily: 'monospace' }}>
            {state.current_task_id}
          </div>
        </div>
      )}

      {/* 控制按钮 */}
      <div style={{
        display: 'flex',
        gap: '12px',
        flexWrap: 'wrap',
        marginBottom: '20px',
      }}>
        {!state.is_running ? (
          <ControlButton
            onClick={handleStart}
            variant="success"
          >
            ▶️ 启动
          </ControlButton>
        ) : (
          <>
            <ControlButton
              onClick={state.is_paused ? handleResume : handlePause}
              variant={state.is_paused ? 'success' : 'warning'}
            >
              {state.is_paused ? '▶️ 恢复' : '⏸️ 暂停'}
            </ControlButton>
            <ControlButton
              onClick={handleStop}
              variant="danger"
            >
              🛑 停止
            </ControlButton>
          </>
        )}
      </div>

      {/* 配置信息 */}
      <div style={{
        borderTop: '1px solid #374151',
        paddingTop: '16px',
      }}>
        <div style={{
          fontSize: '12px',
          color: '#9ca3af',
          marginBottom: '8px',
        }}>
          配置
        </div>
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(2, 1fr)',
          gap: '8px',
          fontSize: '13px',
        }}>
          <ConfigItem label="心跳间隔" value={`${state.config.heartbeat_interval_seconds}秒`} />
          <ConfigItem label="任务检查" value={`${state.config.task_check_interval_seconds}秒`} />
          <ConfigItem label="记忆保存" value={`${state.config.memory_save_interval_seconds}秒`} />
          <ConfigItem label="进度汇报" value={`${state.config.progress_report_interval_seconds}秒`} />
        </div>
      </div>

      {/* 最后心跳 */}
      <div style={{
        marginTop: '16px',
        fontSize: '12px',
        color: '#6b7280',
      }}>
        最后心跳: {autonomousLoopService.formatTimestamp(state.last_heartbeat)}
      </div>

      {/* 错误提示 */}
      {error && (
        <div style={{
          marginTop: '16px',
          padding: '12px',
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid #ef4444',
          borderRadius: '8px',
          color: '#ef4444',
          fontSize: '13px',
        }}>
          ❌ {error}
        </div>
      )}
    </div>
  )
}

export default AutonomousLoopPanel
