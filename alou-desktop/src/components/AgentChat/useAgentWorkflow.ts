import { useCallback, useMemo } from 'react'
import { useWorkflow, Workflow, AgentInfo } from '@/hooks/useWorkflow'

// 智能体类型
interface Agent {
  display_name?: string;
  name?: string;
  role_description?: string;
}

// 消息类型
interface Message {
  id: string;
  type: 'assistant';
  content: string;
  timestamp: number;
  source: string;
}

// 工作流事件类型
interface WorkflowEvent {
  type: string;
  workflowId?: string;
  progress?: { currentStep?: string };
  result?: unknown;
  error?: string;
  stepId?: string;
  message?: string;
}

// Hook参数类型
interface UseAgentWorkflowParams {
  sessionId: string | null;
  selectedAgent: Agent | null;
  appendMessage: (message: Message) => void;
  scrollToBottom: () => void;
  recordInteraction: (action: string, event: unknown) => void;
}

// 工作流进度类型
interface ExecutionProgress {
  currentStep?: string;
  progress?: number;
}

// 工作流面板配置类型
interface WorkflowPanelConfig {
  workflows: Workflow[];
  selectedWorkflow: Workflow | null;
  setSelectedWorkflow: (workflow: Workflow | null) => void;
  workflowLoading: boolean;
  executingWorkflowId: string | null;
  executionProgress: ExecutionProgress;
  activeExecutions: Record<string, unknown>;
  loadWorkflows: () => Promise<void>;
  createSampleWorkflow: () => Promise<unknown>;
  executeWorkflow: (workflowId: string, apiKey?: string, agentInfo?: AgentInfo) => Promise<unknown>;
  deleteWorkflow: (workflowId: string) => Promise<unknown>;
  retryStep: (workflowId: string, stepId: string, apiKey?: string, agentInfo?: AgentInfo) => Promise<unknown>;
  pauseWorkflow: (workflowId: string) => Promise<unknown>;
  resumeWorkflow: (workflowId: string, apiKey?: string, agentInfo?: AgentInfo) => Promise<unknown>;
  cancelWorkflow: (executionId: string) => Promise<unknown>;
  startExecutionPolling: (executionId: string) => void;
  stopExecutionPolling: (executionId: string) => void;
  handleWorkflowMessage: (message: unknown) => void;
  handleWorkflowEvent: (event: WorkflowEvent) => void;
  agentInfo: AgentInfo;
}

/**
 * useAgentWorkflow - 智能体工作流管理Hook
 * 
 * 封装工作流相关的状态管理和操作方法，与智能体聊天界面集成
 * 
 * @param params - 参数对象
 * @returns 工作流状态和方法
 */
export const useAgentWorkflow = ({
  sessionId,
  selectedAgent,
  appendMessage,
  scrollToBottom,
  recordInteraction,
}: UseAgentWorkflowParams): WorkflowPanelConfig => {
  // ==================== 工作流消息处理 ====================
  const handleWorkflowMessage = useCallback((message: unknown) => {
    let messageContent = ''
    
    // 处理不同类型的消息
    if (typeof message === 'string') {
      messageContent = message
    } else if (message && typeof message === 'object') {
      const msgObj = message as { type?: string; message?: string; error?: { message?: string } | string }
      // 如果是错误对象，提取错误信息
      if (msgObj.type === 'error') {
        messageContent = `❌ ${msgObj.message || '工作流错误'}: ${(msgObj.error as { message?: string })?.message || msgObj.error || '未知错误'}`
      } else if (msgObj.message) {
        messageContent = msgObj.message
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
    activeExecutions,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    cancelWorkflow,
    startExecutionPolling,
    stopExecutionPolling,
  } = useWorkflow({
    sessionId,
    agentInfo: selectedAgent ? {
      name: selectedAgent.display_name || selectedAgent.name || 'Agent',
      role_description: selectedAgent.role_description || 'AI Assistant',
      mode: 'agent'
    } : undefined,
    onWorkflowMessage: handleWorkflowMessage,
  })

  // ==================== 工作流事件处理 ====================
  const handleWorkflowEvent = useCallback((event: WorkflowEvent) => {
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
  const agentInfo = useMemo<AgentInfo>(() => 
    selectedAgent ? {
      name: selectedAgent.display_name || selectedAgent.name || 'Agent',
      role_description: selectedAgent.role_description || 'AI Assistant',
      mode: 'agent'
    } : { name: '', role_description: '', mode: 'agent' },
    [selectedAgent]
  )

  // ==================== 工作流面板配置 ====================
  const workflowPanelConfig: WorkflowPanelConfig = useMemo(() => ({
    workflows,
    selectedWorkflow,
    setSelectedWorkflow,
    workflowLoading,
    executingWorkflowId,
    executionProgress,
    activeExecutions,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    cancelWorkflow,
    startExecutionPolling,
    stopExecutionPolling,
    handleWorkflowMessage,
    handleWorkflowEvent,
    agentInfo,
  }), [
    workflows,
    selectedWorkflow,
    workflowLoading,
    executingWorkflowId,
    executionProgress,
    activeExecutions,
    loadWorkflows,
    createSampleWorkflow,
    executeWorkflow,
    deleteWorkflow,
    retryStep,
    pauseWorkflow,
    resumeWorkflow,
    // cancelWorkflow 是稳定的 useCallback，不需要放在依赖数组中
    startExecutionPolling,
    stopExecutionPolling,
    handleWorkflowMessage,
    handleWorkflowEvent,
    agentInfo,
  ])

  return workflowPanelConfig
}

export default useAgentWorkflow
