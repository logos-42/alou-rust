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
      // IPFS URL 解析辅助方法
      resolveIpfsUrl: (cid) => {
        if (!cid) return null
        
        // 如果已经是完整的 URL，直接返回
        if (cid.startsWith('http')) return cid
        
        // 解析 IPFS CID 为 URL
        // 使用 agentAssetsService
        try {
          // 动态导入以避免循环依赖
          const agentAssetsService = require('@/services/agentAssetsService').default
          return agentAssetsService.resolveIpfsUri(cid)
        } catch (error) {
          console.warn('[AgentStore] IPFS URL 解析失败:', error)
          return null
        }
      },
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
        
        // 重要：使用与 buildChannelFromAgent 相同的ID生成逻辑，确保一致性
        // 1. 如果传入的 id 已经存在，优先使用它（通常是 channel.id）
        // 2. 否则，按照 buildChannelFromAgent 的逻辑生成：ipns || did || cid || timestamp
        // 注意：需要过滤 mock IPNS（与 buildChannelFromAgent 保持一致）
        const mockIpns = 'k51qzi5uq'
        let ipnsValue = agentData.ipns || agentData.diapIdentity?.ipns || null
        const isMockIpns = ipnsValue && (
          ipnsValue.includes(mockIpns) ||
          ipnsValue === mockIpns ||
          ipnsValue === `/ipns/${mockIpns}`
        )
        if (isMockIpns) {
          ipnsValue = null // 过滤掉 mock IPNS
        }
        
        // 使用与 buildChannelFromAgent 相同的ID生成逻辑
        const generatedId = agentData.id || 
                            ipnsValue || 
                            agentData.did || 
                            agentData.cid || 
                            `agent_${Date.now()}`
        
        // 处理头像URL
        const avatarUrl = agentData.avatar_cid ? get().resolveIpfsUrl(agentData.avatar_cid) : (agentData.avatar_url || agentData.avatar || null)
        
        console.log('[AgentStore] 创建/更新智能体头像信息:', {
          name: agentData.name,
          hasAvatarCid: !!agentData.avatar_cid,
          avatar_cid: agentData.avatar_cid,
          resolvedAvatarUrl: avatarUrl?.substring(0, 100)
        })
        
        const newAgent = {
          id: generatedId, // 使用一致的ID生成逻辑
          sessionId: agentData.sessionId,
          name: agentData.name || agentData.display_name || '未命名智能体',
          display_name: agentData.display_name || agentData.name || '未命名智能体',
          role_description: agentData.role_description || '',
          avatar_cid: agentData.avatar_cid || null,
          // 修复：始终设置 avatar_url，如果是 avatar_cid，转换为 IPFS URL
          avatar_url: avatarUrl,
          mcp_config_cid: agentData.mcp_config_cid || null,
          mcp_ports: agentData.mcp_ports || [],
          agent_type: agentData.agent_type || 'ai_agent_sdk',
          ipns: ipnsValue, // 使用过滤后的 IPNS
          cid: agentData.cid || agentData.diapIdentity?.cid || null,
          did: agentData.did || agentData.diapIdentity?.did || null,
          diapIdentity: null, // 不在这里存储DIAP身份，只存储引用信息
          customPrompt: agentData.customPrompt || null,
          messages_cid: agentData.messages_cid || null, // 消息历史的 IPFS CID
          created_at: agentData.created_at || now,
          updated_at: now,
        }
        
        // 检查是否已存在（根据id或sessionId）
        const existingIndex = agents.findIndex(
          a => a.id === newAgent.id || a.sessionId === newAgent.sessionId
        )
        
        let updatedAgents
        if (existingIndex >= 0) {
          // 更新现有智能体 - 保护现有头像
          updatedAgents = [...agents]
          const existingAgent = updatedAgents[existingIndex]
          updatedAgents[existingIndex] = {
            ...existingAgent,
            ...newAgent,
            // 保护现有头像，只有在新数据中明确提供时才更新
            avatar_cid: newAgent.avatar_cid || existingAgent.avatar_cid,
            avatar_url: newAgent.avatar_url || existingAgent.avatar_url,
            created_at: existingAgent.created_at, // 保留原始创建时间
          }
        } else {
          // 添加新智能体
          updatedAgents = [newAgent, ...agents]
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
        const existingAgent = updatedAgents[index]
        
        // 处理头像更新：如果提供了 avatar_cid，需要重新解析为 URL
        let finalAvatarUrl = updates.avatar_url || existingAgent.avatar_url
        if (updates.avatar_cid) {
          finalAvatarUrl = get().resolveIpfsUrl(updates.avatar_cid)
        }
        
        updatedAgents[index] = {
          ...existingAgent,
          ...updates,
          // 保护现有头像，只有在新数据中明确提供时才更新
          avatar_cid: updates.avatar_cid || existingAgent.avatar_cid,
          avatar_url: finalAvatarUrl,
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
        
        // 重要：如果 agentData.id 已存在，优先使用它（通常是 channel.id，已经与 buildChannelFromAgent 一致）
        // 否则，使用 addAgent 的逻辑（它也会使用与 buildChannelFromAgent 一致的ID生成）
        const targetId = agentData.id
        
        // 检查是否已存在（优先使用ID匹配，因为ID是最准确的）
        let existing = null
        if (targetId) {
          existing = agents.find(a => a.id === targetId)
        }
        
        // 如果通过ID没找到，尝试通过 IPNS/CID/DID 匹配
        if (!existing) {
          const target = agentData.ipns || agentData.cid || agentData.did
          if (target) {
            existing = agents.find(
              a => a.ipns === target || a.cid === target || a.did === target || a.id === target
            )
          }
        }
        
        if (existing) {
          // 更新现有智能体（合并新数据）
          const updated = get().addAgent({
            ...existing,
            ...agentData,
            id: existing.id, // 保持原有ID不变
            imported_at: Date.now(),
            source: 'network',
          })
          return { agent: updated, isNew: false }
        }
        
        const newAgent = get().addAgent({
          ...agentData,
          imported_at: Date.now(),
          source: 'network',
        })
        
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
