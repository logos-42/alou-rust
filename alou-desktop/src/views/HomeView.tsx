import React from 'react'
import { useNavigate } from 'react-router-dom'
import AgentChat from '@/components/AgentChat'

/**
 * HomeView 组件
 * 应用程序的主页面，包含自主循环快捷入口和 AgentChat
 */
const HomeView: React.FC = () => {
  const navigate = useNavigate()

  const handleOpenAutonomousLoop = () => {
    navigate('/autonomous-loop')
  }

  const handleOpenKappaLoop = () => {
    navigate('/kappa-loop')
  }

  return (
    <div style={{ padding: '20px' }}>
      {/* 自主循环快捷入口 */}
      <div style={{
        marginBottom: '20px',
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '16px',
      }}>
        {/* 自主循环 */}
        <div style={{
          padding: '16px',
          backgroundColor: '#1f2937',
          borderRadius: '12px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" strokeWidth="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
            <div>
              <h3 style={{ margin: 0, color: 'white', fontSize: '15px' }}>🤖 自主循环</h3>
              <p style={{ margin: '2px 0 0', color: '#9ca3af', fontSize: '12px' }}>AI 自动执行任务</p>
            </div>
          </div>
          <button
            onClick={handleOpenAutonomousLoop}
            style={{
              padding: '8px 16px', backgroundColor: '#3b82f6', color: 'white',
              border: 'none', borderRadius: '6px', cursor: 'pointer',
              fontSize: '13px', fontWeight: 500,
            }}
          >
            打开 →
          </button>
        </div>

        {/* 卡帕斯循环 */}
        <div style={{
          padding: '16px',
          backgroundColor: '#1f2937',
          borderRadius: '12px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" strokeWidth="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
            <div>
              <h3 style={{ margin: 0, color: 'white', fontSize: '15px' }}>🔄 卡帕斯循环</h3>
              <p style={{ margin: '2px 0 0', color: '#9ca3af', fontSize: '12px' }}>自修复循环测试</p>
            </div>
          </div>
          <button
            onClick={handleOpenKappaLoop}
            style={{
              padding: '8px 16px', backgroundColor: '#8b5cf6', color: 'white',
              border: 'none', borderRadius: '6px', cursor: 'pointer',
              fontSize: '13px', fontWeight: 500,
            }}
          >
            打开 →
          </button>
        </div>
      </div>

      {/* Agent Chat */}
      <AgentChat />
    </div>
  )
}

export default HomeView
