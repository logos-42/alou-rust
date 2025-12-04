import { create } from 'zustand'
import { persist, createJSONStorage } from 'zustand/middleware'

/**
 * 智能体数据模型
 * @typedef {Object} AgentMetadata
 * @property {string} id - 智能体唯一标识（IPNS/CID/DID）
 * @property {string} sessionId - 会话ID
 * @property {string} name - 智能体名称
 * @property {string} display_name - 显示名称
 * @property {string} role_description - 角色描述
 * @property {string} avatar_cid - 头像CID
 * @property {string} avatar_url - 头像URL
 * @property {string} mcp_config_cid - MCP配置CID
 * @property {Array} mcp_ports - MCP端口配置
 * @property {string} agent_type - 智能体类型
 * @property {string} ipns - IPNS名称
 * @property {string} cid - CID
 * @property {string} did - DID标识
 * @property {Object} diapIdentity - DIAP身份信息
 * @property {string} customPrompt - 自定义prompt
 * @property {number} created_at - 创建时间戳
 * @property {number} updated_at - 更新时间戳
 */

const STORAGE_KEY = 'alou_agents'

const useAgentStore = create(
  persist(
    (set, get) => ({
      // 智能体列表
      agents: [],
      
      // hydration 状态
      _hasHydrated: false,
      
      // 设置 hydration 完成状态
      setHasHydrated: (state) => {
        set({ _hasHydrated: state })
      },
      
      // 添加智能体
      addAgent: (agentData) => {
        const agents = get().agents
        const now = Date.now()
        
        const newAgent = {
          id: agentData.id || agentData.ipns || agentData.cid || `agent_${Date.now()}`,
          sessionId: agentData.sessionId,
          name: agentData.name || agentData.display_name || '未命名智能体',
          display_name: agentData.display_name || agentData.name || '未命名智能体',
          role_description: agentData.role_description || '',
          avatar_cid: agentData.avatar_cid || null,
          avatar_url: agentData.avatar_url || agentData.avatar || null,
          mcp_config_cid: agentData.mcp_config_cid || null,
          mcp_ports: agentData.mcp_ports || [],
          agent_type: agentData.agent_type || 'claude_agent_sdk',
          ipns: agentData.ipns || agentData.diapIdentity?.ipns || null,
          cid: agentData.cid || agentData.diapIdentity?.cid || null,
          did: agentData.did || agentData.diapIdentity?.did || null,
          diapIdentity: agentData.diapIdentity || null,
          customPrompt: agentData.customPrompt || null,
          created_at: agentData.created_at || now,
          updated_at: now,
        }
        
        // 检查是否已存在（根据id或sessionId）
        const existingIndex = agents.findIndex(
          a => a.id === newAgent.id || a.sessionId === newAgent.sessionId
        )
        
        let updatedAgents
        if (existingIndex >= 0) {
          // 更新现有智能体
          updatedAgents = [...agents]
          updatedAgents[existingIndex] = {
            ...updatedAgents[existingIndex],
            ...newAgent,
            created_at: updatedAgents[existingIndex].created_at, // 保留原始创建时间
          }
          console.log(`[AgentStore] 更新智能体: ${newAgent.id}`)
        } else {
          // 添加新智能体
          updatedAgents = [newAgent, ...agents]
          console.log(`[AgentStore] 添加新智能体: ${newAgent.id}`)
        }
        
        set({ agents: updatedAgents })
        return newAgent
      },
      
      // 更新智能体
      updateAgent: (idOrSessionId, updates) => {
        const agents = get().agents
        const index = agents.findIndex(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        )
        
        if (index < 0) {
          console.warn(`[AgentStore] 未找到智能体: ${idOrSessionId}`)
          return null
        }
        
        const updatedAgents = [...agents]
        updatedAgents[index] = {
          ...updatedAgents[index],
          ...updates,
          updated_at: Date.now(),
        }
        
        set({ agents: updatedAgents })
        console.log(`[AgentStore] 更新智能体: ${idOrSessionId}`)
        return updatedAgents[index]
      },
      
      // 删除智能体
      removeAgent: (idOrSessionId) => {
        const agents = get().agents
        const filtered = agents.filter(
          a => a.id !== idOrSessionId && a.sessionId !== idOrSessionId
        )
        
        set({ agents: filtered })
        console.log(`[AgentStore] 删除智能体: ${idOrSessionId}`)
        return filtered
      },
      
      // 根据ID或sessionId获取智能体
      getAgent: (idOrSessionId) => {
        const agents = get().agents
        return agents.find(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        ) || null
      },
      
      // 根据IPNS/CID/DID获取智能体
      getAgentByTarget: (target) => {
        const agents = get().agents
        return agents.find(
          a => a.ipns === target || a.cid === target || a.did === target || a.id === target
        ) || null
      },
      
      // 清空所有智能体
      clearAgents: () => {
        set({ agents: [] })
        console.log('[AgentStore] 清空所有智能体')
      },
      
      // 从网络导入智能体（用于 IPFS/IPNS 解析后添加）
      importAgent: (agentData) => {
        const agents = get().agents
        
        // 检查是否已存在
        const target = agentData.ipns || agentData.cid || agentData.did
        const existing = agents.find(
          a => a.ipns === target || a.cid === target || a.did === target
        )
        
        if (existing) {
          console.log(`[AgentStore] 智能体已存在: ${target}`)
          return { agent: existing, isNew: false }
        }
        
        const newAgent = get().addAgent({
          ...agentData,
          imported_at: Date.now(),
          source: 'network',
        })
        
        console.log(`[AgentStore] 从网络导入智能体: ${target}`)
        return { agent: newAgent, isNew: true }
      },
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ agents: state.agents }),
      onRehydrateStorage: () => (state, error) => {
        if (error) {
          console.error('[AgentStore] Hydration 失败:', error)
        } else if (state) {
          console.log(`[AgentStore] Hydration 完成，已加载 ${state.agents?.length || 0} 个智能体`)
          state.setHasHydrated(true)
        }
      },
      // 数据迁移：处理旧版本数据格式
      migrate: (persistedState, version) => {
        if (version === 0) {
          // 验证数据完整性
          const agents = persistedState.agents || []
          persistedState.agents = agents.filter(agent => {
            if (!agent.id && !agent.sessionId) {
              console.warn('[AgentStore] 迁移时过滤无效数据:', agent)
              return false
            }
            return true
          })
        }
        return persistedState
      },
      version: 1,
    }
  )
)

// 等待 hydration 完成的 hook
export const useAgentStoreHydration = () => {
  return useAgentStore((state) => state._hasHydrated)
}

// 等待 hydration 完成的 Promise
export const waitForHydration = () => {
  return new Promise((resolve) => {
    if (useAgentStore.getState()._hasHydrated) {
      resolve()
      return
    }
    
    const unsubscribe = useAgentStore.subscribe((state) => {
      if (state._hasHydrated) {
        unsubscribe()
        resolve()
      }
    })
  })
}

export default useAgentStore
