/**
 * useAgentMessages - 模块化重构版本
 * 
 * @module components/AgentChat/useAgentMessages
 * 
 * 重构完成：
 * - ✅ 将 1481 行大文件拆分为多个小模块（每个 < 300 行）
 * - ✅ 职责清晰，易于测试和维护
 * - ✅ 支持独立复用各个 hooks 和服务
 * 
 * 模块结构：
 * ```
 * useAgentMessages/
 * ├── index.ts                      # 主入口
 * ├── types.ts                      # 类型定义
 * ├── utils/                        # 工具函数
 * │   ├── progressUtils.ts          # 进度事件转换
 * │   └── messageUtils.ts           # 消息创建工具
 * ├── hooks/                        # React Hooks
 * │   ├── useMessageState.ts        # 消息状态管理
 * │   ├── useMessagePersistence.ts  # IPFS 持久化
 * │   ├── useAgentExecution.ts      # 智能体执行
 * │   └── useGroupChatMessages.ts   # 群聊消息
 * └── services/                     # 服务层
 *     ├── messageQueueService.ts    # 消息队列
 *     └── agentCommunicationService.ts # 智能体通信
 * ```
 */

// 类型导出
export * from './types'

// 工具函数导出
export * from './utils'

// Hooks 导出
export * from './hooks'

// 服务导出
export * from './services'

// 重新导出 legacy 版本（向后兼容）
export { useAgentMessages } from './useAgentMessages.legacy'
