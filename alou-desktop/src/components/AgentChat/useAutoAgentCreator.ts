import { useCallback, useState } from 'react'

/**
 * 智能体信息类型
 */
interface AgentInfo {
  name: string;
  roleDescription: string;
  customPrompt?: string | null;
}

/**
 * 创建智能体参数类型
 */
interface CreateAgentParams {
  name: string;
  roleDescription: string;
  avatarCid: null;
  mcpConfigCid: null;
  mcpPorts: never[];
  diapIdentity: null;
  tempId: null;
  customPrompt?: string | null;
}

/**
 * Hook参数类型
 */
interface UseAutoAgentCreatorParams {
  onCreateAgent: (params: CreateAgentParams) => Promise<void>;
}

/**
 * Hook返回类型
 */
interface UseAutoAgentCreatorReturn {
  isAutoCreating: boolean;
  autoCreationError: string | null;
  lastCreatedAgent: AgentInfo | null;
  autoCreateAgent: (agentInfo: AgentInfo) => Promise<boolean>;
  resetState: () => void;
  hasCreatedAgent: boolean;
  isInProgress: boolean;
}

/**
 * useAutoAgentCreator - 自动创建智能体 Hook
 * 
 * 专门处理通过自然语言指令解析后的智能体自动创建
 * 这个 hook 只负责调用现有的创建函数，使用默认配置
 * 
 * 设计为可独立使用，便于 SDK 集成
 * 
 * @param params - 参数对象
 * @returns 自动创建状态和方法
 */
export const useAutoAgentCreator = ({ onCreateAgent }: UseAutoAgentCreatorParams): UseAutoAgentCreatorReturn => {
  // ==================== 状态管理 ====================
  const [isAutoCreating, setIsAutoCreating] = useState<boolean>(false)
  const [autoCreationError, setAutoCreationError] = useState<string | null>(null)
  const [lastCreatedAgent, setLastCreatedAgent] = useState<AgentInfo | null>(null)

  // ==================== 核心自动创建函数 ====================
  /**
   * 自动创建智能体
   * 
   * @param agentInfo - 智能体信息（从自然语言解析得到）
   * @returns 是否创建成功
   */
  const autoCreateAgent = useCallback(async (agentInfo: AgentInfo): Promise<boolean> => {
    console.log('[useAutoAgentCreator] 开始自动创建智能体:', agentInfo)
    
    if (!onCreateAgent) {
      throw new Error('创建函数未提供，无法自动创建智能体')
    }

    setIsAutoCreating(true)
    setAutoCreationError(null)

    try {
      // 使用默认值：没有头像、使用默认 MCP 配置
      await onCreateAgent({
        name: agentInfo.name,
        roleDescription: agentInfo.roleDescription,
        avatarCid: null, // 没有头像
        mcpConfigCid: null, // 使用默认配置
        mcpPorts: [], // 空端口列表
        diapIdentity: null, // 自动创建 DIAP identity
        tempId: null, // 没有临时 ID
        customPrompt: agentInfo.customPrompt ?? null, // AI 生成的文档系统提示词
      })
      
      console.log('[useAutoAgentCreator] 智能体自动创建成功:', agentInfo.name)
      setLastCreatedAgent(agentInfo)
      return true
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      console.error('[useAutoAgentCreator] 智能体自动创建失败:', message)
      setAutoCreationError(message)
      throw error
    } finally {
      setIsAutoCreating(false)
    }
  }, [onCreateAgent])

  // ==================== 状态重置 ====================
  const resetState = useCallback(() => {
    setIsAutoCreating(false)
    setAutoCreationError(null)
    setLastCreatedAgent(null)
  }, [])

  // ==================== 返回接口 ====================
  return {
    // 状态
    isAutoCreating,
    autoCreationError,
    lastCreatedAgent,
    
    // 核心方法
    autoCreateAgent,
    
    // 工具方法
    resetState,
    
    // 状态检查
    hasCreatedAgent: !!lastCreatedAgent,
    isInProgress: isAutoCreating,
  }
}

export default useAutoAgentCreator
