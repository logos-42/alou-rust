/**
 * Kappa Loop 卡帕斯循环控制面板
 *
 * 卡帕斯循环（自修复循环测试）：Test → Detect → Repair → Verify → Learn → Repeat
 * 整合 Hyperagent AutoResearch 引擎，实现 AI 驱动的自动自修复
 */

import React, { useEffect, useState, useCallback } from 'react'
import kappaLoopService, {
  KappaLoopState,
  KappaExperimentSummary,
  getExperimentOutcomeLabel,
} from '@/services/kappaLoopService'

// 统计卡片
interface StatCardProps {
  label: string
  value: number | string
  color: string
}

const StatCard: React.FC<StatCardProps> = ({ label, value, color }) => (
  <div style={{
    backgroundColor: '#374151',
    borderRadius: '8px',
    padding: '12px',
    textAlign: 'center',
  }}>
    <div style={{ fontSize: '24px', fontWeight: 700, color }}>{value}</div>
    <div style={{ fontSize: '12px', color: '#9ca3af', marginTop: '4px' }}>{label}</div>
  </div>
)

// 控制按钮
interface ControlButtonProps {
  onClick: () => void
  disabled?: boolean
  children: React.ReactNode
  variant?: 'primary' | 'danger' | 'warning' | 'success'
}

const ControlButton: React.FC<ControlButtonProps> = ({
  onClick, disabled = false, children, variant = 'primary'
}) => {
  const colors: Record<string, string> = {
    primary: '#3b82f6', danger: '#ef4444', warning: '#f59e0b', success: '#10b981',
  }
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: '8px 16px', borderRadius: '6px', border: 'none',
        backgroundColor: disabled ? '#6b7280' : colors[variant],
        color: 'white', cursor: disabled ? 'not-allowed' : 'pointer',
        fontSize: '14px', fontWeight: 500, transition: 'all 0.2s',
      }}
    >
      {children}
    </button>
  )
}

// 健康评分环形图
const HealthGauge: React.FC<{ score: number }> = ({ score }) => {
  const color = kappaLoopService.getHealthColor(score)
  const radius = 40
  const circumference = 2 * Math.PI * radius
  const offset = circumference - (score / 100) * circumference

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
      <svg width="100" height="100" viewBox="0 0 100 100">
        <circle cx="50" cy="50" r={radius} fill="none" stroke="#374151" strokeWidth="8" />
        <circle
          cx="50" cy="50" r={radius} fill="none" stroke={color} strokeWidth="8"
          strokeDasharray={circumference} strokeDashoffset={offset}
          strokeLinecap="round" transform="rotate(-90 50 50)"
          style={{ transition: 'stroke-dashoffset 0.5s ease' }}
        />
        <text x="50" y="50" textAnchor="middle" dominantBaseline="central"
          fill="white" fontSize="20" fontWeight="700">
          {score.toFixed(0)}
        </text>
      </svg>
      <div style={{ fontSize: '12px', color: '#9ca3af', marginTop: '4px' }}>健康评分</div>
    </div>
  )
}

