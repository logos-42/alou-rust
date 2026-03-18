import { useCallback, useState, useRef } from 'react'

/**
 * 智能体信息类型
 */
interface AgentInfo {
  name: string;
  roleDescription: string;
  customPrompt?: string | null;
  id?: string | null;
}

/**
 * 创建智能体参数类型（与 handleCreateAgentSubmit 匹配）
 */
interface CreateAgentParams {
  name: string;
  roleDescription: string;
  avatar_cid: string | null;
  avatar_url: string | null;
  mcp_config_cid: string | null;
  mcp_ports: any[];
  diapIdentity: any;
  sessionId: string | null;
  tempId: string | null;
  customPrompt?: string | null;
  id?: string | null; // 来自 agent:created 事件的 ID
}

/**
 * Hook参数类型
 */
interface UseAutoAgentCreatorParams {
  sessionId: string | null;
  onCreateAgent: (params: CreateAgentParams) => Promise<any>;
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
  clearCreatedAgents: () => void; // 清空已创建代理记录，允许重新创建同名代理
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
export const useAutoAgentCreator = ({ sessionId, onCreateAgent }: UseAutoAgentCreatorParams): UseAutoAgentCreatorReturn => {
  // ==================== 状态管理 ====================
  const [isAutoCreating, setIsAutoCreating] = useState<boolean>(false)
  const [autoCreationError, setAutoCreationError] = useState<string | null>(null)
  const [lastCreatedAgent, setLastCreatedAgent] = useState<AgentInfo | null>(null)
  
  // 使用 ref 跟踪正在创建和已创建的agent，防止重复创建
  const creatingAgentsRef = useRef<Set<string>>(new Set())
  const createdAgentsRef = useRef<Set<string>>(new Set())

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

    // 生成唯一键用于去重（使用名称+角色描述的组合）
    const agentKey = `${agentInfo.name}_${agentInfo.roleDescription}`.toLowerCase().trim()
    
    // 检查是否正在创建中或已创建
    if (creatingAgentsRef.current.has(agentKey)) {
      console.log('[useAutoAgentCreator] 智能体正在创建中，跳过重复创建:', agentInfo.name)
      return false
    }
    
    if (createdAgentsRef.current.has(agentKey)) {
      console.log('[useAutoAgentCreator] 智能体已创建过，跳过重复创建:', agentInfo.name)
      return false
    }

    // 标记为正在创建
    creatingAgentsRef.current.add(agentKey)
    setIsAutoCreating(true)
    setAutoCreationError(null)

    console.log('[useAutoAgentCreator] 开始创建智能体, sessionId:', sessionId, 'agentInfo:', agentInfo)
    
    try {
      // 使用默认值：没有头像、使用默认 MCP 配置
      const createParams = {
        name: agentInfo.name,
        roleDescription: agentInfo.roleDescription,
        avatar_cid: null, // 没有头像
        avatar_url: null,
        mcp_config_cid: null, // 使用默认配置
        mcp_ports: [], // 空端口列表
        diapIdentity: null, // 自动创建 DIAP identity
        sessionId: sessionId, // 传递 sessionId
        tempId: null, // 没有临时 ID
        customPrompt: agentInfo.customPrompt ?? null, // AI 生成的文档系统提示词
        id: agentInfo.id ?? null, // 传递 id（来自事件）
      }
      console.log('[useAutoAgentCreator] 调用 onCreateAgent, params:', createParams)
      
      await onCreateAgent(createParams)
      
      console.log('[useAutoAgentCreator] 智能体自动创建成功:', agentInfo.name)
      setLastCreatedAgent(agentInfo)
      // 标记为已创建
      createdAgentsRef.current.add(agentKey)
      return true
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      console.error('[useAutoAgentCreator] 智能体自动创建失败:', message)
      setAutoCreationError(message)
      throw error
    } finally {
      // 从正在创建集合中移除
      creatingAgentsRef.current.delete(agentKey)
      setIsAutoCreating(false)
    }
  }, [onCreateAgent, sessionId])

  // ==================== 状态重置 ====================
  const resetState = useCallback(() => {
    setIsAutoCreating(false)
    setAutoCreationError(null)
    setLastCreatedAgent(null)
    creatingAgentsRef.current.clear()
    createdAgentsRef.current.clear()
    console.log('[useAutoAgentCreator] 状态已重置')
  }, [])

  // ==================== 清空已创建记录 ====================
  /**
   * 清空已创建代理记录
   * 用于允许重新创建同名代理（例如删除后重新创建）
   */
  const clearCreatedAgents = useCallback(() => {
    createdAgentsRef.current.clear()
    console.log('[useAutoAgentCreator] 已清空已创建代理记录')
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
    clearCreatedAgents,

    // 状态检查
    hasCreatedAgent: !!lastCreatedAgent,
    isInProgress: isAutoCreating,
  }
}

export default useAutoAgentCreator
