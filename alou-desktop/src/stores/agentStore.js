import { create } from 'zustand'
import { persist } from 'zustand/middleware'

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

/**
 * 从localStorage加载智能体列表
 */
const loadAgentsFromStorage = () => {
  if (typeof window === 'undefined') return []
  
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return []
    
    const parsed = JSON.parse(stored)
    if (!Array.isArray(parsed)) {
      console.warn('[AgentStore] 存储的数据格式不正确，重置为空数组')
      return []
    }
    
    // 验证数据完整性
    return parsed.filter(agent => {
      if (!agent.id || !agent.sessionId) {
        console.warn('[AgentStore] 发现无效的智能体数据，已过滤:', agent)
        return false
      }
      return true
    })
  } catch (error) {
    console.error('[AgentStore] 从localStorage加载智能体失败:', error)
    // 如果数据损坏，清空存储
    try {
      localStorage.removeItem(STORAGE_KEY)
    } catch (e) {
      console.error('[AgentStore] 清空损坏的存储失败:', e)
    }
    return []
  }
}

/**
 * 保存智能体列表到localStorage
 */
const saveAgentsToStorage = (agents) => {
  if (typeof window === 'undefined') return false
  
  try {
    const serialized = JSON.stringify(agents)
    localStorage.setItem(STORAGE_KEY, serialized)
    return true
  } catch (error) {
    console.error('[AgentStore] 保存智能体到localStorage失败:', error)
    // 检查是否是存储空间不足
    if (error.name === 'QuotaExceededError') {
      console.error('[AgentStore] localStorage存储空间不足，请清理数据')
    }
    return false
  }
}

const useAgentStore = create(
  persist(
    (set, get) => ({
      // 智能体列表
      agents: [],
      
      // 初始化：从localStorage加载
      init: () => {
        const loaded = loadAgentsFromStorage()
        set({ agents: loaded })
        console.log(`[AgentStore] 已加载 ${loaded.length} 个智能体`)
        return loaded
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
        } else {
          // 添加新智能体
          updatedAgents = [newAgent, ...agents]
        }
        
        set({ agents: updatedAgents })
        const saved = saveAgentsToStorage(updatedAgents)
        if (!saved) {
          console.error('[AgentStore] 保存智能体失败')
        }
        
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
        const saved = saveAgentsToStorage(updatedAgents)
        if (!saved) {
          console.error('[AgentStore] 更新智能体失败')
        }
        
        return updatedAgents[index]
      },
      
      // 删除智能体
      removeAgent: (idOrSessionId) => {
        const agents = get().agents
        const filtered = agents.filter(
          a => a.id !== idOrSessionId && a.sessionId !== idOrSessionId
        )
        
        set({ agents: filtered })
        const saved = saveAgentsToStorage(filtered)
        if (!saved) {
          console.error('[AgentStore] 删除智能体失败')
        }
        
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
        if (typeof window !== 'undefined') {
          localStorage.removeItem(STORAGE_KEY)
        }
      },
    }),
    {
      name: STORAGE_KEY,
      partialize: (state) => ({ agents: state.agents }),
    }
  )
)

// 自动初始化
if (typeof window !== 'undefined') {
  // 在模块加载时初始化
  const store = useAgentStore.getState()
  if (store.agents.length === 0) {
    store.init()
  }
}

export default useAgentStore

