import { useCallback, useMemo } from 'react'
import { useMultiAgentChat } from '@/hooks/useMultiAgentChat'

// DIAP身份类型
interface DiapIdentity {
  did: string;
  [key: string]: unknown;
}

// 频道类型
interface Channel {
  id: string;
  meta?: {
    did?: string;
    ipns?: string;
    name?: string;
    display_name?: string;
    role_description?: string;
    pubsub_topics?: string[];
  };
}

// 智能体信息类型
interface AgentInfo {
  id: string;
  did?: string;
  ipns?: string;
  name?: string;
  display_name?: string;
  role_description?: string;
  pubsub_topics?: string[];
}

// 消息类型
interface AgentMessage {
  id?: string;
  content?: string;
  timestamp?: number;
  from?: string;
}

// Hook参数类型
interface UseMultiAgentCoordinatorParams {
  selectedAgent: {
    diapIdentity?: DiapIdentity;
  } | null;
  sessionId: string | null;
  channels: Channel[];
  appendMessage: (message: unknown, agentId: string) => void;
}

// Hook返回类型
interface UseMultiAgentCoordinatorReturn {
  localIdentity: DiapIdentity | null;
  registeredAgentsForCoordinator: AgentInfo[];
  routeMessageToAgent: (message: string, targetAgentId: string) => Promise<boolean>;
  sendToAgent: (targetAgentId: string, message: string) => Promise<boolean>;
  analyzeIntent: (message: string) => Promise<{ targetAgentId: string | null; confidence: number }>;
  isCoordinatorReady: boolean;
}

/**
 * Hook for managing multi-agent coordination
 * 管理多智能体协调器
 */
export const useMultiAgentCoordinator = ({
  selectedAgent,
  sessionId,
  channels,
  appendMessage,
}: UseMultiAgentCoordinatorParams): UseMultiAgentCoordinatorReturn => {
  // 获取本地 DIAP 身份用于多智能体通信
  const localIdentity = useMemo<DiapIdentity | null>(() => {
    if (selectedAgent?.diapIdentity) {
      return selectedAgent.diapIdentity
    }
    // 尝试从 localStorage 获取
    if (sessionId && typeof window !== 'undefined') {
      const stored = localStorage.getItem(`diap_identity_${sessionId}`)
      if (stored) {
        try {
          return JSON.parse(stored) as DiapIdentity
        } catch {
          return null
        }
      }
    }
    return null
  }, [selectedAgent, sessionId])

  // 将所有频道中的智能体注册到协调器
  const registeredAgentsForCoordinator = useMemo<AgentInfo[]>(() => {
    return channels
      .filter(ch => ch.meta)
      .map(ch => ({
        id: ch.id,
        did: ch.meta?.did,
        ipns: ch.meta?.ipns,
        name: ch.meta?.display_name || ch.meta?.name,
        display_name: ch.meta?.display_name || ch.meta?.name,
        role_description: ch.meta?.role_description,
        pubsub_topics: ch.meta?.pubsub_topics || [],
      }))
  }, [channels])

  const multiAgentChat = useMultiAgentChat({
    localIdentity,
    registeredAgents: registeredAgentsForCoordinator,
    onAgentMessage: useCallback((agentId: string, message: AgentMessage) => {
      console.log('[useMultiAgentCoordinator] 收到智能体消息:', agentId, message)
      // 将智能体间消息添加到对应频道
      if (message.content) {
        appendMessage({
          id: message.id || `agent_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`,
          type: 'assistant',
          content: message.content,
          timestamp: message.timestamp || Date.now(),
          source: 'agent-coordinator',
          fromAgent: message.from,
        }, agentId)
      }
    }, [appendMessage]),
    onGroupMessage: useCallback((_groupId: string, message: AgentMessage) => {
      console.log('[useMultiAgentCoordinator] 收到群聊消息:', _groupId, message)
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

export default useMultiAgentCoordinator
