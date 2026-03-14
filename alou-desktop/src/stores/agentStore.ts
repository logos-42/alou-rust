import { create } from 'zustand'
import { getAgentName } from '@/utils/agentNameUtils'
import { persist, createJSONStorage, PersistOptions } from 'zustand/middleware'
import * as sessionDb from '@/utils/sessionStorage'

/**
 * 智能体元数据接口
 *
 * 注意：为优化存储空间，以下字段不会被持久化：
 * - diapIdentity (可能包含大量协作数据)
 * - documents (文档内容应存储在 IPFS)
 * - mcp_ports (端口配置可能很大)
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

/**
 * 获取 session 隔离的存储 key
 * @param sessionId - Session ID（可选，不提供则使用默认 key）
 */
export function getAgentStoreKey(sessionId?: string): string {
  if (!sessionId) {
    return 'alou_agents_global'
  }
  return `alou_agents_${sessionId}`
}

/**
 * 获取当前 session ID
 * 从 URL hash 或 sessionStorage 中获取
 */
function getCurrentSessionId(): string | undefined {
  // 尝试从 URL hash 获取
  const hash = window.location.hash
  const match = hash.match(/session=([^&]+)/)
  if (match && match[1]) {
    return decodeURIComponent(match[1])
  }

  // 尝试从 sessionStorage 获取（比 localStorage 更适合临时 session）
  const stored = sessionStorage.getItem('alou_current_session')
  return stored || undefined
}

/**
 * 创建 IndexedDB 存储适配器（用于 zustand persist）
 */
function createIndexedDBStorage(sessionId?: string) {
  const storeKey = getAgentStoreKey(sessionId)

  return {
    getItem: async (name: string): Promise<string | null> => {
      try {
        const value = await sessionDb.get(name)
        return value ? JSON.stringify(value) : null
      } catch (error) {
        console.error('[AgentStore IndexedDB] getItem 失败:', error)
        return null
      }
    },

    setItem: async (name: string, value: string): Promise<void> => {
      try {
        const parsed = JSON.parse(value)
        await sessionDb.set(name, parsed, { sessionId })
      } catch (error) {
        console.error('[AgentStore IndexedDB] setItem 失败:', error)
        throw error
      }
    },

    removeItem: async (name: string): Promise<void> => {
      try {
        await sessionDb.deleteByKey(name)
      } catch (error) {
        console.error('[AgentStore IndexedDB] removeItem 失败:', error)
        throw error
      }
    },
  }
}

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

        // 重要：使用与 buildChannelFromAgent 相同的 ID 生成逻辑，确保一致性
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

        const generatedId = agentData.id ||
                            ipnsValue ||
                            agentData.did ||
                            agentData.cid ||
                            `agent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`

        // 处理头像 URL
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
          // 使用统一名称函数，确保名称一致性
name: getAgentName(agentData) || '未命名智能体',
          // 与 name 字段保持一致
