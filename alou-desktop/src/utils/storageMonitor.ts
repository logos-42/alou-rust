/**
 * storageMonitor - 本地存储监控工具
 * 用于监控和管理localStorage使用情况
 */

// 群聊数据接口
interface GroupChat {
  action_id: string
  description: string
  status: string
  created_at: number
  agents?: Array<{
    id: string
    name: string
    role?: string
  }>
  [key: string]: unknown
}

// 存储使用情况接口
export interface StorageUsage {
  totalSize: number
  itemCount: number
  estimatedQuota: number
  usagePercentage: number
  details: {
    groupChats?: number
    diapGroups?: number
    diapData?: number
    other?: number
  }
  isNearLimit: boolean
  isOverLimit: boolean
  formattedSize: string
  formattedQuota: string
}

// 清理选项接口
export interface CleanupOptions {
  keepRecent?: number
  keepRecentDiap?: number
  clearOlderThan?: number
  dryRun?: boolean
}

// 清理结果接口
export interface CleanupResult {
  cleaned: number
  errors: string[]
  details: Array<{
    key: string
    reason: string
    size: number
  }>
}

// 清理详情项接口
interface CleanupDetail {
  key: string
  reason: string
  size: number
}

// 优化后的群聊项接口
interface OptimizedChat {
  action_id: string
  description?: string
  status?: string
  created_at?: number
  agents: Array<{
    id: string
    name: string
    role?: string
    mode?: string
  }>
  metadata?: {
    type?: string
    channel_id?: string
    channel_name?: string
  }
}

// 建议接口
export interface Recommendation {
  level: 'critical' | 'warning' | 'info'
  title: string
  message: string
  action: string
}

/**
 * 获取存储使用情况
 */
export const getStorageUsage = (): StorageUsage | null => {
  if (typeof window === 'undefined') return null

  try {
    let totalSize = 0
    let itemCount = 0
    const details: StorageUsage['details'] = {}

    // 计算所有localStorage项目的大小
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key) {
        const value = localStorage.getItem(key)
        if (value) {
          const size = new Blob([key + value]).size
          totalSize += size
          itemCount++

          // 分类统计
          if (key.startsWith('cluster_actions_group_chats_')) {
            details.groupChats = (details.groupChats || 0) + size
          } else if (key.startsWith('diap_group_chat_')) {
            details.diapGroups = (details.diapGroups || 0) + size
          } else if (key.startsWith('diap_')) {
            details.diapData = (details.diapData || 0) + size
          } else {
            details.other = (details.other || 0) + size
          }
        }
      }
    }

    // 估算localStorage的可用空间（通常5-10MB）
    const estimatedQuota = 5 * 1024 * 1024 // 5MB
    const usagePercentage = Math.round((totalSize / estimatedQuota) * 100)

    return {
      totalSize,
      itemCount,
      estimatedQuota,
      usagePercentage,
      details,
      isNearLimit: usagePercentage > 80,
      isOverLimit: usagePercentage > 95,
      formattedSize: formatBytes(totalSize),
      formattedQuota: formatBytes(estimatedQuota)
    }
  } catch (error) {
    console.error('[storageMonitor] 获取存储使用情况失败:', error)
    return null
  }
}

/**
 * 格式化字节数为人类可读格式
 */
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'

  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

/**
 * 清理旧的存储数据
 * @param options 清理选项
 */
