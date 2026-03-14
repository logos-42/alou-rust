import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Ralph Loop 监控组件
 * 
 * 显示Ralph Loop的执行状态、迭代历史和AI决策
 */
const RalphLoopMonitor = ({ executionId, isVisible }) => {
  const [history, setHistory] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [autoRefresh, setAutoRefresh] = useState(true);

  // 获取Ralph Loop历史
  const fetchRalphLoopHistory = async () => {
    if (!executionId) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const result = await invoke('get_ralph_loop_history', { executionId });
      setHistory(result);
    } catch (err) {
      setError(err.message || '获取Ralph Loop历史失败');
    } finally {
      setLoading(false);
    }
  };

  // 自动刷新
  useEffect(() => {
    if (!isVisible || !executionId || !autoRefresh) return;

    fetchRalphLoopHistory();
    const interval = setInterval(fetchRalphLoopHistory, 2000); // 每2秒刷新

    return () => clearInterval(interval);
  }, [executionId, isVisible, autoRefresh]);

  // 清理历史记录
  const cleanupHistories = async () => {
    try {
      await invoke('cleanup_ralph_loop_histories', { maxAgeDays: 7 });
      alert('历史记录清理完成');
      fetchRalphLoopHistory();
    } catch (err) {
      alert('清理失败: ' + err.message);
    }
  };

  if (!isVisible) return null;

  return (
    <div className="ralph-loop-monitor">
      <div className="monitor-header">
        <h3>🔄 Ralph Loop 监控</h3>
        <div className="monitor-controls">
          <label>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            自动刷新
          </label>
          <button onClick={fetchRalphLoopHistory} disabled={loading}>
            🔄 刷新
          </button>
          <button onClick={cleanupHistories}>
            🗑️ 清理历史
          </button>
        </div>
      </div>

      {loading && <div className="loading">加载中...</div>}
      
      {error && (
        <div className="error">
          ❌ {error}
        </div>
      )}

      {history && (
        <div className="history-content">
          {/* 执行概览 */}
          <div className="execution-overview">
            <h4>📊 执行概览</h4>
            <div className="overview-grid">
              <div className="overview-item">
                <span className="label">工作流ID:</span>
                <span className="value">{history.workflow_id}</span>
              </div>
              <div className="overview-item">
                <span className="label">总迭代次数:</span>
                <span className="value">{history.total_iterations}</span>
              </div>
              <div className="overview-item">
                <span className="label">最终状态:</span>
                <span className={`value status ${history.final_status || 'unknown'}`}>
                  {history.final_status || '未知'}
                </span>
              </div>
              <div className="overview-item">
                <span className="label">总成本:</span>
                <span className="value">${history.total_cost?.toFixed(4) || '0.0000'}</span>
              </div>
              <div className="overview-item">
                <span className="label">总执行时间:</span>
                <span className="value">
                  {history.total_execution_time_ms 
                    ? `${(history.total_execution_time_ms / 1000).toFixed(2)}s`
                    : 'N/A'
                  }
                </span>
              </div>
              <div className="overview-item">
                <span className="label">开始时间:</span>
                <span className="value">
                  {new Date(history.started_at * 1000).toLocaleString()}
                </span>
              </div>
              {history.completed_at && (
                <div className="overview-item">
                  <span className="label">完成时间:</span>
                  <span className="value">
                    {new Date(history.completed_at * 1000).toLocaleString()}
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* 迭代历史 */}
          <div className="iterations-history">
            <h4>🔄 迭代历史</h4>
            <div className="iterations-list">
              {history.iterations?.slice().reverse().map((iteration, index) => (
                <div key={iteration.iteration} className="iteration-item">
                  <div className="iteration-header">
                    <span className="iteration-number">
                      #{iteration.iteration}
                    </span>
                    <span className={`iteration-status ${iteration.error ? 'error' : 'success'}`}>
                      {iteration.error ? '❌ 失败' : '✅ 成功'}
                    </span>
                    <span className="iteration-time">
                      {new Date(iteration.started_at * 1000).toLocaleTimeString()}
                    </span>
                  </div>
                  
                  <div className="iteration-details">
                    <div className="detail-item">
                      <span className="label">执行时间:</span>
                      <span className="value">
                        {iteration.execution_time_ms 
                          ? `${iteration.execution_time_ms}ms`
                          : 'N/A'
                        }
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="label">成本:</span>
                      <span className="value">${iteration.cost?.toFixed(4) || '0.0000'}</span>
                    </div>
                    <div className="detail-item">
                      <span className="label">重试次数:</span>
                      <span className="value">{iteration.retry_count || 0}</span>
                    </div>
                    
                    {iteration.error && (
                      <div className="error-detail">
                        <span className="label">错误:</span>
                        <span className="error-message">{iteration.error}</span>
                      </div>
                    )}
                    
                    {iteration.result && (
                      <div className="result-detail">
                        <span className="label">结果:</span>
                        <pre className="result-content">
                          {JSON.stringify(iteration.result, null, 2)}
                        </pre>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      <style jsx>{`
        .ralph-loop-monitor {
          background: #1a1a1a;
          border: 1px solid #333;
          border-radius: 8px;
          padding: 16px;
          margin: 16px 0;
          color: #fff;
        }

        .monitor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .monitor-header h3 {
          margin: 0;
          color: #4CAF50;
        }

        .monitor-controls {
          display: flex;
          gap: 12px;
          align-items: center;
        }

        .monitor-controls label {
          display: flex;
          align-items: center;
          gap: 4px;
          font-size: 12px;
        }

        .monitor-controls button {
          background: #333;
          border: 1px solid #555;
          color: #fff;
          padding: 4px 8px;
          border-radius: 4px;
          cursor: pointer;
          font-size: 12px;
        }

        .monitor-controls button:hover {
          background: #444;
        }

        .monitor-controls button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .loading, .error {
          text-align: center;
          padding: 20px;
        }

        .error {
          color: #f44336;
        }

        .execution-overview {
          margin-bottom: 24px;
        }

        .execution-overview h4 {
          margin: 0 0 12px 0;
          color: #2196F3;
        }

        .overview-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
        }

        .overview-item {
          display: flex;
          justify-content: space-between;
          padding: 8px 12px;
          background: #2a2a2a;
          border-radius: 4px;
        }

        .overview-item .label {
          color: #888;
          font-size: 12px;
        }

        .overview-item .value {
          font-weight: bold;
        }

        .overview-item .value.status.completed {
          color: #4CAF50;
        }

        .overview-item .value.status.failed {
          color: #f44336;
        }

        .overview-item .value.status.running {
          color: #FF9800;
        }

        .iterations-history h4 {
          margin: 0 0 12px 0;
          color: #9C27B0;
        }

        .iterations-list {
          max-height: 400px;
          overflow-y: auto;
        }

        .iteration-item {
          background: #2a2a2a;
          border: 1px solid #444;
          border-radius: 6px;
          margin-bottom: 12px;
          overflow: hidden;
        }

        .iteration-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 8px 12px;
          background: #333;
        }

        .iteration-number {
          font-weight: bold;
          color: #4CAF50;
        }

        .iteration-status.success {
          color: #4CAF50;
        }

        .iteration-status.error {
          color: #f44336;
        }

        .iteration-time {
          font-size: 12px;
          color: #888;
        }

        .iteration-details {
          padding: 12px;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          margin-bottom: 8px;
          font-size: 12px;
        }

        .detail-item .label {
          color: #888;
        }

        .error-detail {
          margin-top: 8px;
          padding-top: 8px;
          border-top: 1px solid #444;
        }

        .error-detail .label {
          color: #f44336;
          font-weight: bold;
        }

        .error-message {
          color: #f44336;
          font-size: 12px;
          word-break: break-all;
        }

        .result-detail {
          margin-top: 8px;
          padding-top: 8px;
          border-top: 1px solid #444;
        }

        .result-detail .label {
          color: #2196F3;
          font-weight: bold;
        }

        .result-content {
          background: #1a1a1a;
          border: 1px solid #444;
          border-radius: 4px;
          padding: 8px;
          margin-top: 4px;
          font-size: 10px;
          color: #ccc;
          max-height: 200px;
          overflow-y: auto;
          white-space: pre-wrap;
          word-break: break-all;
        }
      `}</style>
    </div>
  );
};

export default RalphLoopMonitor;
