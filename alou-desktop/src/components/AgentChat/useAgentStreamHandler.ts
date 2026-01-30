import { useCallback, useEffect } from 'react'
import { useAgentStream } from '@/hooks/useAgentStream'

// 流事件类型
interface StreamEvent {
  event?: string;
  payload?: {
    tool_calls?: ToolCall[];
  };
  isFinal?: boolean;
  timestamp?: number;
  label?: string;
}

// 工具调用类型
interface ToolCall {
  name: string;
  result?: {
    resources?: unknown[];
    resource?: unknown;
    uri?: string;
  };
}

// Hook参数类型
interface UseAgentStreamHandlerParams {
  sessionId: string | null;
  isSessionReady: boolean;
  recordInteraction: (action: string, data: unknown, label?: string) => void;
  openUiResource: (resource: unknown, options: { source: string; toolCall: string }) => void;
  setConnectionStatus: (status: string) => void;
}

// Hook返回类型
interface UseAgentStreamHandlerReturn {
  streamStatus: string;
  streamEvents: StreamEvent[];
}

/**
 * Hook for handling agent stream events
 * 处理智能体流事件
 */
export const useAgentStreamHandler = ({
  sessionId,
  isSessionReady,
  recordInteraction,
  openUiResource,
  setConnectionStatus,
}: UseAgentStreamHandlerParams): UseAgentStreamHandlerReturn => {
  // 处理流事件
  const handleStreamEvent = useCallback(
    (event: StreamEvent) => {
      if (!event) return

      recordInteraction(
        'stream_event',
        { event: event.event, payload: event.payload },
        event.label,
      )

      if (event.isFinal && event.timestamp) {
        if (Array.isArray(event.payload?.tool_calls)) {
          const uiCall = event.payload.tool_calls.find((call: ToolCall) => {
            const result = call?.result
            if (!result) return false
            if (Array.isArray(result.resources) && result.resources.length > 0) return true
            return Boolean(result.resource || result.uri)
          })

          if (uiCall?.result) {
            const result = uiCall.result
            if (Array.isArray(result.resources) && result.resources.length > 0) {
              openUiResource(result.resources[0], { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.resource) {
              openUiResource(result.resource, { source: 'stream_final', toolCall: uiCall.name })
            } else if (result.uri) {
              openUiResource(result, { source: 'stream_final', toolCall: uiCall.name })
            }
          }
        }
      }
    },
    [openUiResource, recordInteraction],
  )

  const { status: streamStatus, events: streamEvents } = useAgentStream(sessionId, {
    enabled: isSessionReady,
    onEvent: handleStreamEvent,
  })

  // 同步 stream 状态到连接状态
  useEffect(() => {
    if (streamStatus === 'error') {
      setConnectionStatus('error')
    } else if (streamStatus === 'active' || streamStatus === 'completed') {
      setConnectionStatus('connected')
    } else if (streamStatus === 'polling') {
      setConnectionStatus('connecting')
    } else {
      setConnectionStatus('disconnected')
    }
  }, [streamStatus, setConnectionStatus])

  return {
    streamStatus,
    streamEvents,
  }
}

export default useAgentStreamHandler
