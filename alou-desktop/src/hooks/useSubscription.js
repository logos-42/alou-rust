import { useState, useEffect, useCallback } from 'react'
import subscriptionService from '@/services/subscriptionService'
import useAuthStore from '@/stores/authStore'

/**
 * Hook for managing subscription state
 */
export const useSubscription = () => {
  const [subscription, setSubscription] = useState(null)
  const [trial, setTrial] = useState(null)
  const [plans, setPlans] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  
  const walletAddress = useAuthStore((state) => state.walletAddress)
  const userId = useAuthStore((state) => state.userId) || walletAddress

  const loadSubscription = useCallback(async () => {
    if (!walletAddress) {
      setLoading(false)
      return
    }

    try {
      setLoading(true)
      setError(null)

      // Load subscription status and trial in parallel
      const [statusResult, trialResult, plansResult] = await Promise.all([
        subscriptionService.getStatus(userId, walletAddress).catch(() => ({ has_subscription: false, subscription: null })),
        subscriptionService.checkTrial(userId, walletAddress).catch(() => ({ has_trial: false, trial: null })),
        subscriptionService.getPlans().catch(() => []),
      ])

      setSubscription(statusResult.subscription)
      setTrial(trialResult.trial)
      setPlans(plansResult)
    } catch (err) {
      console.error('[useSubscription] Failed to load subscription:', err)
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [userId, walletAddress])

  useEffect(() => {
    loadSubscription()
  }, [loadSubscription])

  const isPremium = subscription?.status === 'active' && 
                    subscription?.expires_at > Math.floor(Date.now() / 1000)

  const isTrial = trial?.is_used && 
                  trial?.expires_at > Math.floor(Date.now() / 1000)

  const isExpired = subscription && 
                    subscription.expires_at <= Math.floor(Date.now() / 1000)

  const daysRemaining = subscription && !isExpired
    ? Math.ceil((subscription.expires_at - Math.floor(Date.now() / 1000)) / 86400)
    : null

  const trialDaysRemaining = trial && isTrial
    ? Math.ceil((trial.expires_at - Math.floor(Date.now() / 1000)) / 86400)
    : null

  return {
    subscription,
    trial,
    plans,
    loading,
    error,
    isPremium,
    isTrial,
    isExpired,
    daysRemaining,
    trialDaysRemaining,
    refresh: loadSubscription,
  }
}

