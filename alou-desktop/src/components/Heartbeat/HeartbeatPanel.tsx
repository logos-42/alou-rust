/**
 * 心跳管理面板组件
 * Heartbeat Management Panel Component
 * 
 * 提供心跳服务的可视化界面，包括：
 * - 状态显示
 * - 控制按钮
 * - 配置对话框
 * - 健康状态面板
 */

import React, { useState, useEffect, useCallback } from 'react';
import { heartbeatService } from '@/services/heartbeatService';
import type {
  HeartbeatConfig,
  HeartbeatState,
  HealthStatus,
  CheckResult,
  Issue,
} from '@shared/types/heartbeat';
import './HeartbeatPanel.css';

/**
 * 格式化时间戳为本地时间字符串
 * Format timestamp to local time string
 */
const formatTimestamp = (timestamp: number | null): string => {
  if (!timestamp) return '—';
  return new Date(timestamp).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
};

/**
 * 获取健康状态的样式类
 * Get health status style class
 */
const getHealthStatusClass = (overall: string): string => {
  switch (overall) {
    case 'Good':
      return 'health-status-good';
    case 'Warning':
      return 'health-status-warning';
    case 'Critical':
      return 'health-status-critical';
    default:
      return '';
  }
};

/**
 * 获取问题严重程度的样式类
 * Get issue severity style class
 */
const getIssueSeverityClass = (severity: string): string => {
  switch (severity) {
    case 'low':
      return 'severity-low';
    case 'medium':
      return 'severity-medium';
    case 'high':
      return 'severity-high';
    case 'critical':
      return 'severity-critical';
    default:
      return '';
  }
};

interface HeartbeatPanelProps {
  isDarkMode?: boolean;
  onClose?: () => void;
}

