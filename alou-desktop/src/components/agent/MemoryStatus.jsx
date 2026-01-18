/**
 * MemoryStatus - 内存存储状态显示组件
 * 显示内存使用情况和提供管理功能
 */

import React from 'react'
import { useMemoryStore } from '@/stores/memoryStore'
import './MemoryStatus.css'

const MemoryStatus = ({ visible = false, onClose }) => {
  const { stats, storage, clearMemory, cleanup } = useMemoryStore()

  if (!visible) {
    return null
  }

  const formatBytes = (bytes) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const handleCleanup = (level) => {
    switch (level) {
      case 'light':
        cleanup({
          maxAge: 30 * 60 * 1000, // 30分钟
          maxItems: 300
        })
        break
      case 'deep':
        cleanup({
          maxAge: 10 * 60 * 1000, // 10分钟
          maxItems: 100
        })
        break
      case 'clear':
        clearMemory()
        break
      default:
        cleanup()
    }
  }

  return (
    <div className="memory-status">
      <div className="status-header">
        <h2>内存存储状态</h2>
        <button className="close-btn" onClick={onClose}>×</button>
      </div>

      <div className="status-content">
        {/* 存储统计 */}
        <div className="stats-section">
          <h3>存储统计</h3>
          <div className="stats-grid">
            <div className="stat-item">
              <span className="stat-label">存储项目:</span>
              <span className="stat-value">{stats.size}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">最大容量:</span>
              <span className="stat-value">{stats.maxSize}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">使用率:</span>
              <span className={`stat-value ${stats.usage > 80 ? 'high' : stats.usage > 60 ? 'medium' : 'normal'}`}>
                {stats.usage}%
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">内存占用:</span>
              <span className="stat-value">{stats.memoryUsage?.formatted || '0 KB'}</span>
            </div>
          </div>

          {/* 进度条 */}
          <div className="progress-container">
            <div className="progress-bar">
              <div 
                className={`progress-fill ${stats.usage > 80 ? 'high' : stats.usage > 60 ? 'medium' : 'normal'}`}
                style={{ width: `${Math.min(stats.usage, 100)}%` }}
              />
            </div>
            <div className="progress-text">{stats.usage}%</div>
          </div>
        </div>

        {/* 存储键列表 */}
        <div className="keys-section">
          <h3>存储键列表</h3>
          <div className="keys-container">
            {stats.keys.length === 0 ? (
              <div className="empty-keys">
                <p>暂无存储数据</p>
              </div>
            ) : (
              <div className="keys-list">
                {stats.keys.map(key => (
                  <div key={key} className="key-item">
                    <div className="key-name" title={key}>
                      {key.length > 30 ? `${key.substring(0, 30)}...` : key}
                    </div>
                    <div className="key-type">
                      {key.startsWith('diap_group_chat_') && (
                        <span className="key-type-badge diap">DIAP群聊</span>
                      )}
                      {key.startsWith('cluster_actions_') && (
                        <span className="key-type-badge cluster">集群行动</span>
                      )}
                      {key.startsWith('diap_') && (
                        <span className="key-type-badge diap-data">DIAP数据</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="actions-section">
          <h3>内存管理</h3>
          <div className="action-buttons">
            <button 
              className="action-btn cleanup-light"
              onClick={() => handleCleanup('light')}
            >
              🧹 轻度清理
            </button>
            
            <button 
              className="action-btn cleanup-deep"
              onClick={() => handleCleanup('deep')}
            >
              🗑️ 深度清理
            </button>
            
            <button 
              className="action-btn clear-all"
              onClick={() => handleCleanup('clear')}
            >
              💥 清空所有
            </button>
          </div>
          
          <div className="action-description">
            <p><strong>轻度清理:</strong> 清理30分钟前的数据，保留300个项目</p>
            <p><strong>深度清理:</strong> 清理10分钟前的数据，保留100个项目</p>
            <p><strong>清空所有:</strong> 删除所有存储数据</p>
          </div>
        </div>

        {/* 优势说明 */}
        <div className="benefits-section">
          <h3>内存存储优势</h3>
          <div className="benefits-list">
            <div className="benefit-item">
              <span className="benefit-icon">⚡</span>
              <div className="benefit-content">
                <strong>极速访问</strong>
                <p>比localStorage快10-100倍</p>
              </div>
            </div>
            
            <div className="benefit-item">
              <span className="benefit-icon">♾️</span>
              <div className="benefit-content">
                <strong>无空间限制</strong>
                <p>不受浏览器存储配额限制</p>
              </div>
            </div>
            
            <div className="benefit-item">
              <span className="benefit-icon">🔄</span>
              <div className="benefit-content">
                <strong>自动清理</strong>
                <p>智能管理内存使用</p>
              </div>
            </div>
            
            <div className="benefit-item">
              <span className="benefit-icon">🔒</span>
              <div className="benefit-content">
                <strong>隐私保护</strong>
                <p>数据不会持久化到磁盘</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MemoryStatus
