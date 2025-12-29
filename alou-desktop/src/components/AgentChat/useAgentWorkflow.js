import { useCallback, useMemo } from 'react'
import { useWorkflow } from '@/hooks/useWorkflow'

/**
 * useAgentWorkflow - 智能体工作流管理Hook
 * 
 * 封装工作流相关的状态管理和操作方法，与智能体聊天界面集成
 * 
 * @param {Object} params - 参数对象
 * @param {string} params.sessionId - 会话ID
 * @param {Object} params.selectedAgent - 当前选中的智能体
 * @param {Function} params.appendMessage - 添加消息到对话的函数
 * @param {Function} params.scrollToBottom - 滚动到底部的函数
 * @param {Function} params.recordInteraction - 记录交互的函数
 * @returns {Object} 工作流状态和方法
 */
export const useAgentWorkflow = ({
  sessionId,
  selectedAgent,
  appendMessage,
  scrollToBottom,
  recordInteraction,
}) => {
  // ==================== 工作流消息处理 ====================
  // 使用 useCallback 包装 onWorkflowMessage 以避免无限循环
  const handleWorkflowMessage = useCallback((message) => {
    let messageContent = ''
    
    // 处理不同类型的消息
    if (typeof message === 'string') {
      messageContent = message
    } else if (message && typeof message === 'object') {
      // 如果是错误对象，提取错误信息
      if (message.type === 'error') {
        messageContent = `❌ ${message.message || '工作流错误'}: ${message.error?.message || message.error || '未知错误'}`
      } else if (message.message) {
        messageContent = message.message
      } else {
        // 尝试将对象转换为字符串
        try {
          messageContent = JSON.stringify(message, null, 2)
        } catch {
          messageContent = String(message)
        }
      }
    } else {
      messageContent = String(message)
    }
    
    // 将工作流消息添加到对话中
    appendMessage({
      id: `workflow_${Date.now()}_${Math.random()}`,
      type: 'assistant',
      content: messageContent,
      timestamp: Date.now(),
      source: 'workflow',
    })
    scrollToBottom()
  }, [appendMessage, scrollToBottom])

  // ==================== 工作流Hook ====================
  const {
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    isLoading: workflowLoading,
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
    agentInfo: selectedAgent ? {
      name: selectedAgent.display_name || selectedAgent.name || 'Agent',
      role_description: selectedAgent.role_description || 'AI Assistant',
      mode: 'agent'
    } : {},
    onWorkflowMessage: handleWorkflowMessage,
  })

  // ==================== 工作流事件处理 ====================
  const handleWorkflowEvent = useCallback((event) => {
    console.log('[useAgentWorkflow] Workflow event:', event)
    recordInteraction('workflow_event', event)

    // 将工作流事件转换为对话消息显示
    let messageContent = ''

    switch (event.type) {
      case 'execution_started':
        messageContent = `🔄 开始执行工作流 ${event.workflowId}`
        break

      case 'execution_progress':
        if (event.progress?.currentStep) {
          messageContent = `⚙️ 当前执行步骤: ${event.progress.currentStep}`
        }
        break

      case 'execution_completed':
        messageContent = `✅ 工作流执行完成!\n结果: ${JSON.stringify(event.result || {}, null, 2)}`
        break

      case 'execution_error':
        messageContent = `❌ 工作流执行失败: ${event.error || '未知错误'}`
        break

      case 'workflow_created':
        messageContent = `✨ 工作流创建成功: ${event.workflowId}`
        break

      case 'workflow_deleted':
        messageContent = `🗑️ 工作流删除成功: ${event.workflowId}`
        break

      case 'step_retried':
        messageContent = `🔄 步骤重试成功: ${event.stepId}`
        break

      case 'workflow_paused':
        messageContent = `⏸️ 工作流已暂停: ${event.workflowId}`
        break

      case 'workflow_resumed':
        messageContent = `▶️ 工作流已恢复: ${event.workflowId}`
        break

      case 'error':
        messageContent = `❌ 工作流错误: ${event.message || event.error || '未知错误'}`
        break
    }

    if (messageContent) {
      appendMessage({
        id: `workflow_${Date.now()}_${Math.random()}`,
        type: 'assistant',
        content: messageContent,
        timestamp: Date.now(),
        source: 'workflow',
      })
      scrollToBottom()
    }
  }, [appendMessage, scrollToBottom, recordInteraction])

  // ==================== 智能体信息计算 ====================
  // 计算智能体信息，用于工作流执行
  const agentInfo = useMemo(() => 
    selectedAgent ? {
      name: selectedAgent.display_name || selectedAgent.name || 'Agent',
      role_description: selectedAgent.role_description || 'AI Assistant',
      mode: 'agent'
    } : {},
    [selectedAgent]
  )

  // ==================== 工作流面板配置 ====================
  // 注意：showWorkflowPanel 状态需要在父组件中管理，因为它是UI状态
  // 这里只提供相关的方法和计算属性

  // 工作流面板的配置
  const workflowPanelConfig = useMemo(() => ({
    // 工作流数据
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    workflowLoading,
    executingWorkflowId,
    executionProgress,
    
    // 工作流操作方法
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    
    // 事件处理
    handleWorkflowMessage,
    handleWorkflowEvent,
    
    // 智能体信息（用于工作流执行）
    agentInfo,
  }), [
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    workflowLoading,
    executingWorkflowId,
    executionProgress,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    handleWorkflowMessage,
    handleWorkflowEvent,
    agentInfo,
  ])

  return workflowPanelConfig
}
