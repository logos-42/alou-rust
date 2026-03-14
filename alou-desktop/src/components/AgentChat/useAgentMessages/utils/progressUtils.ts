/**
 * 进度事件工具函数
 * 
 * @module components/AgentChat/useAgentMessages/utils/progressUtils
 */

import { AgentProgressPayload } from '../types'

/** 将进度事件转换为可读的中文消息 */
export function progressToText(e: AgentProgressPayload): string | null {
  switch (e.type) {
    case 'started':
      return '🚀 开始执行任务...'
    case 'thinking':
      return e.content ? `💭 ${e.content}` : null
    case 'tools_pending':
      return `🔧 准备调用 ${e.count} 个工具...`
    case 'tool_calling':
      return `⚙️ 调用工具：**${e.tool_name}**`
    case 'tool_done':
      if (e.success) {
        const preview = e.preview ? `\n\`\`\`\n${e.preview.slice(0, 200)}\n\`\`\`` : ''
        return `✅ 工具 **${e.tool_name}** 完成${preview}`
      } else {
        return `❌ 工具 **${e.tool_name}** 失败：${e.error || '未知错误'}`
      }
    case 'failed':
      return `❌ 任务失败：${e.error || '未知错误'}`
    default:
      return null
  }
}
