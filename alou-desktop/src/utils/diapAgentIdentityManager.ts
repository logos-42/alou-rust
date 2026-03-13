/**
 * DIAP 身份文件管理器
 * 基于 agent-id 的 DIAP 身份存储和加载
 * 存储路径: {app_data_dir}/alou-desktop/diap/agents/{agent-id}/diap.json
 */

import { invoke } from '@tauri-apps/api/core'
import type { DiapIdentity } from './diapIdentityManager'

// DIAP 身份文件格式
export interface DiapIdentityFile {
  version: string
  agent_id: string
  identity: DiapIdentity
  created_at: number
  updated_at: number
}

/**
 * 保存 DIAP 身份到文件 (基于 agent-id)
 * @param agentId - 智能体 ID
 * @param identity - DIAP 身份数据
 */
export async function saveDiapIdentityToFile(
  agentId: string,
  identity: DiapIdentity
): Promise<boolean> {
  try {
    console.log('[DiapAgentIdentity] 保存 DIAP 身份到文件, agentId:', agentId)
    
    // 序列化身份数据为 JSON 字符串
    const identityJson = JSON.stringify(identity)
    
    // 调用 Tauri 命令保存到文件
    console.log('[DiapAgentIdentity] 调用后端 set_diap_identity_for_agent, agentId:', agentId)
    await invoke('set_diap_identity_for_agent', {
      agentId,
      identity: identityJson,
    })
    
    console.log('[DiapAgentIdentity] DIAP 身份已保存:', agentId)
    return true
  } catch (error) {
    console.error('[DiapAgentIdentity] 保存 DIAP 身份失败:', error)
    return false
  }
}

/**
 * 从文件加载 DIAP 身份 (基于 agent-id)
 * @param agentId - 智能体 ID
 * @returns DIAP 身份数据 or null
 */
export async function loadDiapIdentityFromFile(
  agentId: string
): Promise<DiapIdentity | null> {
  try {
    console.log('[DiapAgentIdentity] 从文件加载 DIAP 身份:', agentId)
    
    // 调用 Tauri 命令从文件读取
    console.log('[DiapAgentIdentity] 调用后端 get_diap_identity_for_agent, agentId:', agentId)
    const identityJson = await invoke<string | null>('get_diap_identity_for_agent', {
      agentId,
    })
    
    if (!identityJson) {
      console.log('[DiapAgentIdentity] 未找到 DIAP 身份:', agentId)
      return null
    }
    
    const identity = JSON.parse(identityJson) as DiapIdentity
    console.log('[DiapAgentIdentity] DIAP 身份已加载:', agentId, identity.did)
    return identity
  } catch (error) {
    console.error('[DiapAgentIdentity] 加载 DIAP 身份失败:', error)
    return null
  }
}

/**
 * 删除 DIAP 身份文件
 * @param agentId - 智能体 ID
 */
export async function removeDiapIdentityFromFile(
  agentId: string
): Promise<boolean> {
  try {
    console.log('[DiapAgentIdentity] 删除 DIAP 身份文件:', agentId)
    
    await invoke('remove_diap_identity_for_agent', {
      agentId,
      removeFromIpfs: false,
    })
    
    console.log('[DiapAgentIdentity] DIAP 身份已删除:', agentId)
    return true
  } catch (error) {
    console.error('[DiapAgentIdentity] 删除 DIAP 身份失败:', error)
    return false
  }
}

/**
 * 获取所有智能体的 DIAP 身份
 * @returns 所有 DIAP 身份的映射
 */
export async function getAllDiapIdentitiesFromFile(): Promise<
  Record<string, DiapIdentity>
> {
  try {
    console.log('[DiapAgentIdentity] 获取所有 DIAP 身份')
    
    const identitiesMap = await invoke<Record<string, string>>(
      'get_all_diap_identities_for_agent'
    )
    
    const result: Record<string, DiapIdentity> = {}
    for (const [agentId, identityJson] of Object.entries(identitiesMap)) {
      try {
        result[agentId] = JSON.parse(identityJson)
      } catch (parseError) {
        console.error('[DiapAgentIdentity] 解析身份失败:', agentId, parseError)
      }
    }
    
    console.log('[DiapAgentIdentity] 找到', Object.keys(result).length, '个 DIAP 身份')
    return result
  } catch (error) {
    console.error('[DiapAgentIdentity] 获取所有 DIAP 身份失败:', error)
    return {}
  }
}

/**
 * 检查智能体是否有 DIAP 身份文件
 * @param agentId - 智能体 ID
 */
export async function hasDiapIdentityFile(agentId: string): Promise<boolean> {
  try {
    const identity = await loadDiapIdentityFromFile(agentId)
    return identity !== null
  } catch {
    return false
  }
}

// 全局窗口扩展
declare global {
  interface Window {
    AlouDiapAgentIdentity: {
      save: (agentId: string, identity: DiapIdentity) => Promise<boolean>
      load: (agentId: string) => Promise<DiapIdentity | null>
      remove: (agentId: string) => Promise<boolean>
      getAll: () => Promise<Record<string, DiapIdentity>>
      has: (agentId: string) => Promise<boolean>
    }
  }
}

// 导出单例
export const diapAgentIdentityManager = {
  save: saveDiapIdentityToFile,
  load: loadDiapIdentityFromFile,
  remove: removeDiapIdentityFromFile,
  getAll: getAllDiapIdentitiesFromFile,
  has: hasDiapIdentityFile,
}

// 注册到全局对象
if (typeof window !== 'undefined') {
  window.AlouDiapAgentIdentity = diapAgentIdentityManager
}

export default diapAgentIdentityManager
