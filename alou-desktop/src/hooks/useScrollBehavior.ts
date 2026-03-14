import { useCallback, useEffect, useRef, useState } from 'react'

/**
 * 智能滚动行为 Hook
 * 自动滚动到底部，但当用户向上滚动查看历史消息时暂停
 */
interface UseScrollBehaviorOptions {
  autoScroll?: boolean
  threshold?: number
  scrollDelay?: number
}

interface UseScrollBehaviorReturn {
  containerRef: React.RefObject<HTMLDivElement>
  isAtBottom: boolean
  isUserScrolling: boolean
  hasNewMessages: boolean
  scrollToBottom: (behavior?: ScrollBehavior) => void
  scrollToBottomManual: () => void
  scrollToElement: (element: Element, behavior?: ScrollBehavior) => void
  handleScroll: () => void
  handleNewMessage: () => void
  pauseAutoScroll: () => void
  resumeAutoScroll: () => void
  checkIsAtBottom: () => boolean
}

export const useScrollBehavior = (
  options: UseScrollBehaviorOptions = {}
): UseScrollBehaviorReturn => {
  const {
    autoScroll = true,
    threshold = 100,
    scrollDelay = 100,
  } = options

  const containerRef = useRef<HTMLDivElement>(null)
  const [isAtBottom, setIsAtBottom] = useState<boolean>(true)
  const [isUserScrolling, setIsUserScrolling] = useState<boolean>(false)
  const [hasNewMessages, setHasNewMessages] = useState<boolean>(false)
  const scrollTimeoutRef = useRef<number | null>(null)
  const userScrollTimeoutRef = useRef<number | null>(null)

  /**
   * 检查是否在底部
   */
  const checkIsAtBottom = useCallback((): boolean => {
    const container = containerRef.current
    if (!container) return true

    const { scrollTop, scrollHeight, clientHeight } = container
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight
    return distanceFromBottom <= threshold
  }, [threshold])

  /**
   * 滚动到底部
   */
  const scrollToBottom = useCallback((behavior: ScrollBehavior = 'smooth') => {
    const container = containerRef.current
    if (!container) return

    container.scrollTo({
      top: container.scrollHeight,
      behavior,
    })
  }, [])

  /**
   * 滚动到指定元素
   */
  const scrollToElement = useCallback((element: Element, behavior: ScrollBehavior = 'smooth') => {
    if (!element || !containerRef.current) return

    element.scrollIntoView({
      behavior,
      block: 'end',
    })
  }, [])

  /**
   * 处理滚动事件
   */
  const handleScroll = useCallback(() => {
    const container = containerRef.current
    if (!container) return

    // 清除之前的超时
    if (userScrollTimeoutRef.current) {
      clearTimeout(userScrollTimeoutRef.current)
    }

    // 标记为用户正在滚动
    setIsUserScrolling(true)

    // 检查是否在底部
    const atBottom = checkIsAtBottom()
    setIsAtBottom(atBottom)

    // 如果在底部，清除新消息提示
    if (atBottom) {
      setHasNewMessages(false)
    }

    // 用户停止滚动后一段时间，重置 isUserScrolling
    userScrollTimeoutRef.current = window.setTimeout(() => {
      setIsUserScrolling(false)
    }, 150)
  }, [checkIsAtBottom])

  /**
   * 处理新消息
   */
  const handleNewMessage = useCallback(() => {
    if (!autoScroll) return

    // 如果用户在底部，自动滚动
    if (isAtBottom && !isUserScrolling) {
      // 使用延迟确保 DOM 已更新
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current)
      }

      scrollTimeoutRef.current = window.setTimeout(() => {
        scrollToBottom('smooth')
      }, scrollDelay)
    } else {
      // 用户不在底部，显示新消息提示
      setHasNewMessages(true)
    }
  }, [autoScroll, isAtBottom, isUserScrolling, scrollToBottom, scrollDelay])

  /**
   * 手动滚动到底部
   */
  const scrollToBottomManual = useCallback(() => {
    setIsAtBottom(true)
    setHasNewMessages(false)
    scrollToBottom('smooth')
  }, [scrollToBottom])

  /**
   * 暂停自动滚动
   */
  const pauseAutoScroll = useCallback(() => {
    setIsUserScrolling(true)
  }, [])

  /**
   * 恢复自动滚动
   */
  const resumeAutoScroll = useCallback(() => {
    setIsUserScrolling(false)
    setIsAtBottom(true)
    scrollToBottom('smooth')
  }, [scrollToBottom])

  // 清理超时
  useEffect(() => {
    return () => {
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current)
      }
      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current)
      }
    }
  }, [])

  return {
    containerRef,
    isAtBottom,
    isUserScrolling,
    hasNewMessages,
    scrollToBottom,
    scrollToBottomManual,
    scrollToElement,
    handleScroll,
    handleNewMessage,
    pauseAutoScroll,
    resumeAutoScroll,
    checkIsAtBottom,
  }
}

/**
 * 无限滚动 Hook
 * 用于消息历史加载
 */
interface UseInfiniteScrollOptions {
  onLoadMore?: () => Promise<void>
  hasMore?: boolean
  threshold?: number
  loading?: boolean
}

interface UseInfiniteScrollReturn {
  containerRef: React.RefObject<HTMLDivElement>
  isLoadingMore: boolean
  handleScroll: () => void
}

export const useInfiniteScroll = (
  options: UseInfiniteScrollOptions = {}
): UseInfiniteScrollReturn => {
  const {
    onLoadMore,
    hasMore = true,
    threshold = 100,
    loading = false,
  } = options

  const containerRef = useRef<HTMLDivElement>(null)
  const [isLoadingMore, setIsLoadingMore] = useState<boolean>(false)

  const handleScroll = useCallback(async () => {
    const container = containerRef.current
    if (!container || isLoadingMore || loading || !hasMore || !onLoadMore) return

    const { scrollTop } = container

    // 当滚动到顶部附近时加载更多
    if (scrollTop < threshold) {
      setIsLoadingMore(true)

      // 记录当前滚动位置
      const oldScrollHeight = container.scrollHeight

      try {
        await onLoadMore()

        // 加载完成后，保持滚动位置
        requestAnimationFrame(() => {
          const newScrollHeight = container.scrollHeight
          container.scrollTop = newScrollHeight - oldScrollHeight
        })
      } finally {
        setIsLoadingMore(false)
      }
    }
  }, [onLoadMore, hasMore, threshold, isLoadingMore, loading])

  return {
    containerRef,
    isLoadingMore,
    handleScroll,
  }
}

export default useScrollBehavior