display_name: getAgentName(agentData) || '未命名智能体',
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

        // 检查是否已存在（根据 id 或 sessionId）
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
          console.warn(`[AgentStore] 未找到智能体：${idOrSessionId}`)
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
          avatar_cid: updates.avatar_cid !== undefined ? updates.avatar_cid : existingAgent.avatar_cid,
          avatar_url: finalAvatarUrl,
          updated_at: Date.now(),
        }

        set({ agents: updatedAgents })
        return updatedAgents[index]
      },

      // 移除智能体
      removeAgent: (idOrSessionId: string): AgentMetadata[] => {
        const agents = get().agents
        const updatedAgents = agents.filter(
          a => a.id !== idOrSessionId && a.sessionId !== idOrSessionId
        )

        if (updatedAgents.length === agents.length) {
          console.warn(`[AgentStore] 未找到要移除的智能体：${idOrSessionId}`)
        } else {
          set({ agents: updatedAgents })
        }

        return updatedAgents
      },

      // 获取智能体
      getAgent: (idOrSessionId: string): AgentMetadata | null => {
        const agents = get().agents
        const agent = agents.find(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        )
        return agent || null
      },

      // 根据 target 获取智能体
      getAgentByTarget: (target: string): AgentMetadata | null => {
        const agents = get().agents
        const agent = agents.find(
          a => a.id === target ||
               a.sessionId === target ||
               a.ipns === target ||
               a.did === target ||
               a.cid === target
        )
        return agent || null
      },

      // 更新智能体文档
      updateAgentDocument: (idOrSessionId: string, docType: string, content: string): AgentMetadata | null => {
        const agents = get().agents
        const index = agents.findIndex(
          a => a.id === idOrSessionId || a.sessionId === idOrSessionId
        )

        if (index < 0) {
          console.warn(`[AgentStore] 未找到智能体：${idOrSessionId}`)
          return null
        }

        const updatedAgents = [...agents]
        const existingAgent = updatedAgents[index]

        const documents = existingAgent.documents || {}
        documents[docType] = content

        updatedAgents[index] = {
          ...existingAgent,
          documents,
          updated_at: Date.now(),
        }

        set({ agents: updatedAgents })
        return updatedAgents[index]
      },

      // 清空所有智能体
      clearAgents: () => {
        set({ agents: [] })
      },

      // 导入智能体
      importAgent: (agentData: Partial<AgentMetadata>): { agent: AgentMetadata; isNew: boolean } => {
        const agents = get().agents

        // 检查是否已存在
        const existingAgent = agents.find(
          a => a.id === agentData.id || a.sessionId === agentData.sessionId
        )

        if (existingAgent) {
          // 更新现有智能体
          const updatedAgents = [...agents]
          const index = updatedAgents.findIndex(
            a => a.id === existingAgent.id || a.sessionId === existingAgent.sessionId
          )

          if (index >= 0) {
            updatedAgents[index] = {
              ...existingAgent,
              ...agentData,
              updated_at: Date.now(),
            }
            set({ agents: updatedAgents })
            return { agent: updatedAgents[index], isNew: false }
          }
        }

        // 添加新智能体
        const now = Date.now()
        const newAgent: AgentMetadata = {
          ...agentData,
          id: agentData.id || `agent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
          sessionId: agentData.sessionId,
          // 使用统一名称函数，确保名称一致性
name: getAgentName(agentData) || '未命名智能体',
          // 与 name 字段保持一致
display_name: getAgentName(agentData) || '未命名智能体',
          role_description: agentData.role_description || '',
          avatar_cid: agentData.avatar_cid || null,
          avatar_url: agentData.avatar_url || null,
          mcp_config_cid: agentData.mcp_config_cid || null,
          mcp_ports: agentData.mcp_ports || [],
          agent_type: agentData.agent_type || 'ai_agent_sdk',
          ipns: agentData.ipns || null,
          cid: agentData.cid || null,
          did: agentData.did || null,
          diapIdentity: null,
          customPrompt: agentData.customPrompt || null,
          documents: agentData.documents || null,
          messages_cid: agentData.messages_cid || null,
          created_at: agentData.created_at || now,
          updated_at: now,
          imported_at: Date.now(),
          source: 'network',
        }

        set({ agents: [newAgent, ...agents] })
        return { agent: newAgent, isNew: true }
      },
    }),
    {
      name: 'alou_agents', // 使用固定名称，不依赖 session
      storage: createJSONStorage(() => createIndexedDBStorage(getCurrentSessionId())),
      partialize: (state: AgentStore) => ({
        agents: state.agents.filter(agent => {
          // 只显示当前 session 的智能体
          const currentSessionId = getCurrentSessionId()
          if (!currentSessionId) {
            // 没有 session 时，显示所有没有 sessionId 或 sessionId 为空的智能体
            return !agent.sessionId || agent.sessionId === ''
          }
          // 有 session 时，显示匹配的智能体
          return agent.sessionId === currentSessionId || !agent.sessionId
        }),
        _hasHydrated: state._hasHydrated
      }),
      onRehydrateStorage: () => (state: AgentStore | undefined, error: Error | undefined) => {
        if (error) {
          console.error('[AgentStore] Hydration 失败:', error)
        } else if (state) {
          // 重新解析所有头像 URL（确保跨系统兼容性）
          const updatedAgents = state.agents.map(agent => {
            // 如果有 avatar_cid 但 avatar_url 为空、无效或使用旧网关，重新解析
            if (agent.avatar_cid) {
              const needsReresolve = !agent.avatar_url ||
                agent.avatar_url.includes('undefined') ||
                agent.avatar_url.includes('null') ||
                agent.avatar_url.includes('gateway.ipfs.io') // 旧网关可能不可访问

              if (needsReresolve) {
                return {
                  ...agent,
                  avatar_url: state.resolveIpfsUrl(agent.avatar_cid),
                }
              }
            }
            return agent
          })

          state.agents = updatedAgents
          console.log(`[AgentStore] 头像 URL 重新解析完成，共 ${updatedAgents.length} 个智能体`)
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

/**
 * 设置当前 session ID
 * @param sessionId - Session ID
 */
export async function setCurrentSessionId(sessionId: string): Promise<void> {
  await sessionDb.set('alou_current_session', sessionId)
  // 触发自定义事件，通知其他组件 session 已更改
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('alou:session-changed', {
      detail: { sessionId }
    }))
  }
}

/**
 * 清除当前 session ID
 */
export async function clearCurrentSessionId(): Promise<void> {
  await sessionDb.deleteByKey('alou_current_session')
}

/**
 * 获取指定 session 的所有智能体
 * @param sessionId - Session ID
 * @returns 智能体数组
 */
export async function getAgentsForSession(sessionId: string): Promise<AgentMetadata[]> {
  const storeKey = getAgentStoreKey(sessionId)
  try {
    const stored = await sessionStorage.get(storeKey)
    if (!stored) return []
    
    return stored.agents || []
  } catch (error) {
    console.error('[AgentStore] 获取 session 智能体失败:', error)
    return []
  }
}

/**
 * 合并多个 session 的智能体到当前 session
 * @param sessionIds - Session ID 数组
 */
export async function mergeAgentsFromSessions(sessionIds: string[]): Promise<void> {
  const currentSessionId = getCurrentSessionId()
  if (!currentSessionId) return

  const currentStore = useAgentStore.getState()
  const existingIds = new Set(currentStore.agents.map(a => a.id))

  for (const sessionId of sessionIds) {
    if (sessionId === currentSessionId) continue

    const agents = await getAgentsForSession(sessionId)
    for (const agent of agents) {
      // 只添加不存在的智能体
      if (!existingIds.has(agent.id)) {
        currentStore.addAgent(agent)
        existingIds.add(agent.id)
      }
    }
  }
}

/**
 * 清除指定 session 的所有数据
 * @param sessionId - Session ID
 */
export async function clearSessionData(sessionId: string): Promise<void> {
  const storeKey = getAgentStoreKey(sessionId)
  try {
    await sessionStorage.deleteByKey(storeKey)
    console.log('[AgentStore] 已清除 session 数据:', sessionId)
  } catch (error) {
    console.error('[AgentStore] 清除 session 数据失败:', error)
    throw error
  }
}

/**
 * 获取存储使用统计
 */
export async function getStorageStats(): Promise<{
  totalAgents: number
  sessions: string[]
  storageUsed: number
}> {
  try {
    const stats = await sessionStorage.getStats()
    const currentStore = useAgentStore.getState()
    
    return {
      totalAgents: currentStore.agents.length,
      sessions: stats.sessions,
      storageUsed: stats.totalSize,
    }
  } catch (error) {
    console.error('[AgentStore] 获取存储统计失败:', error)
    return {
      totalAgents: 0,
      sessions: [],
      storageUsed: 0,
    }
  }
}
