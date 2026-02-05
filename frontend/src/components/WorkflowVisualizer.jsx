import React, { useState, useEffect, useCallback } from 'react';
import { useI18n } from '@/hooks/useI18n';
import { useWorkflow } from '@/hooks/useWorkflow';
import './WorkflowVisualizer.css';

// Mock hooks for stream functionality - these should be replaced with actual implementations
const useAgentStream = (sessionId, options) => {
  // This is a mock implementation - replace with actual stream hook
  return { status: 'active' }; // or 'polling', 'error', 'disconnected'
};

/**
 * WorkflowVisualizer - 桌面版工作流可视化和控制组件
 * 使用useWorkflow hook管理工作流状态和操作
 */
const WorkflowVisualizer = ({
  sessionId,
  apiKey,
  agentInfo = {},
  onWorkflowEvent,
  onSendMessage,
  className = ''
}) => {
  const { t } = useI18n();

  const {
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    isLoading,
    executingWorkflowId,
    executionProgress,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
  } = useWorkflow({
    sessionId,
    apiKey,
    agentInfo,
    onWorkflowMessage: onSendMessage,
  });

  // 使用流式事件监听
  const { status: streamStatus } = useAgentStream(sessionId, {
    enabled: !!sessionId,
    onEvent: () => {} // Mock event handler
  });

  // 获取步骤状态样式
  const getStepStatusClass = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return 'step-completed';
      case 'running': return 'step-running';
      case 'failed': return 'step-failed';
      case 'pending': return 'step-pending';
      case 'paused': return 'step-paused';
      default: return 'step-unknown';
    }
  };

  // 获取步骤状态图标
  const getStepStatusIcon = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return '✅';
      case 'running': return '⏳';
      case 'failed': return '❌';
      case 'pending': return '⏸️';
      case 'paused': return '⏸️';
      default: return '❓';
    }
  };

  // 初始化加载
  useEffect(() => {
    if (sessionId) {
      loadWorkflows();
    }
  }, [sessionId, loadWorkflows]);

  return (
    <div className={`workflow-visualizer ${className}`}>
      <div className="workflow-header">
        <div className="workflow-header-top">
          <h3>{t('common.workflow')}</h3>
          <div className="workflow-stream-status">
            <span className={`stream-indicator ${streamStatus}`}></span>
            <span className="stream-label">
              {streamStatus === 'active' ? t('common.workflow.realtimeUpdate') :
               streamStatus === 'polling' ? t('common.workflow.connecting') :
               streamStatus === 'error' ? t('common.workflow.connectionError') : t('common.workflow.notConnected')}
            </span>
          </div>
        </div>
        <div className="workflow-actions">
          <button
            onClick={createSampleWorkflow}
            disabled={isLoading || !sessionId}
            className="btn-create"
          >
            {isLoading ? t('common.loading') : t('common.workflow.createSample')}
          </button>
          <button
            onClick={loadWorkflows}
            disabled={isLoading || !sessionId}
            className="btn-refresh"
          >
            {t('common.refresh')}
          </button>
        </div>
      </div>

      {isLoading && (
        <div className="workflow-loading">
          <div className="spinner"></div>
          加载中...
        </div>
      )}

      <div className="workflow-list">
        {workflows.length === 0 && !isLoading ? (
          <div className="workflow-empty">
            <p>{t('common.workflow.noWorkflows')}</p>
            <p>{t('common.workflow.createSample')}</p>
          </div>
        ) : (
          workflows.map(workflow => (
            <div key={workflow.id} className="workflow-card">
              <div className="workflow-card-header">
                <div className="workflow-info">
                  <h4>{workflow.name}</h4>
                  <p>{workflow.description}</p>
                  <div className="workflow-meta">
                    <span className="step-count">
                      {workflow.step_count || workflow.steps?.length || 0} {t('common.workflow.steps')}
                    </span>
                    <span className={`workflow-status status-${workflow.status?.toLowerCase()}`}>
                      {workflow.status === 'pending' ? t('common.workflow.status.pending') :
                       workflow.status === 'running' ? t('common.workflow.status.running') :
                       workflow.status === 'completed' ? t('common.workflow.status.completed') :
                       workflow.status === 'failed' ? t('common.workflow.status.failed') :
                       workflow.status === 'paused' ? t('common.workflow.status.paused') :
                       workflow.status || 'Draft'}
                    </span>
                  </div>
                </div>
                <div className="workflow-controls">
                  <button
                    onClick={() => setSelectedWorkflow(
                      selectedWorkflow?.id === workflow.id ? null : workflow
                    )}
                    className="btn-toggle"
                  >
                    {selectedWorkflow?.id === workflow.id ? t('common.collapse') : t('common.expand')}
                  </button>
                  <button
                    onClick={() => executeWorkflow(workflow.id)}
                    disabled={executingWorkflowId === workflow.id}
                    className="btn-execute"
                  >
                    {executingWorkflowId === workflow.id ? t('common.status.processing') : t('common.workflow.execute')}
                  </button>
                  <button
                    onClick={() => deleteWorkflow(workflow.id)}
                    className="btn-delete"
                  >
                    {t('common.workflow.delete')}
                  </button>
                </div>
              </div>

              {selectedWorkflow?.id === workflow.id && (
                <div className="workflow-details">
                  <div className="workflow-steps">
                    {workflow.steps && workflow.steps.length > 0 ? (
                      workflow.steps.map((step, index) => (
                        <div key={step.id} className={`workflow-step ${getStepStatusClass(step.status)}`}>
                          <div className="step-header">
                            <div className="step-icon">
                              {getStepStatusIcon(step.status)}
                            </div>
                            <div className="step-info">
                              <div className="step-name">{step.name}</div>
                              <div className="step-tool">工具: {step.tool}</div>
                            </div>
                            <div className="step-number">#{index + 1}</div>
                          </div>

                          {step.depends_on && step.depends_on.length > 0 && (
                            <div className="step-dependencies">
                              依赖: {step.depends_on.join(', ')}
                            </div>
                          )}

                          {step.result && (
                            <div className="step-result">
                              <pre>{JSON.stringify(step.result, null, 2)}</pre>
                            </div>
                          )}

                          {step.error && (
                            <div className="step-error">
                              {t('common.error')}: {step.error}
                              <button
                                onClick={() => retryStep(selectedWorkflow.id, step.id)}
                                className="step-retry-btn"
                                title={t('common.workflow.retry')}
                              >
                                🔄 {t('common.workflow.retry')}
                              </button>
                            </div>
                          )}

                          {/* 步骤控制按钮 */}
                          <div className="step-controls">
                            {step.status === 'running' && (
                              <button
                                onClick={() => pauseWorkflow(selectedWorkflow.id)}
                                className="step-control-btn pause"
                                title={t('common.workflow.pause')}
                              >
                                ⏸️
                              </button>
                            )}
                            {(step.status === 'pending' || step.status === 'paused') && (
                              <button
                                onClick={() => resumeWorkflow(selectedWorkflow.id)}
                                className="step-control-btn resume"
                                title={t('common.workflow.resume')}
                              >
                                ▶️
                              </button>
                            )}
                          </div>
                        </div>
                      ))
                    ) : (
                      <div className="workflow-steps-empty">
                        {t('common.noData')}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default WorkflowVisualizer;