export const cleanupStorage = (options: CleanupOptions = {}): CleanupResult => {
  const {
    keepRecent = 3, // 保留最近N个频道的群聊
    keepRecentDiap = 2, // 保留最近N个DIAP群聊
    clearOlderThan = 7 * 24 * 60 * 60 * 1000, // 清理超过N天的数据
    dryRun = false // 是否只是模拟清理，不实际删除
  } = options

  if (typeof window === 'undefined') return { cleaned: 0, errors: [], details: [] }

  const results: CleanupResult = {
    cleaned: 0,
    errors: [],
    details: []
  }

  try {
    const now = Date.now()
    const keysToRemove: CleanupDetail[] = []

    // 分析所有localStorage项目
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (!key) continue

      const value = localStorage.getItem(key)
      if (!value) continue

      try {
        const parsed = JSON.parse(value)
        const itemAge = now - ((parsed as { timestamp?: number; created_at?: number }).timestamp || (parsed as { timestamp?: number; created_at?: number }).created_at || 0)

        let shouldRemove = false
        let reason = ''

        // 清理旧的群聊数据
        if (key.startsWith('cluster_actions_group_chats_')) {
          // const channelId = key.replace('cluster_actions_group_chats_', '') // 未使用变量
          const channelKeys = Array.from({ length: localStorage.length }, (_, idx) => localStorage.key(idx))
            .filter((k): k is string => k !== null && k.startsWith('cluster_actions_group_chats_'))
          
          if (channelKeys.length > keepRecent) {
            // 按时间排序，保留最新的
            const sortedKeys = channelKeys.sort((a, b) => {
              const aData = JSON.parse(localStorage.getItem(a) || '{}')
              const bData = JSON.parse(localStorage.getItem(b) || '{}')
              const aTime = aData?.[0]?.created_at || 0
              const bTime = bData?.[0]?.created_at || 0
              return bTime - aTime
            })
            
            const keysToKeep = sortedKeys.slice(0, keepRecent)
            if (!keysToKeep.includes(key)) {
              shouldRemove = true
              reason = '超出保留频道数量限制'
            }
          }

          if (itemAge > clearOlderThan) {
            shouldRemove = true
            reason = '数据过期'
          }
        }

        // 清理旧的DIAP群聊数据
        else if (key.startsWith('diap_group_chat_')) {
          const diapKeys = Array.from({ length: localStorage.length }, (_, idx) => localStorage.key(idx))
            .filter((k): k is string => k !== null && k.startsWith('diap_group_chat_'))
          
          if (diapKeys.length > keepRecentDiap) {
            const sortedKeys = diapKeys.sort((a, b) => {
              const aData = JSON.parse(localStorage.getItem(a) || '{}')
              const bData = JSON.parse(localStorage.getItem(b) || '{}')
              const aTime = aData?.createdAt || 0
              const bTime = bData?.createdAt || 0
              return bTime - aTime
            })
            
            const keysToKeep = sortedKeys.slice(0, keepRecentDiap)
            if (!keysToKeep.includes(key)) {
              shouldRemove = true
              reason = '超出保留DIAP群聊数量限制'
            }
          }

          if (itemAge > clearOlderThan) {
            shouldRemove = true
            reason = 'DIAP数据过期'
          }
        }

        // 清理其他旧数据
        else if (key.startsWith('diap_') && itemAge > clearOlderThan) {
          shouldRemove = true
          reason = 'DIAP相关数据过期'
        }

        if (shouldRemove) {
          keysToRemove.push({ key, reason, size: new Blob([key + value]).size })
        }
      } catch (parseError) {
        results.errors.push(`解析键 ${key} 失败: ${(parseError as Error).message}`)
      }
    }

    // 执行清理
    if (!dryRun) {
      keysToRemove.forEach(({ key }) => {
        try {
          localStorage.removeItem(key)
          results.cleaned++
        } catch (removeError) {
          results.errors.push(`删除键 ${key} 失败: ${(removeError as Error).message}`)
        }
      })
    }

    results.details = keysToRemove
    console.log(`[storageMonitor] ${dryRun ? '模拟' : '执行'}清理:`, results)

    return results
  } catch (error) {
    console.error('[storageMonitor] 清理存储失败:', error)
    results.errors.push(`清理失败: ${(error as Error).message}`)
    return results
  }
}

/**
 * 优化存储数据
 */
