import { useCallback, useEffect, useState } from 'react'
import useAgentStore, { useAgentStoreHydration } from '@/stores/agentStore'
import { buildChannelFromAgent } from './agentUtils'

/**
 * Agent Persistence Hook
 * 处理智能体的本地持久化，与 agentStore 集成
 */
export const useAgentPersistence = ({ sessionId, channels, setChannels }) => {
  // Hydration 状态
  const hasHydrated = useAgentStoreHydration()
  const [isReady, setIsReady] = useState(false)
  
  // Agent Store hooks
  const addAgentToStore = useAgentStore((state) => state.addAgent)
  const getAgentFromStore = useAgentStore((state) => state.getAgent)
  const getAgentByTarget = useAgentStore((state) => state.getAgentByTarget)
  const importAgentToStore = useAgentStore((state) => state.importAgent)
  const storedAgents = useAgentStore((state) => state.agents)

  // 等待 hydration 完成
  useEffect(() => {
    if (hasHydrated) {
      console.log(`[useAgentPersistence] Store hydration 完成，已加载 ${storedAgents.length} 个智能体`)
      setIsReady(true)
    }
  }, [hasHydrated, storedAgents.length])

  // 从本地存储加载智能体到channels
  const loadStoredAgents = useCallback(() => {
    if (!isReady || !storedAgents || storedAgents.length === 0) {
      return []
    }

    const localChannels = storedAgents
      .map(agent => buildChannelFromAgent(agent))
      .filter(Boolean)
    
    return localChannels
  }, [storedAgents, isReady])

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

  // 从网络导入智能体（IPFS/IPNS 解析后添加到本地）
  const importAgentFromNetwork = useCallback((agentData) => {
    try {
      const result = importAgentToStore(agentData)
      
      if (result.isNew) {
        console.log('[useAgentPersistence] 从网络导入新智能体:', result.agent.id)
      } else {
        console.log('[useAgentPersistence] 智能体已存在:', result.agent.id)
      }
      
      return result
    } catch (error) {
      console.error('[useAgentPersistence] 从网络导入智能体失败:', error)
      return { agent: null, isNew: false, error }
    }
  }, [importAgentToStore])

  // 检查智能体是否已存在于本地
  const isAgentStored = useCallback((target) => {
    return getAgentByTarget(target) !== null
  }, [getAgentByTarget])

  return {
    isReady,
    storedAgents,
    loadStoredAgents,
    mergeAgentsWithStorage,
    saveAgentToStorage,
    getStoredAgent,
    importAgentFromNetwork,
    isAgentStored,
  }
}