// 实验日志条目
const ExperimentItem: React.FC<{ exp: KappaExperimentSummary }> = ({ exp }) => {
  const { label, color } = getExperimentOutcomeLabel(exp.outcome)
  return (
    <div style={{
      backgroundColor: '#374151', borderRadius: '6px', padding: '10px',
      marginBottom: '8px', fontSize: '13px',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
        <span style={{ color: '#e5e7eb' }}>
          #{exp.iteration} {exp.file}
        </span>
        <span style={{ color, fontWeight: 600, fontSize: '12px' }}>{label}</span>
      </div>
      <div style={{ color: '#9ca3af', fontSize: '12px', lineHeight: '1.4' }}>
        {exp.hypothesis}
      </div>
      <div style={{ color: '#6b7280', fontSize: '11px', marginTop: '4px' }}>
        测试: {exp.tests_before[0]}/{exp.tests_before[1]} → {exp.tests_after[0]}/{exp.tests_after[1]}
      </div>
    </div>
  )
}

/**
 * KappaLoopPanel 组件
 */
const KappaLoopPanel: React.FC = () => {
  const [state, setState] = useState<KappaLoopState | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refreshState = useCallback(async () => {
    try {
      const newState = await kappaLoopService.getState()
      setState(newState)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取状态失败')
    } finally {
      setLoading(false)
    }
  }, [])

  const handleStart = async () => {
    setLoading(true)
    const result = await kappaLoopService.start()
    if (!result.success) setError(result.message)
    await refreshState()
    setLoading(false)
  }

  const handleStop = async () => {
    setLoading(true)
    const result = await kappaLoopService.stop()
    if (!result.success) setError(result.message)
    await refreshState()
    setLoading(false)
  }

  const handlePause = async () => {
    setLoading(true)
    const result = await kappaLoopService.pause()
    if (!result.success) setError(result.message)
    await refreshState()
    setLoading(false)
  }

  const handleResume = async () => {
    setLoading(true)
    const result = await kappaLoopService.resume()
    if (!result.success) setError(result.message)
    await refreshState()
    setLoading(false)
  }

  const handleSelfRepair = async () => {
    setLoading(true)
    const result = await kappaLoopService.triggerSelfRepair()
    if (!result.success) setError(result.error || '自修复失败')
    await refreshState()
    setLoading(false)
  }

  useEffect(() => {
    refreshState()
    const interval = setInterval(refreshState, 3000)
    return () => clearInterval(interval)
  }, [refreshState])

  if (loading && !state) {
    return <div style={{ padding: '20px', textAlign: 'center', color: '#9ca3af' }}>加载中...</div>
  }

  if (!state) {
    return <div style={{ padding: '20px', textAlign: 'center', color: '#ef4444' }}>{error || '未知错误'}</div>
  }

  const statusText = kappaLoopService.getStatusText(state)
  const statusColor = kappaLoopService.getStatusColor(state)

  return (
    <div style={{
      padding: '20px', backgroundColor: '#1f2937', borderRadius: '12px',
      color: 'white', fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    }}>
      {/* 标题 */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '20px' }}>
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" strokeWidth="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z" />
          <path d="M2 17l10 5 10-5" />
          <path d="M2 12l10 5 10-5" />
        </svg>
        <h2 style={{ margin: 0, fontSize: '20px', fontWeight: 600 }}>
          Kappa Loop 卡帕斯循环
        </h2>
        <span style={{
          color: statusColor, fontWeight: 600, fontSize: '14px',
          padding: '2px 8px', borderRadius: '4px',
          backgroundColor: `${statusColor}20`,
        }}>
          {statusText}
        </span>
      </div>

      {/* 健康评分 + 统计信息 */}
      <div style={{ display: 'flex', gap: '20px', marginBottom: '20px', alignItems: 'center' }}>
        <HealthGauge score={state.health_score} />
        <div style={{ flex: 1, display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
          <StatCard label="改进次数" value={state.total_improvements} color="#10b981" />
          <StatCard label="回退次数" value={state.total_regressions} color="#ef4444" />
          <StatCard label="失败次数" value={state.total_failures} color="#6b7280" />
          <StatCard label="完成循环" value={state.total_cycles_completed} color="#3b82f6" />
        </div>
      </div>

      {/* 熔断器状态 */}
      {state.circuit_breaker_open && (
        <div style={{
          marginBottom: '16px', padding: '10px',
          backgroundColor: 'rgba(245, 158, 11, 0.1)', border: '1px solid #f59e0b',
          borderRadius: '8px', color: '#f59e0b', fontSize: '13px',
        }}>
          熔断器已打开 — 连续失败次数过多，系统正在冷却中...
        </div>
      )}

      {/* 控制按钮 */}
      <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', marginBottom: '20px' }}>
        {!state.is_running ? (
          <ControlButton onClick={handleStart} variant="success">▶️ 启动循环</ControlButton>
        ) : (
          <>
            <ControlButton onClick={state.is_paused ? handleResume : handlePause}
              variant={state.is_paused ? 'success' : 'warning'}>
              {state.is_paused ? '▶️ 恢复' : '⏸️ 暂停'}
            </ControlButton>
            <ControlButton onClick={handleStop} variant="danger">🛑 停止</ControlButton>
          </>
        )}
        <ControlButton onClick={handleSelfRepair} disabled={state.is_running} variant="primary">
          🔧 手动自修复
        </ControlButton>
      </div>

      {/* 实验日志 */}
      {state.experiments_log.length > 0 && (
        <div style={{ borderTop: '1px solid #374151', paddingTop: '16px' }}>
          <div style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', color: '#e5e7eb' }}>
            实验日志 ({state.experiments_log.length})
          </div>
          <div style={{ maxHeight: '300px', overflowY: 'auto' }}>
            {[...state.experiments_log].reverse().map((exp, i) => (
              <ExperimentItem key={i} exp={exp} />
            ))}
          </div>
        </div>
      )}

      {/* 当前迭代 */}
      {state.current_iteration > 0 && (
        <div style={{ marginTop: '12px', fontSize: '12px', color: '#6b7280' }}>
          当前迭代: #{state.current_iteration}
        </div>
      )}

      {/* 错误提示 */}
      {state.last_error && (
        <div style={{
          marginTop: '12px', padding: '10px',
          backgroundColor: 'rgba(239, 68, 68, 0.1)', border: '1px solid #ef4444',
          borderRadius: '8px', color: '#ef4444', fontSize: '13px',
        }}>
          ❌ {state.last_error}
        </div>
      )}

      {/* 操作错误 */}
      {error && (
        <div style={{
          marginTop: '12px', padding: '10px',
          backgroundColor: 'rgba(239, 68, 68, 0.1)', border: '1px solid #ef4444',
          borderRadius: '8px', color: '#ef4444', fontSize: '13px',
        }}>
          ❌ {error}
        </div>
      )}

      {/* 说明 */}
      <div style={{ marginTop: '16px', borderTop: '1px solid #374151', paddingTop: '12px' }}>
        <div style={{ fontSize: '12px', color: '#6b7280', lineHeight: '1.6' }}>
          <strong>卡帕斯循环</strong>：Test → Detect → Repair → Verify → Learn → Repeat<br/>
          基于 Hyperagent AutoResearch 引擎，AI 自动分析错误、生成修复、验证结果。
        </div>
      </div>
    </div>
  )
}

export default KappaLoopPanel
