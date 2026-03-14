/**
 * 消息持久化 Hook - IPFS 存储
 * 
 * @module components/AgentChat/useAgentMessages/hooks/useMessagePersistence
 */

import { useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Message } from '../types'

interface UseMessagePersistenceConfig {
  appendMessage: (message: Message, channelId?: string) => void
  setAgentLoading: (agentId: string, loading: boolean) => void
}

interface UseMessagePersistenceReturn {
  saveMessagesToIpfs: (channelId: string, agentId: string, force?: boolean) => Promise<string | null>
  loadMessagesFromIpfs: (channelId: string, messagesCid: string) => Promise<boolean>
  updateIpfsDocument: (agentId: string, sessionInfo: unknown) => Promise<void>
}

/**
 * 消息 IPFS 持久化 Hook
 */
export function useMessagePersistence({
  appendMessage,
  setAgentLoading,
}: UseMessagePersistenceConfig): UseMessagePersistenceReturn {
  const savedMessageCountRef = useRef<Record<string, number>>({})

  /**
   * 保存消息到 IPFS
   */
  const saveMessagesToIpfs = useCallback(async (
    channelId: string,
    agentId: string,
    force = false
  ): Promise<string | null> => {
    try {
      // 获取当前频道的消息
      const messagesKey = `messages_${channelId}`
      const messagesStr = localStorage.getItem(messagesKey)
      
      if (!messagesStr) {
        console.warn('[saveMessagesToIpfs] 没有找到本地消息')
        return null
      }

      const messages: Message[] = JSON.parse(messagesStr)
      
      // 如果没有新消息，跳过保存
      const savedCount = savedMessageCountRef.current[channelId] || 0
      if (!force && messages.length <= savedCount) {
        return null
      }

      // 调用 IPFS 保存
      const cid = await invoke<string>('save_messages_to_ipfs', {
        messages: messages.map(m => ({
          id: m.id,
          type: m.type,
          content: m.content,
          timestamp: m.timestamp,
          source: m.source,
          agentId: m.agentId,
          metadata: m.metadata,
        })),
        agentId,
      })

      savedMessageCountRef.current[channelId] = messages.length
      console.log('[saveMessagesToIpfs] 消息已保存到 IPFS，CID:', cid)
      
      return cid
    } catch (error) {
      console.warn('[saveMessagesToIpfs] 保存消息失败:', error)
      return null
    }
  }, [appendMessage, setAgentLoading])

  /**
   * 从 IPFS 加载消息
   */
  const loadMessagesFromIpfs = useCallback(async (
    channelId: string,
    messagesCid: string
  ): Promise<boolean> => {
    try {
      if (!messagesCid) {
        console.log('[loadMessagesFromIpfs] CID 为空，跳过加载')
        return false
      }

      // 调用 IPFS 加载
      const result = await invoke<{ messages: unknown[] }>('load_messages_from_ipfs', {
        cid: messagesCid,
      })

      if (result && result.messages && result.messages.length > 0) {
        // 恢复消息到本地存储
        const messagesKey = `messages_${channelId}`
        localStorage.setItem(messagesKey, JSON.stringify(result.messages))
        
        // 更新已保存计数
        savedMessageCountRef.current[channelId] = result.messages.length
        
        console.log('[loadMessagesFromIpfs] 成功加载', result.messages.length, '条消息')
        return true
      }

      return false
    } catch (error) {
      console.error('[loadMessagesFromIpfs] 加载消息失败:', error)
      return false
    }
  }, [appendMessage])

  /**
   * 更新 IPFS 文档
   */
  const updateIpfsDocument = useCallback(async (
    agentId: string,
    sessionInfo: unknown
  ): Promise<void> => {
    try {
      await invoke('update_ipfs_document', {
        agentId,
        sessionInfo,
      })
    } catch (error) {
      console.warn('[updateIpfsDocument] 更新文档失败:', error)
    }
  }, [])

  return {
    saveMessagesToIpfs,
    loadMessagesFromIpfs,
    updateIpfsDocument,
  }
}
