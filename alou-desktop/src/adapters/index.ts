/**
 * 统一群聊适配器模块入口
 * 
 * 提供统一的群聊操作接口，屏蔽底层实现差异
 * 支持 Memory、PubSub、Iroh 三种模式
 * 
 * @module adapters/index
 * 
 * @example
 * // 基本使用
 * import {
 *   groupChatAdapterFactory,
 *   getBestAvailableAdapter,
 *   createUnifiedGroup,
 *   sendUnifiedMessage,
 * } from '@/adapters'
 * 
 * // 创建群聊
 * const group = await createUnifiedGroup({
 *   name: '我的群聊',
 *   description: '测试群聊',
 *   mode: 'auto' // 自动选择最佳模式
 * })
 * 
 * // 发送消息
 * await sendUnifiedMessage(group, 'Hello, World!')
 */

// 工厂和便捷函数
export {
  groupChatAdapterFactory,
  DefaultGroupChatAdapterFactory,
  getAdapterForGroup,
  getAdapterByMode,
  getBestAvailableAdapter,
  detectBestGroupChatMode,
  createUnifiedGroup,
  sendUnifiedMessage,
} from './groupChatAdapter'

// 适配器实现
export { MemoryGroupChatAdapter } from './memoryAdapter'
export { PubSubGroupChatAdapter } from './pubsubAdapter'
export { IrohGroupChatAdapter } from './irohAdapter'

// 重新导出类型
export type {
  GroupChatAdapter,
  GroupChatAdapterFactory,
  GroupChatConfig,
  GroupChatMode,
  MessageHandler,
  UnifiedGroup,
  UnifiedMessage,
  UnsubscribeFunction,
} from '../types/groupchat'
