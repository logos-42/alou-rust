import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { requestMcpUiResource } from '@/services/mcpUiService'
import { ACTION_LABELS } from '@/hooks/useAgentChat'

/**
 * Hook for managing UI state and interactions
 */
export const useAgentUI = (options: {
  sessionId?: string | null;
  viewportWidth?: number;
  isLeftSidebarCollapsed?: boolean;
  isSidebarCollapsed?: boolean;
} = {}) => {
  const {
    sessionId,
    viewportWidth: initialViewportWidth = 1440,
    isLeftSidebarCollapsed,
    isSidebarCollapsed,
  } = options;

  const [isDarkMode, setIsDarkMode] = useState(false)
  const [isSidebarCollapsedLocal, setSidebarCollapsed] = useState(true)
  const [isLeftSidebarCollapsedLocal, setLeftSidebarCollapsed] = useState(false)
  const [leftSidebarWidth, setLeftSidebarWidth] = useState(300) // 默认宽度
  const [isInteractionCollapsed, setInteractionCollapsed] = useState(true)
  const [isConversationVisible, setConversationVisible] = useState(false)
  const [viewportWidth, setViewportWidth] = useState(initialViewportWidth)
  const [interactionLogs, setInteractionLogs] = useState([])
  const [uiResource, setUiResource] = useState(null)
  const [isUiModalOpen, setUiModalOpen] = useState(false)
  const [showDiapPanel, setShowDiapPanel] = useState(false)

  const finalEventHandledRef = useRef(null)
  const contextEventsRef = useRef([])

  // Use external collapse states if provided, otherwise use local
  const effectiveSidebarCollapsed = isSidebarCollapsed ?? isSidebarCollapsedLocal
  const effectiveLeftSidebarCollapsed = isLeftSidebarCollapsed ?? isLeftSidebarCollapsedLocal

  const recordInteraction = useCallback((action, detail, label = undefined) => {
    const timestamp = Date.now()
    const entry = {
      id: `log_${timestamp}_${Math.random().toString(36).slice(2, 6)}`,
      action,
      label: label || ACTION_LABELS[action] || action,
      timestamp,
      detail,
    }

    setInteractionLogs((prev) => {
      const next = [entry, ...prev]
      return next.slice(0, 20)
    })

    contextEventsRef.current.push({ action, detail, timestamp })
    if (contextEventsRef.current.length > 50) {
      contextEventsRef.current.splice(0, contextEventsRef.current.length - 50)
    }

    if (typeof window !== 'undefined') {
      window.dispatchEvent(
        new CustomEvent('agent-context-event', { detail: { action, detail, timestamp } }),
      )
    }
  }, [])

  const openUiResource = useCallback(
    (payload, meta = {}) => {
      if (!payload || !payload.resource) return
      const embedded =
        payload.resource && payload.resource.uri
          ? { resource: payload.resource, metadata: payload.metadata }
          : payload
      setUiResource(embedded)
      setUiModalOpen(true)
      recordInteraction('trigger_mcp', {
        kind: 'ui_resource_open',
        uri: embedded.resource?.uri,
        metadata: payload.metadata,
        ...meta,
      })
    },
    [recordInteraction],
  )

  const fetchAndOpenUiResource = useCallback(
    async (target, params = {}, meta = {}) => {
      if (!target) {
        return
      }
      try {
        const payload = {
          session_id: sessionId,
          ...params,
        }
        const { resource, metadata } = await requestMcpUiResource(target, payload)
        if (!resource) {
          // 资源为空时静默处理，不抛出错误
          console.warn(`[MCP UI] 资源为空 (${target})，跳过UI打开`)
          return
        }
        openUiResource(
          { resource, metadata },
          {
            source: 'frontend',
            target,
            ...meta,
          },
        )
      } catch (error) {
        // 静默处理MCP UI资源加载错误
        console.warn(`[MCP UI] 资源加载失败 (${target}):`, error.message)
        // 不再记录交互日志，避免错误信息干扰
      }
    },
    [openUiResource, sessionId],
  )

  const closeUiResource = useCallback(() => {
    setUiModalOpen(false)
    setUiResource(null)
  }, [])

  const handleUiAction = useCallback(
    async (action) => {
      console.debug('MCP UI action', action)
      recordInteraction('trigger_mcp', {
        kind: 'ui_action',
        action,
      })
      if (action?.type === 'close') {
        closeUiResource()
      }
      return { type: 'success' }
    },
    [closeUiResource, recordInteraction],
  )

  const toggleSidebar = useCallback(() => {
    setSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const toggleInteractionPanel = useCallback(() => {
    setInteractionCollapsed((prev) => !prev)
  }, [])

  const toggleDarkMode = useCallback(() => {
    setIsDarkMode((prev) => {
      const next = !prev
      recordInteraction('toggle_theme', { theme: next ? 'dark' : 'light' })
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('alou-theme', next ? 'dark' : 'light')
      }
      return next
    })
  }, [recordInteraction])

  const toggleLeftSidebar = useCallback(() => {
    setLeftSidebarCollapsed((prev) => {
      const next = !prev
      recordInteraction('toggle_left_sidebar', { collapsed: next })
      return next
    })
  }, [recordInteraction])

  const handleLeftSidebarWidthChange = useCallback((newWidth: number) => {
    setLeftSidebarWidth(newWidth)
  }, [])

  const openConversationPanel = useCallback(() => {
    setConversationVisible(true)
  }, [])

  const closeConversationPanel = useCallback(() => {
    setConversationVisible(false)
  }, [])

  const handleResize = useCallback(() => {
    if (typeof window === 'undefined') return
    setViewportWidth(window.innerWidth)
  }, [])

  // Console dock style calculation
  const consoleDockStyle = useMemo(() => {
    if (viewportWidth <= 1024) {
      return { margin: '0 1rem 0 1rem' }
    }
    const leftWidth = effectiveLeftSidebarCollapsed || leftSidebarWidth <= 100 ? 84 : leftSidebarWidth
    const rightWidth = effectiveSidebarCollapsed ? 80 : 340
    return {
      marginLeft: `${leftWidth + 24}px`,
      marginRight: `${rightWidth + 24}px`,
    }
  }, [viewportWidth, effectiveSidebarCollapsed, effectiveLeftSidebarCollapsed, leftSidebarWidth])

  // Initialize dark mode from localStorage or system preference
  useEffect(() => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('alou-theme')
      if (savedTheme) {
        const timer = setTimeout(() => {
          setIsDarkMode(savedTheme === 'dark')
        }, 0)
        return () => clearTimeout(timer)
      } else if (typeof window !== 'undefined') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
        const timer = setTimeout(() => {
          setIsDarkMode(prefersDark)
        }, 0)
        return () => clearTimeout(timer)
      }
    }
  }, [])

  return {
    isDarkMode,
    setIsDarkMode,
    isSidebarCollapsed: effectiveSidebarCollapsed,
    setSidebarCollapsed,
    isLeftSidebarCollapsed: effectiveLeftSidebarCollapsed,
    setLeftSidebarCollapsed,
    leftSidebarWidth,
    setLeftSidebarWidth,
    isInteractionCollapsed,
    setInteractionCollapsed,
    isConversationVisible,
    setConversationVisible,
    viewportWidth,
    setViewportWidth,
    interactionLogs,
    setInteractionLogs,
    uiResource,
    setUiResource,
    isUiModalOpen,
    setUiModalOpen,
    showDiapPanel,
    setShowDiapPanel,
    finalEventHandledRef,
    contextEventsRef,
    recordInteraction,
    openUiResource,
    fetchAndOpenUiResource,
    closeUiResource,
    handleUiAction,
    toggleSidebar,
    toggleInteractionPanel,
    toggleDarkMode,
    toggleLeftSidebar,
    handleLeftSidebarWidthChange,
    openConversationPanel,
    closeConversationPanel,
    handleResize,
    consoleDockStyle,
  }
}