const HeartbeatPanel: React.FC<HeartbeatPanelProps> = ({
  isDarkMode = false,
  onClose,
}) => {
  // 状态管理
  const [heartbeatState, setHeartbeatState] = useState<HeartbeatState | null>(null);
  const [heartbeatConfig, setHeartbeatConfig] = useState<HeartbeatConfig | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showConfigDialog, setShowConfigDialog] = useState(false);
  const [showHealthPanel, setShowHealthPanel] = useState(false);
  const [configForm, setConfigForm] = useState<Partial<HeartbeatConfig>>({});
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  /**
   * 加载心跳状态
   * Load heartbeat state
   */
  const loadState = useCallback(async () => {
    try {
      const response = await heartbeatService.getState();
      if (response.success && response.data) {
        setHeartbeatState(response.data);
      }
    } catch (err: any) {
      console.error('加载状态失败:', err);
    }
  }, []);

  /**
   * 加载心跳配置
   * Load heartbeat configuration
   */
  const loadConfig = useCallback(async () => {
    try {
      const response = await heartbeatService.getConfig();
      if (response.success && response.data) {
        setHeartbeatConfig(response.data);
      }
    } catch (err: any) {
      console.error('加载配置失败:', err);
    }
  }, []);

  /**
   * 加载健康状态
   * Load health status
   */
  const loadHealthStatus = useCallback(async () => {
    try {
      const response = await heartbeatService.getHealthStatus();
      if (response.success && response.data) {
        setHealthStatus(response.data);
      }
    } catch (err: any) {
      console.error('加载健康状态失败:', err);
    }
  }, []);

  /**
   * 初始化加载数据
   * Initialize and load data
   */
  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      await Promise.all([loadState(), loadConfig()]);
      setLoading(false);
    };

    loadData();

    // 定时刷新状态（每 5 秒）
    const intervalId = setInterval(loadState, 5000);
    return () => clearInterval(intervalId);
  }, [loadState, loadConfig]);

  /**
   * 显示消息提示
   * Show message toast
   */
  const showMessage = useCallback((type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 3000);
  }, []);

  /**
   * 处理启动/停止心跳
   * Handle start/stop heartbeat
   */
  const handleToggleHeartbeat = async () => {
    setLoading(true);
    setError(null);

    try {
      const isRunning = heartbeatState?.is_running;
      const response = isRunning
        ? await heartbeatService.stopHeartbeat()
        : await heartbeatService.startHeartbeat();

      if (response.success) {
        await loadState();
        showMessage('success', isRunning ? '心跳已停止' : '心跳已启动');
      } else {
        setError(response.error || '操作失败');
        showMessage('error', response.error || '操作失败');
      }
    } catch (err: any) {
      setError(err.message || '操作失败');
      showMessage('error', err.message || '操作失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理立即触发心跳
   * Handle trigger heartbeat now
   */
  const handleTriggerNow = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await heartbeatService.triggerHeartbeatNow();
      if (response.success) {
        await loadState();
        showMessage('success', '心跳已触发');
      } else {
        setError(response.error || '触发失败');
        showMessage('error', response.error || '触发失败');
      }
    } catch (err: any) {
      setError(err.message || '触发失败');
      showMessage('error', err.message || '触发失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理打开配置对话框
   * Handle open config dialog
   */
  const handleOpenConfig = () => {
    if (heartbeatConfig) {
      setConfigForm({ ...heartbeatConfig });
      setShowConfigDialog(true);
    }
  };

  /**
   * 处理保存配置
   * Handle save configuration
   */
  const handleSaveConfig = async () => {
    setLoading(true);
    setError(null);

    try {
      // 验证间隔范围
      if (configForm.interval_minutes) {
        if (configForm.interval_minutes < 30 || configForm.interval_minutes > 120) {
          throw new Error('心跳间隔必须在 30-120 分钟之间');
        }
      }

      const response = await heartbeatService.updateConfig(configForm);
      if (response.success) {
        await loadConfig();
        setShowConfigDialog(false);
        showMessage('success', '配置已更新');
      } else {
        setError(response.error || '更新失败');
        showMessage('error', response.error || '更新失败');
      }
    } catch (err: any) {
      setError(err.message || '更新失败');
      showMessage('error', err.message || '更新失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理打开健康面板
   * Handle open health panel
   */
  const handleOpenHealthPanel = async () => {
    setLoading(true);
    try {
      await loadHealthStatus();
      setShowHealthPanel(true);
    } catch (err: any) {
      showMessage('error', '加载健康状态失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理运行健康检查
   * Handle run health check
   */
  const handleRunHealthCheck = async () => {
    setLoading(true);
    try {
      const response = await heartbeatService.runHealthCheck();
      if (response.success && response.data) {
        setHealthStatus(response.data);
        showMessage('success', '健康检查完成');
      } else {
        showMessage('error', response.error || '健康检查失败');
      }
    } catch (err: any) {
      showMessage('error', '健康检查失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理执行自愈
   * Handle execute self-heal
   */
  const handleSelfHeal = async () => {
    setLoading(true);
    try {
      const response = await heartbeatService.executeSelfHeal();
      if (response.success) {
        await loadHealthStatus();
        showMessage('success', response.message || '自愈完成');
      } else {
        showMessage('error', response.error || '自愈失败');
      }
    } catch (err: any) {
      showMessage('error', '自愈失败');
    } finally {
      setLoading(false);
    }
  };

  /**
   * 处理配置表单变更
   * Handle config form change
   */
  const handleConfigChange = (
    field: keyof HeartbeatConfig,
    value: HeartbeatConfig[keyof HeartbeatConfig]
  ) => {
    setConfigForm((prev) => ({ ...prev, [field]: value }));
  };

  const panelClass = `heartbeat-panel ${isDarkMode ? 'dark' : ''}`;

  return (
    <div className={panelClass}>
      {/* 消息提示 */}
      {message && (
        <div className={`heartbeat-message ${message.type}`}>
          {message.text}
        </div>
      )}

      {/* 状态显示区域 */}
      <div className="heartbeat-section">
        <h3 className="heartbeat-section-title">心跳状态</h3>

        <div className="heartbeat-status-grid">
          {/* 运行状态指示灯 */}
          <div className="heartbeat-status-item">
            <span className="heartbeat-status-label">运行状态</span>
            <div className="heartbeat-status-value">
              <span
                className={`heartbeat-status-indicator ${
                  heartbeatState?.is_running ? 'running' : 'stopped'
                }`}
              />
              <span className="heartbeat-status-text">
                {heartbeatState?.is_running ? '运行中' : '已停止'}
              </span>
            </div>
          </div>

          {/* 上次心跳时间 */}
          <div className="heartbeat-status-item">
            <span className="heartbeat-status-label">上次心跳</span>
            <span className="heartbeat-status-value">
              {formatTimestamp(heartbeatState?.last_heartbeat || null)}
            </span>
          </div>

          {/* 下次心跳时间 */}
          <div className="heartbeat-status-item">
            <span className="heartbeat-status-label">下次心跳</span>
            <span className="heartbeat-status-value">
              {formatTimestamp(heartbeatState?.next_heartbeat || null)}
            </span>
          </div>

          {/* 总心跳次数 */}
          <div className="heartbeat-status-item">
            <span className="heartbeat-status-label">总心跳次数</span>
            <span className="heartbeat-status-value">
              {heartbeatState?.total_beats || 0}
            </span>
          </div>
        </div>
      </div>

      {/* 控制按钮 */}
      <div className="heartbeat-section">
        <div className="heartbeat-controls">
          <button
            className={`heartbeat-btn heartbeat-btn-primary ${
              heartbeatState?.is_running ? 'heartbeat-btn-stop' : ''
            }`}
            onClick={handleToggleHeartbeat}
            disabled={loading}
          >
            {heartbeatState?.is_running ? '停止心跳' : '启动心跳'}
          </button>

          <button
            className="heartbeat-btn heartbeat-btn-secondary"
            onClick={handleTriggerNow}
            disabled={loading || !heartbeatState?.is_running}
          >
            立即触发
          </button>

          <button
            className="heartbeat-btn heartbeat-btn-secondary"
            onClick={handleOpenConfig}
            disabled={loading}
          >
            配置
          </button>

          <button
            className="heartbeat-btn heartbeat-btn-secondary"
            onClick={handleOpenHealthPanel}
            disabled={loading}
          >
            健康检查
          </button>
        </div>
      </div>

      {/* 错误显示 */}
      {error && <div className="heartbeat-error">{error}</div>}

      {/* 配置对话框 */}
      {showConfigDialog && (
        <div className="heartbeat-dialog-overlay" onClick={() => setShowConfigDialog(false)}>
          <div className="heartbeat-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="heartbeat-dialog-header">
              <h3>心跳配置</h3>
              <button
                className="heartbeat-dialog-close"
                onClick={() => setShowConfigDialog(false)}
              >
                ×
              </button>
            </div>

            <div className="heartbeat-dialog-content">
              {/* 启用/禁用开关 */}
              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">启用心跳</label>
                <label className="heartbeat-switch">
                  <input
                    type="checkbox"
                    checked={configForm.enabled || false}
                    onChange={(e) =>
                      handleConfigChange('enabled', e.target.checked)
                    }
                  />
                  <span className="heartbeat-switch-slider" />
                </label>
              </div>

              {/* 心跳间隔 */}
              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">
                  心跳间隔（分钟）
                </label>
                <input
                  type="number"
                  className="heartbeat-form-input"
                  min={1}
                  max={120}
                  value={configForm.interval_minutes || 10}
                  onChange={(e) =>
                    handleConfigChange(
                      'interval_minutes',
                      parseInt(e.target.value, 10) || 10
                    )
                  }
                />
                <span className="heartbeat-form-hint">范围：1-120 分钟（推荐 10-30 分钟）</span>
              </div>

              {/* 🔥 持续任务模式说明 */}
              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">持续任务模式</label>
                <div className="heartbeat-form-hint" style={{ marginTop: '8px' }}>
                  <p style={{ margin: '0 0 8px 0' }}>
                    在 HEARTBEAT.md 文件中添加 <code># CONTINUOUS</code> 或 <code>[持续执行]</code> 标记，
                    即可启用持续任务模式。
                  </p>
                  <p style={{ margin: 0 }}>
                    🔹 普通模式：任务执行后清空文件<br/>
                    🔹 持续模式：保留文件内容，每次心跳继续执行
                  </p>
                </div>
              </div>

              {/* 模型选择 */}
              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">便宜模型</label>
                <input
                  type="text"
                  className="heartbeat-form-input"
                  value={configForm.cheap_model || ''}
                  onChange={(e) =>
                    handleConfigChange('cheap_model', e.target.value)
                  }
                  placeholder="用于常规心跳（如：deepseek-chat）"
                />
              </div>

              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">昂贵模型</label>
                <input
                  type="text"
                  className="heartbeat-form-input"
                  value={configForm.expensive_model || ''}
                  onChange={(e) =>
                    handleConfigChange('expensive_model', e.target.value)
                  }
                  placeholder="用于重要操作（如：claude-sonnet-4）"
                />
              </div>

              {/* 文件路径显示 */}
              <div className="heartbeat-form-group">
                <label className="heartbeat-form-label">心跳文件路径</label>
                <div className="heartbeat-form-path">
                  {configForm.heartbeat_file_path || '未设置'}
                </div>
                <button
                  type="button"
                  className="heartbeat-btn heartbeat-btn-secondary"
                  style={{ marginTop: '8px' }}
                  onClick={() => {
                    navigator.clipboard.writeText(configForm.heartbeat_file_path || '');
                    showMessage('success', '路径已复制到剪贴板');
                  }}
                >
                  复制路径
                </button>
              </div>
            </div>

            <div className="heartbeat-dialog-footer">
              <button
                className="heartbeat-btn heartbeat-btn-secondary"
                onClick={() => setShowConfigDialog(false)}
              >
                取消
              </button>
              <button
                className="heartbeat-btn heartbeat-btn-primary"
                onClick={handleSaveConfig}
                disabled={loading}
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 健康状态面板 */}
      {showHealthPanel && healthStatus && (
        <div className="heartbeat-dialog-overlay" onClick={() => setShowHealthPanel(false)}>
          <div className="heartbeat-dialog heartbeat-dialog-health" onClick={(e) => e.stopPropagation()}>
            <div className="heartbeat-dialog-header">
              <h3>健康状态</h3>
              <button
                className="heartbeat-dialog-close"
                onClick={() => setShowHealthPanel(false)}
              >
                ×
              </button>
            </div>

            <div className="heartbeat-dialog-content">
              {/* 整体健康状态 */}
              <div className={`heartbeat-health-overall ${getHealthStatusClass(healthStatus.overall)}`}>
                <span className="heartbeat-health-label">整体状态</span>
                <span className="heartbeat-health-value">{healthStatus.overall}</span>
              </div>

              {/* 检查结果列表 */}
              <div className="heartbeat-health-section">
                <h4>检查项</h4>
                <div className="heartbeat-checks-list">
                  {healthStatus.checks.map((check: CheckResult, index: number) => (
                    <div
                      key={index}
                      className={`heartbeat-check-item ${
                        check.passed ? 'check-passed' : 'check-failed'
                      }`}
                    >
                      <span className="heartbeat-check-icon">
                        {check.passed ? '✓' : '✗'}
                      </span>
                      <div className="heartbeat-check-content">
                        <span className="heartbeat-check-name">{check.name}</span>
                        {check.details && (
                          <span className="heartbeat-check-details">{check.details}</span>
                        )}
                        {check.recommendation && (
                          <span className="heartbeat-check-recommendation">
                            建议：{check.recommendation}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* 问题列表 */}
              {healthStatus.issues.length > 0 && (
                <div className="heartbeat-health-section">
                  <h4>发现的问题</h4>
                  <div className="heartbeat-issues-list">
                    {healthStatus.issues.map((issue: Issue, index: number) => (
                      <div
                        key={index}
                        className={`heartbeat-issue-item ${getIssueSeverityClass(
                          issue.severity
                        )}`}
                      >
                        <span className="heartbeat-issue-code">{issue.code}</span>
                        <div className="heartbeat-issue-content">
                          <span className="heartbeat-issue-description">
                            {issue.description}
                          </span>
                          {issue.fix && (
                            <span className="heartbeat-issue-fix">修复：{issue.fix}</span>
                          )}
                        </div>
                        <span className="heartbeat-issue-severity">
                          {issue.severity}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="heartbeat-dialog-footer">
              <button
                className="heartbeat-btn heartbeat-btn-secondary"
                onClick={handleRunHealthCheck}
                disabled={loading}
              >
                重新检查
              </button>
              <button
                className="heartbeat-btn heartbeat-btn-primary"
                onClick={handleSelfHeal}
                disabled={loading || healthStatus.issues.length === 0}
              >
                执行自愈
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default HeartbeatPanel;
