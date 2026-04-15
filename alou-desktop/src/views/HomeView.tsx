import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import './HomeView.css'

/**
 * HomeView - Alou 首页
 * 
 * 前面：极简介绍（乔布斯风格）
 * 后面：功能入口（保留所有功能）
 */
const HomeView: React.FC = () => {
  const navigate = useNavigate()
  const [showFeatures, setShowFeatures] = useState(false)

  const handleCreateCollab = () => {
    navigate('/collab/new')
  }

  return (
    <div className="home-view">
      {/* 前面：极简介绍 */}
      <div className="home-hero">
        <h1 className="home-title">
          不再和一个 AI 说话
          <br />
          <span className="home-title-accent">让一群 AI 为你工作</span>
        </h1>
        <p className="home-subtitle">
          创建 AI 协作团队，多个 AI 自动分工、一起工作、交付结果
        </p>
        
        <div className="home-actions">
          <button 
            className="home-cta primary"
            onClick={handleCreateCollab}
          >
            ✨ 创建协作
          </button>
          
          <button 
            className="home-cta secondary"
            onClick={() => setShowFeatures(!showFeatures)}
          >
            {showFeatures ? '收起' : '了解更多'} ↓
          </button>
        </div>
      </div>

      {/* 后面：功能入口（保留所有功能） */}
      {showFeatures && (
        <div className="home-features">
          <h2>功能</h2>
          
          <div className="features-grid">
            <div className="feature-card" onClick={() => navigate('/collab/new')}>
              <div className="feature-icon">🤝</div>
              <h3>AI 协作</h3>
              <p>创建 AI 团队，多个 AI 一起完成任务</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/chat')}>
              <div className="feature-icon">💬</div>
              <h3>智能体对话</h3>
              <p>和单个 AI 进行深度对话</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/group-chat')}>
              <div className="feature-icon">👥</div>
              <h3>群聊</h3>
              <p>邀请多个 AI 一起讨论</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/autonomous-loop')}>
              <div className="feature-icon">🔄</div>
              <h3>自主循环</h3>
              <p>AI 自动执行任务，持续学习进化</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/wallet')}>
              <div className="feature-icon">💰</div>
              <h3>Web3 钱包</h3>
              <p>管理加密资产，支持多链</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/ipfs')}>
              <div className="feature-icon">🌐</div>
              <h3>IPFS 存储</h3>
              <p>去中心化存储，数据永久保存</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/skills')}>
              <div className="feature-icon">🛠️</div>
              <h3>技能市场</h3>
              <p>发现和安装 AI 技能</p>
            </div>

            <div className="feature-card" onClick={() => navigate('/settings')}>
              <div className="feature-icon">⚙️</div>
              <h3>设置</h3>
              <p>配置 API、主题、语言等</p>
            </div>
          </div>
        </div>
      )}

      {/* 底部 */}
      <div className="home-footer">
        <p>Alou v1.0.0 - AI 协作团队平台</p>
      </div>
    </div>
  )
}

export default HomeView
