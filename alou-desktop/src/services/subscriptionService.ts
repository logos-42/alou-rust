// Subscription service for API calls
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
  (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev')
const API_BASE = API_BASE_URL ? `${API_BASE_URL}/api` : '/api'

/**
 * 试用状态响应
 */
export interface TrialStatus {
  isActive: boolean
  daysRemaining: number
  startDate?: string
  endDate?: string
  [key: string]: any
}

/**
 * 订阅计划
 */
export interface SubscriptionPlan {
  id: string
  name: string
  description?: string
  price: number
  currency: string
  interval: 'month' | 'year'
  features: string[]
  [key: string]: any
}

/**
 * 订阅状态
 */
export interface SubscriptionStatus {
  id: string
  userId: string
  planId: string
  status: 'active' | 'cancelled' | 'expired' | 'trial'
  startDate?: string
  endDate?: string
  [key: string]: any
}

/**
 * 创建订阅数据
 */
export interface CreateSubscriptionData {
  userId: string
  walletAddress: string
  planId: string
  chainType?: string
  [key: string]: any
}

/**
 * 创建订阅响应
 */
export interface CreateSubscriptionResponse {
  success: boolean
  subscriptionId?: string
  paymentAddress?: string
  amount?: string
  [key: string]: any
}

/**
 * 续订订阅数据
 */
export interface RenewSubscriptionData {
  subscriptionId: string
  [key: string]: any
}

/**
 * 支付验证响应
 */
export interface PaymentVerificationResponse {
  success: boolean
  status?: string
  message?: string
  [key: string]: any
}

/**
 * 通知
 */
export interface Notification {
  id: string
  type: string
  message: string
  createdAt: string
  read?: boolean
  [key: string]: any
}

/**
 * 通知列表响应
 */
export interface NotificationsResponse {
  notifications: Notification[]
  total?: number
  [key: string]: any
}

class SubscriptionService {
  /**
   * Check trial period status
   */
  async checkTrial(userId: string, walletAddress: string): Promise<TrialStatus> {
    const response = await fetch(`${API_BASE}/subscription/check-trial`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
        wallet_address: walletAddress,
      }),
    })

    if (!response.ok) {
      throw new Error(`Failed to check trial: ${response.statusText}`)
    }

    return await response.json() as TrialStatus
  }

  /**
   * Get or create trial period
   */
  async getOrCreateTrial(userId: string, walletAddress: string): Promise<TrialStatus> {
    const response = await fetch(`${API_BASE}/subscription/get-trial`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
        wallet_address: walletAddress,
      }),
    })

    if (!response.ok) {
      throw new Error(`Failed to get trial: ${response.statusText}`)
    }

    return await response.json() as TrialStatus
  }

  /**
   * Get all subscription plans
   */
  async getPlans(): Promise<SubscriptionPlan[]> {
    const response = await fetch(`${API_BASE}/subscription/plans`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get plans: ${response.statusText}`)
    }

    return await response.json() as SubscriptionPlan[]
  }

  /**
   * Get subscription status
   */
  async getStatus(userId: string, walletAddress: string): Promise<SubscriptionStatus> {
    const response = await fetch(`${API_BASE}/subscription/status?user_id=${encodeURIComponent(userId)}&wallet_address=${encodeURIComponent(walletAddress)}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get status: ${response.statusText}`)
    }

    return await response.json() as SubscriptionStatus
  }

  /**
   * Create subscription
   */
  async createSubscription(data: CreateSubscriptionData): Promise<CreateSubscriptionResponse> {
    const response = await fetch(`${API_BASE}/subscription/create`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }))
      throw new Error(error.error || `Failed to create subscription: ${response.statusText}`)
    }

    return await response.json() as CreateSubscriptionResponse
  }

  /**
   * Renew subscription
   */
  async renewSubscription(data: RenewSubscriptionData): Promise<CreateSubscriptionResponse> {
    const response = await fetch(`${API_BASE}/subscription/renew`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })

    if (!response.ok) {
      throw new Error(`Failed to renew subscription: ${response.statusText}`)
    }

    return await response.json() as CreateSubscriptionResponse
  }

  /**
   * Verify payment
   */
  async verifyPayment(txHash: string, chainType: string): Promise<PaymentVerificationResponse> {
    const response = await fetch(`${API_BASE}/subscription/verify-payment`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        tx_hash: txHash,
        chain_type: chainType,
      }),
    })

    if (!response.ok) {
      throw new Error(`Failed to verify payment: ${response.statusText}`)
    }

    return await response.json() as PaymentVerificationResponse
  }

  /**
   * Get notifications
   */
  async getNotifications(daysBefore = 7): Promise<NotificationsResponse> {
    const response = await fetch(`${API_BASE}/subscription/notifications?days_before=${daysBefore}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get notifications: ${response.statusText}`)
    }

    return await response.json() as NotificationsResponse
  }
}

export default new SubscriptionService()
