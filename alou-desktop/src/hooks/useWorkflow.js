import { useState, useCallback, useEffect, useRef } from 'react'
import { useI18n } from '@/hooks/useI18n'
import workflowService from '@/services/workflowService'

/**
 * useWorkflow - 桌面版工作流管理Hook
 * 提供工作流相关的状态管理和操作方法
 */
export const useWorkflow = ({
  sessionId,
  apiKey,
  agentInfo = {},
  onWorkflowMessage
}) => {
  const { t } = useI18n()
  const [workflows, setWorkflows] = useState([])
  const [selectedWorkflow, setSelectedWorkflow] = useState(null)
  const [isLoading, setIsLoading] = useState(false)
  const [executingWorkflowId, setExecutingWorkflowId] = useState(null)
  const [executionProgress, setExecutionProgress] = useState({})
  
  // 添加轮询相关状态
  const pollIntervalRef = useRef(null)
  const [pollingWorkflowId, setPollingWorkflowId] = useState(null)

  // 加载工作流列表
  const loadWorkflows = useCallback(async () => {
    if (!sessionId) return

    try {
      setIsLoading(true)
      const response = await workflowService.listWorkflows()

      if (response.success) {
        setWorkflows(response.workflows || [])
      } else {
        console.error('[useWorkflow] 加载工作流列表失败:', response.error)
        onWorkflowMessage?.({
          type: 'error',
          message: '加载工作流列表失败',
          error: response.error
        })
      }
    } catch (error) {
      console.error('[useWorkflow] 加载工作流列表异常:', error)
      onWorkflowMessage?.({
        type: 'error',
        message: '加载工作流列表异常',
        error
      })
    } finally {
      setIsLoading(false)
    }
  }, [sessionId, onWorkflowMessage])

  // 创建示例工作流
  const createSampleWorkflow = useCallback(async () => {
    if (!sessionId) return

    try {
      setIsLoading(true)
      const sampleWorkflow = workflowService.createSampleWorkflow()

      const response = await workflowService.createWorkflow(sampleWorkflow, agentInfo)

      if (response.success) {
        onWorkflowMessage?.({
          type: 'workflow_created',
          workflowId: response.workflowId,
          workflow: sampleWorkflow
        })

        // 重新加载工作流列表
        await loadWorkflows()
      } else {
        onWorkflowMessage?.({
          type: 'creation_error',
          error: response.error
        })
      }

    } catch (error) {
      console.error('[useWorkflow] 创建工作流异常:', error)
      onWorkflowMessage?.({
        type: 'creation_error',
        error
      })
    } finally {
      setIsLoading(false)
    }
  }, [sessionId, agentInfo, onWorkflowMessage, loadWorkflows])


  // 重试失败的步骤
  const retryStep = useCallback(async (workflowId, stepId) => {
    const effectiveApiKey = apiKey || (typeof window !== 'undefined' ? localStorage.getItem('claude_api_key') : null)
    if (!sessionId || !effectiveApiKey) return

    try {
      const response = await workflowService.retryStep(workflowId, stepId, effectiveApiKey, agentInfo)

      if (response.success) {
        onWorkflowMessage?.({
          type: 'step_retried',
          workflowId,
          stepId,
          result: response.result
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowMessage?.({
          type: 'retry_error',
          workflowId,
          stepId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[useWorkflow] 重试步骤异常:', error)
      onWorkflowMessage?.({
        type: 'retry_error',
        workflowId,
        stepId,
        error
      })
    }
  }, [sessionId, apiKey, agentInfo, onWorkflowMessage, loadWorkflows])

  // 恢复工作流
  const resumeWorkflow = useCallback(async (workflowId) => {
    const effectiveApiKey = apiKey || (typeof window !== 'undefined' ? localStorage.getItem('claude_api_key') : null)
    if (!sessionId || !effectiveApiKey) return

    try {
      const response = await workflowService.resumeWorkflow(workflowId, effectiveApiKey, agentInfo)

      if (response.success) {
        onWorkflowMessage?.({
          type: 'workflow_resumed',
          workflowId,
          result: response.result
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowMessage?.({
          type: 'resume_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[useWorkflow] 恢复工作流异常:', error)
      onWorkflowMessage?.({
        type: 'resume_error',
        workflowId,
        error
      })
    }
  }, [sessionId, apiKey, agentInfo, onWorkflowMessage, loadWorkflows])

  // 删除工作流
  const deleteWorkflow = useCallback(async (workflowId) => {
    if (!sessionId) return

    try {
      const response = await workflowService.deleteWorkflow(workflowId)

      if (response.success) {
        onWorkflowMessage?.({
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
        onWorkflowMessage?.({
          type: 'deletion_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[useWorkflow] 删除工作流异常:', error)
      onWorkflowMessage?.({
        type: 'deletion_error',
        workflowId,
        error
      })
    }
  }, [sessionId, selectedWorkflow, onWorkflowMessage, loadWorkflows])

  // 暂停工作流
  const pauseWorkflow = useCallback(async (workflowId) => {
    if (!sessionId) return

    try {
      const response = await workflowService.pauseWorkflow(workflowId)

      if (response.success) {
        onWorkflowMessage?.({
          type: 'workflow_paused',
          workflowId
        })

        // 重新获取工作流状态
        await loadWorkflows()
      } else {
        onWorkflowMessage?.({
          type: 'pause_error',
          workflowId,
          error: response.error
        })
      }

    } catch (error) {
      console.error('[useWorkflow] 暂停工作流异常:', error)
      onWorkflowMessage?.({
        type: 'pause_error',
        workflowId,
        error
      })
    }
  }, [sessionId, onWorkflowMessage, loadWorkflows])

  // 清除轮询
  const clearPolling = useCallback(() => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current)
      pollIntervalRef.current = null
      setPollingWorkflowId(null)
    }
  }, [])

  // 开始轮询工作流状态
  const startPolling = useCallback((workflowId) => {
    clearPolling()
    setPollingWorkflowId(workflowId)
    
    // 立即检查一次状态
    checkWorkflowStatus(workflowId)
    
    // 每3秒检查一次状态
    pollIntervalRef.current = setInterval(() => {
      checkWorkflowStatus(workflowId)
    }, 3000)
  }, [clearPolling])

  // 检查工作流状态
  const checkWorkflowStatus = useCallback(async (workflowId) => {
    try {
      const status = await workflowService.getWorkflowStatus(workflowId)
      
      if (status.success) {
        setExecutionProgress(prev => ({
          ...prev,
          [workflowId]: {
            status: status.data?.status || 'running',
            currentStep: status.data?.current_step || null,
            progress: status.data?.progress || 0
          }
        }))
        
        // 在对话框中显示进度更新
        if (status.data?.current_step) {
          onWorkflowMessage?.(`⚙️ 当前执行步骤: ${status.data.current_step}`)
        }
        
        // 如果任务完成或失败，停止轮询
        if (status.data?.status === 'completed' || status.data?.status === 'failed') {
          clearPolling()
          
          if (status.data?.status === 'completed') {
            onWorkflowMessage?.(`✅ 工作流执行完成!
${JSON.stringify(status.data?.result || {}, null, 2)}`)
          } else {
            onWorkflowMessage?.(`❌ 工作流执行失败: ${status.data?.error || '未知错误'}`)
          }
        }
      }
    } catch (error) {
      console.error('[useWorkflow] 检查状态异常:', error)
    }
  }, [clearPolling, onWorkflowMessage])

  // 修改执行工作流函数，添加轮询
  const executeWorkflow = useCallback(async (workflowId) => {
    // 从localStorage获取API密钥（如果没有传入的话）
    const effectiveApiKey = apiKey || (typeof window !== 'undefined' ? localStorage.getItem('claude_api_key') : null)
    if (!sessionId || !effectiveApiKey || executingWorkflowId) return

    try {
      setExecutingWorkflowId(workflowId)
      setExecutionProgress({ [workflowId]: { status: 'running', currentStep: null } })

      // 在对话框中显示开始执行的消息
      onWorkflowMessage?.(`🔄 开始执行工作流: ${workflowId}`)

      const response = await workflowService.executeWorkflow(
        workflowId,
        effectiveApiKey,
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

          // 在对话框中显示进度更新
          if (progress.currentStep) {
            onWorkflowMessage?.(`⚙️ 当前执行步骤: ${progress.currentStep}`)
          }
        }
      )

      if (response.success) {
        // 开始轮询状态
        startPolling(workflowId)
      } else {
        // 在对话框中显示错误
        onWorkflowMessage?.(`❌ 工作流执行失败: ${response.error}`)
        setExecutingWorkflowId(null)
      }

    } catch (error) {
      console.error('[useWorkflow] 执行工作流异常:', error)
      onWorkflowMessage?.(`❌ 工作流执行异常: ${error.message}`)
      setExecutingWorkflowId(null)
      setExecutionProgress(prev => {
        const newProgress = { ...prev }
        delete newProgress[workflowId]
        return newProgress
      })
    }
  }, [sessionId, apiKey, agentInfo, executingWorkflowId, onWorkflowMessage, startPolling])

  // 清理轮询
  useEffect(() => {
    return () => {
      clearPolling()
    }
  }, [clearPolling])

  // 初始化加载
  useEffect(() => {
    if (sessionId) {
      loadWorkflows()
    }
  }, [sessionId, loadWorkflows])

  return {
    // 状态
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    isLoading,
    executingWorkflowId,
    executionProgress,
    pollingWorkflowId,

    // 方法
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    clearPolling,
    startPolling,
  }
}