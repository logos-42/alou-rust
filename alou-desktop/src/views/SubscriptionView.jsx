import React, { useState, useEffect } from 'react'
import { useSubscription } from '@/hooks/useSubscription'
import subscriptionService from '@/services/subscriptionService'
import useAuthStore from '@/stores/authStore'
import { useI18n } from '@/hooks/useI18n'
import './SubscriptionView.css'

const SubscriptionView = () => {
  const { t } = useI18n()
  const { subscription, trial, plans, loading, isPremium, isTrial, daysRemaining, trialDaysRemaining, refresh } = useSubscription()
  const walletAddress = useAuthStore((state) => state.walletAddress)
  const userId = useAuthStore((state) => state.userId) || walletAddress
  const [selectedPlan, setSelectedPlan] = useState(null)
  const [paymentStatus, setPaymentStatus] = useState(null)

  useEffect(() => {
    if (plans.length > 0 && !selectedPlan) {
      setSelectedPlan(plans[0])
    }
  }, [plans, selectedPlan])

  const handleSubscribe = async (plan) => {
    if (!walletAddress) {
      alert('Please connect your wallet first')
      return
    }

    try {
      setPaymentStatus('processing')
      // Here you would integrate with wallet to make payment
      // For now, this is a placeholder
      alert('Payment integration needed - connect to wallet and call smart contract')
      setPaymentStatus('success')
      await refresh()
    } catch (error) {
      console.error('Subscription error:', error)
      setPaymentStatus('error')
      alert(`Subscription failed: ${error.message}`)
    }
  }

  if (loading) {
    return (
      <div className="subscription-view">
        <div className="loading">Loading...</div>
      </div>
    )
  }

  return (
    <div className="subscription-view">
      <div className="subscription-header">
        <h1>{t('subscription.title', 'Subscription')}</h1>
        {isPremium && (
          <div className="premium-badge">
            {t('subscription.premium', 'Premium Member')} - {daysRemaining} {t('subscription.daysRemaining', 'days remaining')}
          </div>
        )}
        {isTrial && !isPremium && (
          <div className="trial-badge">
            {t('subscription.trial', 'Trial Period')} - {trialDaysRemaining} {t('subscription.daysRemaining', 'days remaining')}
          </div>
        )}
      </div>

      <div className="subscription-plans">
        <h2>{t('subscription.choosePlan', 'Choose a Plan')}</h2>
        <div className="plans-grid">
          {plans.map((plan) => (
            <div
              key={plan.id}
              className={`plan-card ${selectedPlan?.id === plan.id ? 'selected' : ''}`}
              onClick={() => setSelectedPlan(plan)}
            >
              <h3>{plan.display_name}</h3>
              <div className="plan-price">
                ${plan.price_usd}
                <span className="plan-period">
                  /{plan.duration_days === 30 ? t('subscription.month', 'month') : t('subscription.year', 'year')}
                </span>
              </div>
              <div className="plan-features">
                <div className="feature">✓ Unlimited requests</div>
                <div className="feature">✓ All features</div>
                <div className="feature">✓ Priority support</div>
              </div>
              <button
                className="subscribe-button"
                onClick={(e) => {
                  e.stopPropagation()
                  handleSubscribe(plan)
                }}
                disabled={paymentStatus === 'processing'}
              >
                {paymentStatus === 'processing'
                  ? t('subscription.processing', 'Processing...')
                  : t('subscription.subscribe', 'Subscribe')}
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="subscription-comparison">
        <h2>{t('subscription.features', 'Features Comparison')}</h2>
        <table className="comparison-table">
          <thead>
            <tr>
              <th>{t('subscription.feature', 'Feature')}</th>
              <th>{t('subscription.free', 'Free')}</th>
              <th>{t('subscription.premium', 'Premium')}</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>{t('subscription.dailyRequests', 'Daily Requests')}</td>
              <td>20</td>
              <td>∞</td>
            </tr>
            <tr>
              <td>{t('subscription.allFeatures', 'All Features')}</td>
              <td>✗</td>
              <td>✓</td>
            </tr>
            <tr>
              <td>{t('subscription.prioritySupport', 'Priority Support')}</td>
              <td>✗</td>
              <td>✓</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  )
}

export default SubscriptionView

