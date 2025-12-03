import { useCallback, useEffect } from 'react'
import useAgentStore from '@/stores/agentStore'
import { buildChannelFromAgent } from './agentUtils'

/**
 * Agent Persistence Hook
 * 处理智能体的本地持久化，与 agentStore 集成
 */
export const useAgentPersistence = ({ sessionId, channels, setChannels }) => {
  // Agent Store hooks
  const agentStoreInit = useAgentStore((state) => state.init)
  const addAgentToStore = useAgentStore((state) => state.addAgent)
  const getAgentFromStore = useAgentStore((state) => state.getAgent)
  const storedAgents = useAgentStore((state) => state.agents)

  // 初始化：从localStorage加载已保存的智能体
  useEffect(() => {
    agentStoreInit()
  }, [agentStoreInit])

  // 从本地存储加载智能体到channels
  const loadStoredAgents = useCallback(() => {
    if (!storedAgents || storedAgents.length === 0) {
      return []
    }

    const localChannels = storedAgents
      .map(agent => buildChannelFromAgent(agent))
      .filter(Boolean)
    
    return localChannels
  }, [storedAgents])

  // 合并后端和本地存储的智能体
  const mergeAgentsWithStorage = useCallback((backendChannels = []) => {
    const localChannels = loadStoredAgents()
    
    if (localChannels.length === 0) {
      return backendChannels
    }

    // 合并去重（优先使用后端数据）
    const seen = new Set(backendChannels.map(c => c.id))
    const merged = [...backendChannels]
    
    localChannels.forEach(lc => {
      if (!seen.has(lc.id)) {
        merged.push(lc)
        seen.add(lc.id)
      }
    })
    
    return merged
  }, [loadStoredAgents])

  // 保存智能体到本地存储
  const saveAgentToStorage = useCallback((agentMetadata) => {
    try {
      const agentData = {
        ...agentMetadata,
        sessionId,
        id: agentMetadata.id || agentMetadata.ipns || agentMetadata.cid || `agent_${Date.now()}`,
      }
      
      addAgentToStore(agentData)
      console.log('[useAgentPersistence] 智能体已保存到本地存储:', agentData.id)
      return true
    } catch (error) {
      console.error('[useAgentPersistence] 保存智能体到本地存储失败:', error)
      return false
    }
  }, [addAgentToStore, sessionId])

  // 从存储中获取智能体
  const getStoredAgent = useCallback((idOrSessionId) => {
    return getAgentFromStore(idOrSessionId)
  }, [getAgentFromStore])

  return {
    storedAgents,
    loadStoredAgents,
    mergeAgentsWithStorage,
    saveAgentToStorage,
    getStoredAgent,
  }
}

