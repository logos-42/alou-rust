import { useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'

/**
 * 限额弹窗状态类型
 */
interface RateLimitModalState {
  isOpen: boolean;
  remainingRequests: number;
  resetTime: Date | null;
}

/**
 * 打开弹窗参数类型
 */
interface OpenRateLimitModalParams {
  remainingRequests?: number;
  resetTime?: Date | null;
}

/**
 * Hook返回类型
 */
interface UseRateLimitModalReturn {
  rateLimitModal: RateLimitModalState;
  openRateLimitModal: (params: OpenRateLimitModalParams) => void;
  closeRateLimitModal: () => void;
  handleSubscribe: () => void;
}

/**
 * Hook for managing rate limit modal state
 * 管理限额弹窗的状态
 */
export const useRateLimitModal = (): UseRateLimitModalReturn => {
  const navigate = useNavigate()
  const [rateLimitModal, setRateLimitModal] = useState<RateLimitModalState>({
    isOpen: false,
    remainingRequests: 0,
    resetTime: null,
  })

  const openRateLimitModal = useCallback(({ remainingRequests = 0, resetTime = null }: OpenRateLimitModalParams) => {
    setRateLimitModal({
      isOpen: true,
      remainingRequests,
      resetTime,
    })
  }, [])

  const closeRateLimitModal = useCallback(() => {
    setRateLimitModal((prev) => ({
      ...prev,
      isOpen: false,
    }))
  }, [])

  const handleSubscribe = useCallback(() => {
    navigate('/subscription')
    closeRateLimitModal()
  }, [navigate, closeRateLimitModal])

  return {
    rateLimitModal,
    openRateLimitModal,
    closeRateLimitModal,
    handleSubscribe,
  }
}

export default useRateLimitModal
