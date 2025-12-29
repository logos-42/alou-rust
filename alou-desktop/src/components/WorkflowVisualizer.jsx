import React, { useState, useEffect, useCallback } from 'react'
import { useI18n } from '../hooks/useI18n'
import workflowService from '../services/workflowService'
import './WorkflowVisualizer.css'

/**
 * WorkflowVisualizer - 桌面版工作流可视化和控制组件
 * 直接使用Claude Agent SDK执行工作流
 */
const WorkflowVisualizer = ({
  sessionId,
  apiKey,
  agentInfo = {},
  onWorkflowEvent,
  className = ''
}) => {
  const { t } = useI18n()
  const [workflows, setWorkflows] = useState([])
  const [selectedWorkflow, setSelectedWorkflow] = useState(null)
  const [isLoading, setIsLoading] = useState(false)
  const [executingWorkflowId, setExecutingWorkflowId] = useState(null)
  const [executionProgress, setExecutionProgress] = useState({})

  // 加载工作流列表
  const loadWorkflows = useCallback(async () => {
    if (!sessionId) return

    try {
      setIsLoading(true)
      const response = await workflowService.listWorkflows()

      if (response.success) {
        setWorkflows(response.workflows || [])
      } else {
        console.error('[WorkflowVisualizer] 加载工作流列表失败:', response.error)
        onWorkflowEvent?.({
          type: 'error',
          message: '加载工作流列表失败',
          error: response.error
        })
      }
    } catch (error) {
      console.error('[WorkflowVisualizer] 加载工作流列表异常:', error)
      onWorkflowEvent?.({
        type: 'error',
        message: '加载工作流列表异常',
        error
      })
    } finally {
      setIsLoading(false)
    }
  }, [sessionId, onWorkflowEvent])

  // 执行工作流
  const executeWorkflow = async (workflowId) => {
    if (!sessionId || !apiKey || executingWorkflowId) return

    try {
      setExecutingWorkflowId(workflowId)
      setExecutionProgress({ [workflowId]: { status: 'running', currentStep: null } })

      onWorkflowEvent?.({
        type: 'execution_started',
        workflowId
      })

      const response = await workflowService.executeWorkflow(
        workflowId,
        apiKey,
        agentInfo,
        (progress) => {
          // 处理实时进度更新
          setExecutionProgress(prev => ({
            ...prev,
            [workflowId]: {
              ...prev[workflowId],
              ...progress
            }
          }))

          onWorkflowEvent?.({
            type: 'execution_progress',
            workflowId,
            progress
          })
        }
      )

      if (response.success) {
        onWorkflowEvent?.({
          type: 'execution_completed',
          workflowId,
          result: response.result,
          steps: response.steps
        })
      } else {
        onWorkflowEvent?.({
          type: 'execution_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 执行工作流异常:', error)
      onWorkflowEvent?.({
        type: 'execution_error',
        workflowId,
        error
      })
    } finally {
      setExecutingWorkflowId(null)
      setExecutionProgress(prev => {
        const newProgress = { ...prev }
        delete newProgress[workflowId]
        return newProgress
      })
    }
  }

  // 创建示例工作流
  const createSampleWorkflow = async () => {
    if (!sessionId) return

    try {
      setIsLoading(true)
      const sampleWorkflow = workflowService.createSampleWorkflow()

      const response = await workflowService.createWorkflow(sampleWorkflow, agentInfo)

      if (response.success) {
        onWorkflowEvent?.({
          type: 'workflow_created',
          workflowId: response.workflowId,
          workflow: sampleWorkflow
        })

        // 重新加载工作流列表
        await loadWorkflows()
      } else {
        onWorkflowEvent?.({
          type: 'creation_error',
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 创建工作流异常:', error)
      onWorkflowEvent?.({
        type: 'creation_error',
        error
      })
    } finally {
      setIsLoading(false)
    }
  }

  // 删除工作流
  const deleteWorkflow = async (workflowId) => {
    if (!sessionId) return

    try {
      const response = await workflowService.deleteWorkflow(workflowId)

      if (response.success) {
        onWorkflowEvent?.({
          type: 'workflow_deleted',
          workflowId
        })

        // 重新加载工作流列表
        await loadWorkflows()

        // 如果删除的是当前选中的工作流，清除选择
        if (selectedWorkflow?.id === workflowId) {
          setSelectedWorkflow(null)
        }
      } else {
        onWorkflowEvent?.({
          type: 'deletion_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 删除工作流异常:', error)
      onWorkflowEvent?.({
        type: 'deletion_error',
        workflowId,
        error
      })
    }
  }

  // 重试失败的步骤
  const retryStep = async (workflowId, stepId) => {
    if (!sessionId || !apiKey) return

    try {
      const response = await workflowService.retryStep(workflowId, stepId, apiKey, agentInfo)

      if (response.success) {
        onWorkflowEvent?.({
          type: 'step_retried',
          workflowId,
          stepId,
          result: response.result
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowEvent?.({
          type: 'retry_error',
          workflowId,
          stepId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 重试步骤异常:', error)
      onWorkflowEvent?.({
        type: 'retry_error',
        workflowId,
        stepId,
        error
      })
    }
  }

  // 暂停工作流
  const pauseWorkflow = async (workflowId) => {
    if (!sessionId) return

    try {
      const response = await workflowService.pauseWorkflow(workflowId)

      if (response.success) {
        onWorkflowEvent?.({
          type: 'workflow_paused',
          workflowId
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowEvent?.({
          type: 'pause_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 暂停工作流异常:', error)
      onWorkflowEvent?.({
        type: 'pause_error',
        workflowId,
        error
      })
    }
  }

  // 恢复工作流
  const resumeWorkflow = async (workflowId) => {
    if (!sessionId || !apiKey) return

    try {
      const response = await workflowService.resumeWorkflow(workflowId, apiKey, agentInfo)

      if (response.success) {
        onWorkflowEvent?.({
          type: 'workflow_resumed',
          workflowId,
          result: response.result
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowEvent?.({
          type: 'resume_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[WorkflowVisualizer] 恢复工作流异常:', error)
      onWorkflowEvent?.({
        type: 'resume_error',
        workflowId,
        error
      })
    }
  }

  // 获取步骤状态样式
  const getStepStatusClass = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return 'step-completed'
      case 'running': return 'step-running'
      case 'failed': return 'step-failed'
      case 'pending': return 'step-pending'
      case 'paused': return 'step-paused'
      default: return 'step-unknown'
    }
  }

  // 获取步骤状态图标
  const getStepStatusIcon = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return '✅'
      case 'running': return '⏳'
      case 'failed': return '❌'
      case 'pending': return '⏸️'
      case 'paused': return '⏸️'
      default: return '❓'
    }
  }

  // 初始化加载
  useEffect(() => {
    if (sessionId) {
      loadWorkflows()
    }
  }, [sessionId, loadWorkflows])

  return (
    <div className={`workflow-visualizer ${className}`}>
      <div className="workflow-header">
        <div className="workflow-header-top">
          <h3>工作流管理 (Claude SDK)</h3>
          <div className="workflow-stream-status">
            <span className="stream-indicator active"></span>
            <span className="stream-label">本地Claude SDK</span>
          </div>
        </div>
        <div className="workflow-actions">
          <button
            onClick={createSampleWorkflow}
            disabled={isLoading || !sessionId || !apiKey}
            className="btn-create"
          >
            {isLoading ? '创建中...' : '创建示例工作流'}
          </button>
          <button
            onClick={loadWorkflows}
            disabled={isLoading || !sessionId}
            className="btn-refresh"
          >
            刷新
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
            <p>暂无工作流</p>
            <p>点击"创建示例工作流"开始使用Claude SDK执行工作流</p>
            {!apiKey && (
              <p className="warning">⚠️ 请先配置Claude API密钥</p>
            )}
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
                      {workflow.steps?.length || 0} 步骤
                    </span>
                    <span className={`workflow-status status-${workflow.status?.toLowerCase()}`}>
                      {workflow.status === 'pending' ? '等待中' :
                       workflow.status === 'running' ? '执行中' :
                       workflow.status === 'completed' ? '已完成' :
                       workflow.status === 'failed' ? '失败' :
                       workflow.status === 'paused' ? '已暂停' :
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
                    {selectedWorkflow?.id === workflow.id ? '收起' : '展开'}
                  </button>
                  <button
                    onClick={() => executeWorkflow(workflow.id)}
                    disabled={executingWorkflowId === workflow.id || !apiKey}
                    className="btn-execute"
                  >
                    {executingWorkflowId === workflow.id ? '执行中...' : '执行'}
                  </button>
                  <button
                    onClick={() => deleteWorkflow(workflow.id)}
                    className="btn-delete"
                  >
                    删除
                  </button>
                </div>
              </div>

              {selectedWorkflow?.id === workflow.id && (
                <div className="workflow-details">
                  <div className="workflow-steps">
                    {workflow.steps?.map((step, index) => {
                      const progress = executionProgress[workflow.id]
                      const isCurrentStep = progress?.currentStep === step.id
                      const stepStatus = isCurrentStep ? 'running' : (step.status || 'pending')

                      return (
                        <div key={step.id} className={`workflow-step ${getStepStatusClass(stepStatus)} ${isCurrentStep ? 'current' : ''}`}>
                          <div className="step-header">
                            <div className="step-icon">
                              {getStepStatusIcon(stepStatus)}
                              {isCurrentStep && <div className="step-spinner"></div>}
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
                              <pre>{typeof step.result === 'string' ? step.result : JSON.stringify(step.result, null, 2)}</pre>
                            </div>
                          )}

                          {step.error && (
                            <div className="step-error">
                              错误: {step.error}
                              <button
                                onClick={() => retryStep(workflow.id, step.id)}
                                className="step-retry-btn"
                                title="重试此步骤"
                              >
                                🔄 重试
                              </button>
                            </div>
                          )}

                          {/* 步骤控制按钮 */}
                          <div className="step-controls">
                            {step.status === 'running' && (
                              <button
                                onClick={() => pauseWorkflow(workflow.id)}
                                className="step-control-btn pause"
                                title="暂停工作流"
                              >
                                ⏸️
                              </button>
                            )}
                            {(step.status === 'pending' || step.status === 'paused') && (
                              <button
                                onClick={() => resumeWorkflow(workflow.id)}
                                className="step-control-btn resume"
                                title="恢复工作流"
                              >
                                ▶️
                              </button>
                            )}
                          </div>
                        </div>
                      )
                    }) || (
                      <div className="workflow-steps-empty">
                        暂无步骤信息
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
  )
}

export default WorkflowVisualizer