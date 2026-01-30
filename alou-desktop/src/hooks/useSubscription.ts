import { useState, useEffect, useCallback } from 'react'
import subscriptionService from '@/services/subscriptionService'
import useAuthStore from '@/stores/authStore'

/**
 * 订阅信息接口
 */
export interface Subscription {
  id?: string
  status: 'active' | 'cancelled' | 'expired' | 'pending'
  expires_at: number
  plan_id?: string
  [key: string]: any
}

/**
 * 试用信息接口
 */
export interface Trial {
  is_used: boolean
  expires_at: number
  [key: string]: any
}

/**
 * 订阅计划接口
 */
export interface SubscriptionPlan {
  id: string
  name: string
  price: number
  interval: 'month' | 'year'
  features: string[]
  [key: string]: any
}

/**
 * Hook返回结果接口
 */
export interface UseSubscriptionReturn {
  subscription: Subscription | null
  trial: Trial | null
  plans: SubscriptionPlan[]
  loading: boolean
  error: string | null
  isPremium: boolean
  isTrial: boolean
  isExpired: boolean
  daysRemaining: number | null
  trialDaysRemaining: number | null
  refresh: () => Promise<void>
}

/**
 * 订阅状态管理 Hook
 */
export const useSubscription = (): UseSubscriptionReturn => {
  const [subscription, setSubscription] = useState<Subscription | null>(null)
  const [trial, setTrial] = useState<Trial | null>(null)
  const [plans, setPlans] = useState<SubscriptionPlan[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  
  const walletAddress = useAuthStore((state: any) => state.walletAddress)
  const userId = useAuthStore((state: any) => state.userId) || walletAddress

  /**
   * 加载订阅信息
   */
  const loadSubscription = useCallback(async () => {
    if (!walletAddress) {
      setLoading(false)
      return
    }

    try {
      setLoading(true)
      setError(null)

      // 并行加载订阅状态、试用状态和计划列表
      const [statusResult, trialResult, plansResult] = await Promise.all([
        subscriptionService.getStatus(userId, walletAddress).catch(() => ({ has_subscription: false, subscription: null })),
        subscriptionService.checkTrial(userId, walletAddress).catch(() => ({ has_trial: false, trial: null })),
        subscriptionService.getPlans().catch(() => []),
      ])

      setSubscription(statusResult.subscription)
      setTrial(trialResult.trial)
      setPlans(plansResult)
    } catch (err: any) {
      console.error('[useSubscription] Failed to load subscription:', err)
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [userId, walletAddress])

  useEffect(() => {
    loadSubscription()
  }, [loadSubscription])

  // 计算派生状态
  const isPremium = subscription?.status === 'active' && 
                    subscription?.expires_at > Math.floor(Date.now() / 1000)

  const isTrial = trial?.is_used && 
                  trial?.expires_at > Math.floor(Date.now() / 1000)

  const isExpired = subscription !== null && 
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

export default useSubscription
