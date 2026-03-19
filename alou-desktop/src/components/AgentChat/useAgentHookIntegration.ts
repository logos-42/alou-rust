/**
 * Agent Hook 集成 Hook
 * 
 * 允许在 Agent 执行过程中实时注入指令、暂停、恢复、取消执行
 */

import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface UseAgentHookIntegrationProps {
  activeChannelId: string | null
  sessionId: string
  isLoading: boolean
}

export interface AgentHookStatus {
  status: 'idle' | 'executing' | 'paused' | 'completed' | 'cancelled' | 'failed'
  current_task?: string
  progress: number
  pending_instructions: number
  processed_instructions: number
  snapshot_time: number
}

export interface InstructionResult {
  success: boolean
  instruction_id?: string
  message: string
}

export interface UseAgentHookIntegrationReturn {
  injectInstruction: (
    content: string,
    priority?: 'low' | 'medium' | 'high' | 'critical'
  ) => Promise<InstructionResult>
  pauseAgent: (reason: string) => Promise<InstructionResult>
  resumeAgent: () => Promise<InstructionResult>
  cancelAgent: (reason: string) => Promise<InstructionResult>
  getAgentStatus: () => Promise<AgentHookStatus | null>
  agentStatus: AgentHookStatus | null
  hasPendingInstructions: boolean
  isExecuting: boolean
  refreshStatus: () => Promise<void>
}

export function useAgentHookIntegration({
  activeChannelId,
  sessionId,
  isLoading,
}: UseAgentHookIntegrationProps): UseAgentHookIntegrationReturn {
  const [agentStatus, setAgentStatus] = useState<AgentHookStatus | null>(null)
  const [hasPendingInstructions, setHasPendingInstructions] = useState(false)
  const [isExecuting, setIsExecuting] = useState(false)

  const injectInstruction = useCallback(
    async (
      content: string,
      priority: 'low' | 'medium' | 'high' | 'critical' = 'medium'
    ): Promise<InstructionResult> => {
      if (!activeChannelId) {
        throw new Error('没有活动的智能体频道')
      }

      try {
        console.log('[AgentHook] 注入指令:', { agent_id: activeChannelId, content, priority })

        const result = await invoke('agent_hook_inject', {
          agent_id: activeChannelId,
          session_id: sessionId,
          content,
          priority,
        })

        console.log('[AgentHook] 指令注入结果:', result)

        const typedResult = result as InstructionResult
        if (typedResult.success) {
          await refreshStatus()
          return typedResult
        } else {
          throw new Error(typedResult.message || '指令注入失败')
        }
      } catch (error) {
        console.error('[AgentHook] 指令注入失败:', error)
        throw error
      }
    },
    [activeChannelId, sessionId]
  )

  const pauseAgent = useCallback(
    async (reason: string): Promise<InstructionResult> => {
      if (!activeChannelId) {
        throw new Error('没有活动的智能体频道')
      }

      try {
        const result = await invoke('agent_hook_pause', {
          agent_id: activeChannelId,
          session_id: sessionId,
          reason,
        })

        await refreshStatus()
        return result as InstructionResult
      } catch (error) {
        console.error('[AgentHook] 暂停失败:', error)
        throw error
      }
    },
    [activeChannelId, sessionId]
  )

  const resumeAgent = useCallback(async (): Promise<InstructionResult> => {
    if (!activeChannelId) {
      throw new Error('没有活动的智能体频道')
    }

    try {
      const result = await invoke('agent_hook_resume', {
        agent_id: activeChannelId,
        session_id: sessionId,
      })

      await refreshStatus()
      return result as InstructionResult
    } catch (error) {
      console.error('[AgentHook] 恢复失败:', error)
      throw error
    }
  }, [activeChannelId, sessionId])

  const cancelAgent = useCallback(
    async (reason: string): Promise<InstructionResult> => {
      if (!activeChannelId) {
        throw new Error('没有活动的智能体频道')
      }

      try {
        const result = await invoke('agent_hook_cancel', {
          agent_id: activeChannelId,
          session_id: sessionId,
          reason,
        })

        await refreshStatus()
        return result as InstructionResult
      } catch (error) {
        console.error('[AgentHook] 取消失败:', error)
        throw error
      }
    },
    [activeChannelId, sessionId]
  )

  const getAgentStatus = useCallback(async (): Promise<AgentHookStatus | null> => {
    if (!activeChannelId) {
      return null
    }

    try {
      const result = await invoke('agent_hook_get_status', {
        agent_id: activeChannelId,
        session_id: sessionId,
      })

      return result as AgentHookStatus
    } catch (error) {
      console.error('[AgentHook] 获取状态失败:', error)
      return null
    }
  }, [activeChannelId, sessionId])

  const refreshStatus = useCallback(async () => {
    const status = await getAgentStatus()
    setAgentStatus(status)
    setHasPendingInstructions(status?.pending_instructions ? status.pending_instructions > 0 : false)
    setIsExecuting(status?.status === 'executing')
  }, [getAgentStatus])

  useEffect(() => {
    if (!activeChannelId) {
      setAgentStatus(null)
      setHasPendingInstructions(false)
      setIsExecuting(false)
      return
    }

    refreshStatus()
    const interval = setInterval(refreshStatus, 2000)
    return () => clearInterval(interval)
  }, [activeChannelId, refreshStatus])

  return {
    injectInstruction,
    pauseAgent,
    resumeAgent,
    cancelAgent,
    getAgentStatus,
    agentStatus,
    hasPendingInstructions,
    isExecuting,
    refreshStatus,
  }
}
