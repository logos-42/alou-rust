import { useCallback, useMemo } from 'react'
import { useMultiAgentChat } from '@/hooks/useMultiAgentChat'

/**
 * Hook for managing multi-agent coordination
 * 管理多智能体协调器
 */
export const useMultiAgentCoordinator = ({
  selectedAgent,
  sessionId,
  channels,
  appendMessage,
}) => {
  // 获取本地 DIAP 身份用于多智能体通信
  const localIdentity = useMemo(() => {
    if (selectedAgent?.diapIdentity) {
      return selectedAgent.diapIdentity
    }
    // 尝试从 localStorage 获取
    if (sessionId && typeof window !== 'undefined') {
      const stored = localStorage.getItem(`diap_identity_${sessionId}`)
      if (stored) {
        try {
          return JSON.parse(stored)
        } catch {
          return null
        }
      }
    }
    return null
  }, [selectedAgent, sessionId])

  // 将所有频道中的智能体注册到协调器
  const registeredAgentsForCoordinator = useMemo(() => {
    return channels
      .filter(ch => ch.meta)
      .map(ch => ({
        id: ch.id,
        did: ch.meta.did,
        ipns: ch.meta.ipns,
        name: ch.meta.display_name || ch.meta.name,
        display_name: ch.meta.display_name || ch.meta.name,
        role_description: ch.meta.role_description,
        pubsub_topics: ch.meta.pubsub_topics || [],
      }))
  }, [channels])

  const multiAgentChat = useMultiAgentChat({
    localIdentity,
    registeredAgents: registeredAgentsForCoordinator,
    onAgentMessage: useCallback((agentId, message) => {
      console.log('[useMultiAgentCoordinator] 收到智能体消息:', agentId, message)
      // 将智能体间消息添加到对应频道
      if (message.content) {
        appendMessage({
          id: message.id || `agent_${Date.now()}`,
          type: 'assistant',
          content: message.content,
          timestamp: message.timestamp || Date.now(),
          source: 'agent-coordinator',
          fromAgent: message.from,
        }, agentId)
      }
    }, [appendMessage]),
    onGroupMessage: useCallback((groupId, message) => {
      console.log('[useMultiAgentCoordinator] 收到群聊消息:', groupId, message)
    }, []),
  })

  return {
    localIdentity,
    registeredAgentsForCoordinator,
    routeMessageToAgent: multiAgentChat.routeMessageToAgent,
    sendToAgent: multiAgentChat.sendToAgent,
    analyzeIntent: multiAgentChat.analyzeIntent,
    isCoordinatorReady: multiAgentChat.isCoordinatorReady,
  }
}

