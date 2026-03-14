/**
 * StorageManager - 存储管理组件
 * 提供本地存储监控、清理和优化功能
 */

import React, { useState, useEffect } from 'react'
import { 
  getStorageUsage, 
  cleanupStorage, 
  optimizeStorage, 
  getStorageRecommendations,
  logStorageStatus,
  formatBytes 
} from '@/utils/storageMonitor'
import './StorageManager.css'

const StorageManager = ({ visible = false, onClose }) => {
  const [usage, setUsage] = useState(null)
  const [recommendations, setRecommendations] = useState([])
  const [isCleaning, setIsCleaning] = useState(false)
  const [cleanResult, setCleanResult] = useState(null)

  // 刷新存储状态
  const refreshUsage = () => {
    const usageData = getStorageUsage()
    setUsage(usageData)
    
    const recs = getStorageRecommendations()
    setRecommendations(recs)
  }

  // 组件挂载时获取存储状态
  useEffect(() => {
    if (visible) {
      refreshUsage()
      logStorageStatus()
    }
  }, [visible])

  // 执行清理
  const handleCleanup = async (options = {}) => {
    setIsCleaning(true)
    setCleanResult(null)

    try {
      const result = cleanupStorage(options)
      setCleanResult(result)
      
      // 清理后刷新状态
      setTimeout(() => {
        refreshUsage()
      }, 500)
      
      console.log('[StorageManager] 清理完成:', result)
    } catch (error) {
      console.error('[StorageManager] 清理失败:', error)
      setCleanResult({
        cleaned: 0,
        errors: [error.message],
        details: []
      })
    } finally {
      setIsCleaning(false)
    }
  }

  // 执行优化
  const handleOptimize = async () => {
    setIsCleaning(true)
    setCleanResult(null)

    try {
      const result = optimizeStorage()
      setCleanResult({
        cleaned: result ? 1 : 0,
        errors: result ? [] : ['优化失败'],
        details: []
      })
      
      // 优化后刷新状态
      setTimeout(() => {
        refreshUsage()
      }, 500)
      
      console.log('[StorageManager] 优化完成:', result)
    } catch (error) {
      console.error('[StorageManager] 优化失败:', error)
      setCleanResult({
        cleaned: 0,
        errors: [error.message],
        details: []
      })
    } finally {
      setIsCleaning(false)
    }
  }

  if (!visible) {
    return null
  }

  return (
    <div className="storage-manager">
      <div className="manager-header">
        <h2>存储管理</h2>
        <button className="close-btn" onClick={onClose}>×</button>
      </div>

      <div className="manager-content">
        {/* 存储使用情况 */}
        {usage && (
          <div className="usage-section">
            <h3>存储使用情况</h3>
            <div className="usage-overview">
              <div className="usage-item">
                <span className="label">总大小:</span>
                <span className="value">{usage.formattedSize}</span>
              </div>
              <div className="usage-item">
                <span className="label">使用率:</span>
                <span className={`value ${usage.isOverLimit ? 'over' : usage.isNearLimit ? 'near' : 'normal'}`}>
                  {usage.usagePercentage}%
                </span>
              </div>
              <div className="usage-item">
                <span className="label">项目数量:</span>
                <span className="value">{usage.itemCount}</span>
              </div>
            </div>

            {/* 进度条 */}
            <div className="progress-bar">
              <div 
                className={`progress-fill ${usage.isOverLimit ? 'over' : usage.isNearLimit ? 'near' : 'normal'}`}
                style={{ width: `${Math.min(usage.usagePercentage, 100)}%` }}
              />
            </div>

            {/* 数据分布 */}
            {usage.details && (
              <div className="usage-details">
                <h4>数据分布</h4>
                <div className="detail-grid">
                  {usage.details.groupChats && (
                    <div className="detail-item">
                      <span className="detail-label">群聊数据:</span>
                      <span className="detail-value">{formatBytes(usage.details.groupChats)}</span>
                    </div>
                  )}
                  {usage.details.diapGroups && (
                    <div className="detail-item">
                      <span className="detail-label">DIAP群聊:</span>
                      <span className="detail-value">{formatBytes(usage.details.diapGroups)}</span>
                    </div>
                  )}
                  {usage.details.diapData && (
                    <div className="detail-item">
                      <span className="detail-label">DIAP其他:</span>
                      <span className="detail-value">{formatBytes(usage.details.diapData)}</span>
                    </div>
                  )}
                  {usage.details.other && (
                    <div className="detail-item">
                      <span className="detail-label">其他数据:</span>
                      <span className="detail-value">{formatBytes(usage.details.other)}</span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}

        {/* 建议 */}
        {recommendations.length > 0 && (
          <div className="recommendations-section">
            <h3>优化建议</h3>
            <div className="recommendations-list">
              {recommendations.map((rec, index) => (
                <div key={index} className={`recommendation-item ${rec.level}`}>
                  <div className="recommendation-header">
                    <span className="recommendation-icon">
                      {rec.level === 'critical' ? '🚨' : rec.level === 'warning' ? '⚠️' : 'ℹ️'}
                    </span>
                    <span className="recommendation-title">{rec.title}</span>
                  </div>
                  <div className="recommendation-message">{rec.message}</div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 操作按钮 */}
        <div className="actions-section">
          <h3>存储操作</h3>
          <div className="action-buttons">
            <button 
              className="action-btn optimize"
              onClick={handleOptimize}
              disabled={isCleaning}
            >
              {isCleaning ? '优化中...' : '优化存储'}
            </button>
            
            <button 
              className="action-btn cleanup-light"
              onClick={() => handleCleanup({ keepRecent: 3, keepRecentDiap: 2 })}
              disabled={isCleaning}
            >
              {isCleaning ? '清理中...' : '清理旧数据'}
            </button>
            
            <button 
              className="action-btn cleanup-deep"
              onClick={() => handleCleanup({ keepRecent: 1, keepRecentDiap: 1, clearOlderThan: 3 * 24 * 60 * 60 * 1000 })}
              disabled={isCleaning}
            >
              {isCleaning ? '深度清理...' : '深度清理'}
            </button>
          </div>
        </div>

        {/* 清理结果 */}
        {cleanResult && (
          <div className="result-section">
            <h3>操作结果</h3>
            <div className={`result ${cleanResult.errors.length > 0 ? 'error' : 'success'}`}>
              {cleanResult.errors.length > 0 ? (
                <div>
                  <div className="result-error">清理过程中出现错误:</div>
                  <ul className="error-list">
                    {cleanResult.errors.map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                  </ul>
                </div>
              ) : (
                <div>
                  <div className="result-success">
                    ✅ 清理完成！
                    {cleanResult.cleaned > 0 && (
                      <span> 清理了 {cleanResult.cleaned} 个项目</span>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* 刷新按钮 */}
        <div className="refresh-section">
          <button 
            className="refresh-btn"
            onClick={refreshUsage}
            disabled={isCleaning}
          >
            🔄 刷新状态
          </button>
        </div>
      </div>
    </div>
  )
}

export default StorageManager