export const optimizeStorage = (): StorageUsage | null => {
  if (typeof window === 'undefined') return null

  try {
    console.log('[storageMonitor] 开始优化存储数据...')

    // 重新保存群聊数据（使用优化版本）
    const groupChatKeys: string[] = []
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key && key.startsWith('cluster_actions_group_chats_')) {
        groupChatKeys.push(key)
      }
    }

    let optimizedCount = 0
    groupChatKeys.forEach(key => {
      try {
        const value = localStorage.getItem(key)
        if (value) {
          const chats: GroupChat[] = JSON.parse(value)
          
          // 优化：只保留必要字段
          const optimizedChats: OptimizedChat[] = chats.map(chat => ({
            action_id: chat.action_id,
            description: chat.description,
            status: chat.status,
            created_at: chat.created_at,
            agents: chat.agents ? chat.agents.slice(0, 3).map(agent => ({
              id: agent.id,
              name: agent.name,
              role: agent.role
            })) : [],
            metadata: chat.metadata ? {
              type: (chat.metadata as any)?.type,
              channel_id: (chat.metadata as any)?.channel_id,
              channel_name: (chat.metadata as any)?.channel_name
            } : {}
          }))

          localStorage.setItem(key, JSON.stringify(optimizedChats))
          optimizedCount++
        }
      } catch (error) {
        console.error(`[storageMonitor] 优化键 ${key} 失败:`, error)
      }
    })

    console.log(`[storageMonitor] 优化完成，优化了 ${optimizedCount} 个群聊数据`)
    
    // 返回优化后的存储使用情况
    return getStorageUsage()
  } catch (error) {
    console.error('[storageMonitor] 优化存储失败:', error)
    return null
  }
}

/**
 * 监控存储状态并给出建议
 */
export const getStorageRecommendations = (): Recommendation[] => {
  const usage = getStorageUsage()
  if (!usage) return []

  const recommendations: Recommendation[] = []

  if (usage.usagePercentage > 90) {
    recommendations.push({
      level: 'critical',
      title: '存储空间严重不足',
      message: '建议立即清理旧数据，否则可能影响正常使用',
      action: 'cleanup'
    })
  } else if (usage.usagePercentage > 75) {
    recommendations.push({
      level: 'warning',
      title: '存储空间不足',
      message: '建议清理部分旧数据以释放空间',
      action: 'optimize'
    })
  } else if (usage.usagePercentage > 50) {
    recommendations.push({
      level: 'info',
      title: '存储使用正常',
      message: '可以定期清理旧数据以保持良好性能',
      action: 'monitor'
    })
  }

  // 分析数据分布
  if (usage.details) {
    const total = usage.totalSize
    const groupChatPercentage = (usage.details?.groupChats || 0) / total * 100
    const diapPercentage = (usage.details?.diapGroups || 0) / total * 100

    if (groupChatPercentage > 60) {
      recommendations.push({
        level: 'info',
        title: '群聊数据占用较多',
        message: `群聊数据占用了 ${Math.round(groupChatPercentage)}% 的存储空间`,
        action: 'cleanup_group_chats'
      })
    }

    if (diapPercentage > 40) {
      recommendations.push({
        level: 'info',
        title: 'DIAP数据占用较多',
        message: `DIAP数据占用了 ${Math.round(diapPercentage)}% 的存储空间`,
        action: 'cleanup_diap'
      })
    }
  }

  return recommendations
}

/**
 * 在控制台显示存储状态
 */
export const logStorageStatus = (): void => {
  const usage = getStorageUsage()
  const recommendations = getStorageRecommendations()

  console.group('📊 本地存储状态报告')
  
  if (usage) {
    console.log('📈 使用情况:', {
      总大小: usage.formattedSize,
      估算配额: usage.formattedQuota,
      使用率: `${usage.usagePercentage}%`,
      项目数量: usage.itemCount,
      接近限制: usage.isNearLimit,
      超出限制: usage.isOverLimit
    })

    if (usage.details) {
      console.log('📂 数据分布:', {
        群聊数据: usage.details.groupChats ? formatBytes(usage.details.groupChats) : '0',
        DIAP群聊: usage.details.diapGroups ? formatBytes(usage.details.diapGroups) : '0',
        DIAP其他: usage.details.diapData ? formatBytes(usage.details.diapData) : '0',
        其他数据: usage.details.other ? formatBytes(usage.details.other) : '0'
      })
    }
  }

  if (recommendations.length > 0) {
    console.log('💡 建议:')
    recommendations.forEach((rec, index) => {
      const icon = rec.level === 'critical' ? '🚨' : rec.level === 'warning' ? '⚠️' : 'ℹ️'
      console.log(`${index + 1}. ${icon} ${rec.title}: ${rec.message}`)
    })
  }

  console.groupEnd()
}

// 默认导出
export default {
  getStorageUsage,
  formatBytes,
  cleanupStorage,
  optimizeStorage,
  getStorageRecommendations,
  logStorageStatus
}
