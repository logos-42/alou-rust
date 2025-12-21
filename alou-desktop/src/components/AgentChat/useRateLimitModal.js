import { useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'

/**
 * Hook for managing rate limit modal state
 * 管理限额弹窗的状态
 */
export const useRateLimitModal = () => {
  const navigate = useNavigate()
  const [rateLimitModal, setRateLimitModal] = useState({
    isOpen: false,
    remainingRequests: 0,
    resetTime: null,
  })

  const openRateLimitModal = useCallback(({ remainingRequests = 0, resetTime = null }) => {
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

