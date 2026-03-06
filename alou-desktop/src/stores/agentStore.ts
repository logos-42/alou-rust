import { create } from 'zustand'
import { persist, createJSONStorage, PersistOptions } from 'zustand/middleware'

/**
 * 智能体元数据接口
 */
export interface AgentMetadata {
  id: string
  sessionId?: string
  name: string
  display_name?: string
  role_description?: string
  avatar_cid?: string | null
  avatar_url?: string | null
  mcp_config_cid?: string | null
  mcp_ports?: any[]
  agent_type?: string
  ipns?: string | null
  cid?: string | null
  did?: string | null
  diapIdentity?: any | null
  customPrompt?: string | null
  documents?: Record<string, string> | null
  messages_cid?: string | null
  last_saved_at?: number
  created_at: number
  updated_at: number
  imported_at?: number
  source?: string
}

/**
 * AgentStore 状态接口
 */
interface AgentStoreState {
  agents: AgentMetadata[]
  _hasHydrated: boolean
}

/**
 * AgentStore 动作接口
 */
interface AgentStoreActions {
  setHasHydrated: (state: boolean) => void
  resolveIpfsUrl: (cid: string | null | undefined) => string | null
  addAgent: (agentData: Partial<AgentMetadata>) => AgentMetadata
  updateAgent: (idOrSessionId: string, updates: Partial<AgentMetadata>) => AgentMetadata | null
  removeAgent: (idOrSessionId: string) => AgentMetadata[]
  getAgent: (idOrSessionId: string) => AgentMetadata | null
  getAgentByTarget: (target: string) => AgentMetadata | null
  updateAgentDocument: (idOrSessionId: string, docType: string, content: string) => AgentMetadata | null
  clearAgents: () => void
  importAgent: (agentData: Partial<AgentMetadata>) => { agent: AgentMetadata; isNew: boolean }
}

type AgentStore = AgentStoreState & AgentStoreActions

const STORAGE_KEY = 'alou_agents'

