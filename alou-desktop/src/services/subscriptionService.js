// Subscription service for API calls
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
  (import.meta.env.DEV ? 'http://localhost:1420' : 'https://alou-edge.yuanjieliu65.workers.dev')

class SubscriptionService {
  /**
   * Check trial period status
   */
  async checkTrial(userId, walletAddress) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/check-trial`, {
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

    return await response.json()
  }

  /**
   * Get or create trial period
   */
  async getOrCreateTrial(userId, walletAddress) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/get-trial`, {
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

    return await response.json()
  }

  /**
   * Get all subscription plans
   */
  async getPlans() {
    const response = await fetch(`${API_BASE_URL}/api/subscription/plans`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get plans: ${response.statusText}`)
    }

    return await response.json()
  }

  /**
   * Get subscription status
   */
  async getStatus(userId, walletAddress) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/status?user_id=${encodeURIComponent(userId)}&wallet_address=${encodeURIComponent(walletAddress)}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get status: ${response.statusText}`)
    }

    return await response.json()
  }

  /**
   * Create subscription
   */
  async createSubscription(data) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/create`, {
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

    return await response.json()
  }

  /**
   * Renew subscription
   */
  async renewSubscription(data) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/renew`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    })

    if (!response.ok) {
      throw new Error(`Failed to renew subscription: ${response.statusText}`)
    }

    return await response.json()
  }

  /**
   * Verify payment
   */
  async verifyPayment(txHash, chainType) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/verify-payment`, {
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

    return await response.json()
  }

  /**
   * Get notifications
   */
  async getNotifications(daysBefore = 7) {
    const response = await fetch(`${API_BASE_URL}/api/subscription/notifications?days_before=${daysBefore}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to get notifications: ${response.statusText}`)
    }

    return await response.json()
  }
}

export default new SubscriptionService()