const useAgentStore = create<AgentStore>()(
  persist(
    (set, get) => ({
      // 初始状态
      agents: [],
      _hasHydrated: false,

      // IPFS URL 解析辅助方法
      resolveIpfsUrl: (cid: string | null | undefined): string | null => {
        if (!cid) return null

        // 如果已经是完整的 URL，直接返回
        if (cid.startsWith('http')) return cid
        
        // 如果是 data URL，直接返回
        if (cid.startsWith('data:')) return cid

        // 解析 IPFS CID 为 URL
        try {
          // 优先使用本地网关（桌面环境）
          const localGateway = import.meta.env.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080'
          if (typeof window !== 'undefined' && (window as any).__TAURI__) {
            return `${localGateway}/ipfs/${cid}`
          }
          // 浏览器环境使用公共网关
          return `https://gateway.ipfs.io/ipfs/${cid}`
        } catch (error) {
          console.warn('[AgentStore] IPFS URL 解析失败:', error)
          return `https://gateway.ipfs.io/ipfs/${cid}`
        }
      },

      // 设置 hydration 完成状态
      setHasHydrated: (state: boolean) => {
        set({ _hasHydrated: state })
      },

      // 添加智能体
      addAgent: (agentData: Partial<AgentMetadata>): AgentMetadata => {
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
                            `agent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
        
        // 处理头像URL
        const avatarUrl = agentData.avatar_cid ? get().resolveIpfsUrl(agentData.avatar_cid) : (agentData.avatar_url || agentData.avatar_cid || null)
        
        console.log('[AgentStore] 创建/更新智能体头像信息:', {
          name: agentData.name,
          hasAvatarCid: !!agentData.avatar_cid,
          avatar_cid: agentData.avatar_cid,
          resolvedAvatarUrl: avatarUrl?.substring(0, 100)
        })
        
        const newAgent: AgentMetadata = {
          id: generatedId,
          sessionId: agentData.sessionId,
          name: agentData.name || agentData.display_name || '未命名智能体',
          display_name: agentData.display_name || agentData.name || '未命名智能体',
          role_description: agentData.role_description || '',
          avatar_cid: agentData.avatar_cid || null,
          avatar_url: avatarUrl,
          mcp_config_cid: agentData.mcp_config_cid || null,
          mcp_ports: agentData.mcp_ports || [],
          agent_type: agentData.agent_type || 'ai_agent_sdk',
          ipns: ipnsValue,
          cid: agentData.cid || agentData.diapIdentity?.cid || null,
          did: agentData.did || agentData.diapIdentity?.did || null,
          diapIdentity: null,
          customPrompt: agentData.customPrompt || null,
          documents: agentData.documents || null,
          messages_cid: agentData.messages_cid || null,
          created_at: agentData.created_at || now,
          updated_at: now,
        }
        
        // 检查是否已存在（根据id或sessionId）
        const existingIndex = agents.findIndex(
          a => a.id === newAgent.id || a.sessionId === newAgent.sessionId
        )
        
        let updatedAgents: AgentMetadata[]
        if (existingIndex >= 0) {
          // 更新现有智能体 - 保护现有头像
          updatedAgents = [...agents]
          const existingAgent = updatedAgents[existingIndex]
          updatedAgents[existingIndex] = {
            ...existingAgent,
            ...newAgent,
            avatar_cid: newAgent.avatar_cid || existingAgent.avatar_cid,
            avatar_url: newAgent.avatar_url || existingAgent.avatar_url,
            created_at: existingAgent.created_at,
          }
        } else {
          // 添加新智能体
          updatedAgents = [newAgent, ...agents]
        }
        
        set({ agents: updatedAgents })
        return newAgent
      },

      // 更新智能体
      updateAgent: (idOrSessionId: string, updates: Partial<AgentMetadata>): AgentMetadata | null => {
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
          avatar_cid: updates.avatar_cid || existingAgent.avatar_cid,
          avatar_url: finalAvatarUrl,
          updated_at: Date.now(),
        }
        
        set({ agents: updatedAgents })
        console.log(`[AgentStore] 更新智能体: ${idOrSessionId}`)
        return updatedAgents[index]
      },

      // 删除智能体
      removeAgent: (idOrSessionId: string): AgentMetadata[] => {
        const agents = get().agents
        const filtered = agents.filter(
          a => a.id !== idOrSessionId && a.sessionId !== idOrSessionId
        )
        
        set({ agents: filtered })
        console.log(`[AgentStore] 删除智能体: ${idOrSessionId}`)
        return filtered
      },

      // 根据ID或sessionId获取智能体
      getAgent: (idOrSessionId: string): AgentMetadata | null => {
        const agents = get().agents
        return agents.find(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        ) || null
      },

      // 根据IPNS/CID/DID获取智能体
      getAgentByTarget: (target: string): AgentMetadata | null => {
        const agents = get().agents
        return agents.find(
          a => a.ipns === target || a.cid === target || a.did === target || a.id === target
        ) || null
      },

      // 更新智能体的单个文档，并重建 customPrompt
      updateAgentDocument: (idOrSessionId: string, docType: string, content: string): AgentMetadata | null => {
        const agents = get().agents
        const index = agents.findIndex(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        )

        if (index < 0) {
          console.warn(`[AgentStore] updateAgentDocument: 未找到智能体: ${idOrSessionId}`)
          return null
        }

        const updatedAgents = [...agents]
        const existingAgent = updatedAgents[index]

        // 合并新文档到 documents map
        const updatedDocs: Record<string, string> = {
          ...(existingAgent.documents || {}),
          [docType.toLowerCase()]: content,
        }

        // 重建 customPrompt（按照固定顺序拼接）
        const docOrder = ['soul', 'identity', 'capabilities', 'constraints', 'tools', 'memory', 'agents']
        const parts: string[] = []
        for (const key of docOrder) {
          if (updatedDocs[key]) {
            parts.push(`=== ${key.toUpperCase()} ===\n${updatedDocs[key]}`)
          }
        }
        const newCustomPrompt = parts.join('\n\n')

        updatedAgents[index] = {
          ...existingAgent,
          documents: updatedDocs,
          customPrompt: newCustomPrompt,
          updated_at: Date.now(),
        }

        set({ agents: updatedAgents })
        console.log(`[AgentStore] 智能体文档已更新: ${idOrSessionId}, 文档类型: ${docType}`)
        return updatedAgents[index]
      },

      // 清空所有智能体
      clearAgents: () => {
        set({ agents: [] })
        console.log('[AgentStore] 清空所有智能体')
      },

      // 从网络导入智能体
      importAgent: (agentData: Partial<AgentMetadata>): { agent: AgentMetadata; isNew: boolean } => {
        const agents = get().agents
        
        const targetId = agentData.id
        
        // 检查是否已存在（优先使用ID匹配，因为ID是最准确的）
        let existing: AgentMetadata | null = null
        if (targetId) {
          existing = agents.find(a => a.id === targetId) || null
        }
        
        // 如果通过ID没找到，尝试通过 IPNS/CID/DID 匹配
        if (!existing) {
          const target = agentData.ipns || agentData.cid || agentData.did
          if (target) {
            existing = agents.find(
              a => a.ipns === target || a.cid === target || a.did === target || a.id === target
            ) || null
          }
        }
        
        if (existing) {
          // 更新现有智能体（合并新数据）
          const updated = get().addAgent({
            ...existing,
            ...agentData,
            id: existing.id,
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
      partialize: (state: AgentStore) => ({ 
        agents: state.agents,
        _hasHydrated: state._hasHydrated 
      }),
      onRehydrateStorage: () => (state: AgentStore | undefined, error: Error | undefined) => {
        if (error) {
          console.error('[AgentStore] Hydration 失败:', error)
        } else if (state) {
          state.setHasHydrated(true)
        }
      },
      // 数据迁移：处理旧版本数据格式
      migrate: (persistedState: any, version: number): any => {
        if (version === 0) {
          // 验证数据完整性
          const agents: AgentMetadata[] = persistedState.agents || []
          persistedState.agents = agents.filter((agent: AgentMetadata) => {
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
    } as unknown as PersistOptions<AgentStore>
  )
)

// 等待 hydration 完成的 hook
export const useAgentStoreHydration = (): boolean => {
  return useAgentStore((state: AgentStore) => state._hasHydrated)
}

// 等待 hydration 完成的 Promise
export const waitForHydration = (): Promise<void> => {
  return new Promise((resolve) => {
    if (useAgentStore.getState()._hasHydrated) {
      resolve()
      return
    }
    
    const unsubscribe = useAgentStore.subscribe((state: AgentStore) => {
      if (state._hasHydrated) {
        unsubscribe()
        resolve()
      }
    })
  })
}

export default useAgentStore
