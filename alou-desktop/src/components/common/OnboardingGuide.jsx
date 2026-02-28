import React, { useState, useEffect } from 'react'
import { useI18n } from '@/hooks/useI18n'
import './OnboardingGuide.css'

/**
 * 用户引导组件
 * 为首次使用的用户提供引导提示
 */
export const OnboardingGuide = ({ 
  steps = [],
  storageKey = 'onboarding-completed',
  onComplete,
}) => {
  const { t } = useI18n()
  const [isVisible, setIsVisible] = useState(false)
  const [currentStep, setCurrentStep] = useState(0)
  const [hasCompleted, setHasCompleted] = useState(false)

  useEffect(() => {
    // 检查是否已完成引导
    if (typeof window !== 'undefined') {
      const completed = localStorage.getItem(storageKey)
      if (!completed && steps.length > 0) {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setIsVisible(true)
      } else {
        // eslint-disable-next-line react-hooks/setState-in-effect
        setHasCompleted(true)
      }
    }
  }, [storageKey, steps.length])

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1)
    } else {
      handleComplete()
    }
  }

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1)
    }
  }

  const handleComplete = () => {
    setIsVisible(false)
    setHasCompleted(true)
    
    if (typeof window !== 'undefined') {
      localStorage.setItem(storageKey, 'true')
    }
    
    onComplete?.()
  }

  const handleSkip = () => {
    handleComplete()
  }

  if (!isVisible || steps.length === 0) return null

  const step = steps[currentStep]
  const progress = ((currentStep + 1) / steps.length) * 100

  return (
    <div className="onboarding-overlay" role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
      <div className="onboarding-container">
        <div className="onboarding-content">
          <div className="onboarding-step-indicator">
            <span className="onboarding-step-current">{currentStep + 1}</span>
            <span className="onboarding-step-total">/ {steps.length}</span>
          </div>

          <div className="onboarding-icon" aria-hidden="true">
            {step.icon || '💡'}
          </div>

          <h2 id="onboarding-title" className="onboarding-title">
            {step.title}
          </h2>

          <p className="onboarding-description">
            {step.description}
          </p>

          {step.tip && (
            <div className="onboarding-tip">
              <span className="onboarding-tip-icon">💡</span>
              <span>{step.tip}</span>
            </div>
          )}

          {step.shortcut && (
            <div className="onboarding-shortcut">
              <span className="onboarding-shortcut-label">{t('onboarding.shortcut')}:</span>
              <kbd className="onboarding-shortcut-key">{step.shortcut}</kbd>
            </div>
          )}
        </div>

        <div className="onboarding-progress">
          <div 
            className="onboarding-progress-bar"
            style={{ width: `${progress}%` }}
            aria-hidden="true"
          />
        </div>

        <div className="onboarding-actions">
          <button
            type="button"
            className="onboarding-btn onboarding-btn-skip"
            onClick={handleSkip}
          >
            {t('onboarding.skip')}
          </button>

          <div className="onboarding-nav">
            {currentStep > 0 && (
              <button
                type="button"
                className="onboarding-btn onboarding-btn-secondary"
                onClick={handlePrevious}
              >
                {t('onboarding.previous')}
              </button>
            )}

            <button
              type="button"
              className="onboarding-btn onboarding-btn-primary"
              onClick={handleNext}
            >
              {currentStep === steps.length - 1 ? t('onboarding.finish') : t('onboarding.next')}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

/**
 * 空状态引导组件
 */
export const EmptyStateGuide = ({ 
  icon = '💬',
  title,
  description,
  actionText,
  onAction,
  tip,
}) => {
  const { t } = useI18n()

  return (
    <div className="empty-state-guide">
      <div className="empty-state-icon" aria-hidden="true">
        {icon}
      </div>
      <h3 className="empty-state-title">{title || t('emptyState.title')}</h3>
      <p className="empty-state-description">{description || t('emptyState.description')}</p>
      
      {tip && (
        <div className="empty-state-tip">
          <span className="empty-state-tip-icon">💡</span>
          <span>{tip}</span>
        </div>
      )}

      {actionText && onAction && (
        <button
          type="button"
          className="empty-state-action"
          onClick={onAction}
        >
          {actionText}
        </button>
      )}
    </div>
  )
}

/**
 * 快捷键提示组件
 */
export const ShortcutHint = ({ shortcuts = [] }) => {
  const { t } = useI18n()
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    const handleKeyDown = (e) => {
      // 按 ? 显示快捷键提示
      if (e.key === '?' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault()
        setIsVisible(prev => !prev)
      }
      
      // 按 Escape 关闭
      if (e.key === 'Escape') {
        setIsVisible(false)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  if (shortcuts.length === 0) return null

  return (
    <>
      {/* 浮动提示按钮 */}
      <button
        type="button"
        className="shortcut-hint-btn"
        onClick={() => setIsVisible(true)}
        title={t('shortcuts.show')}
        aria-label={t('shortcuts.show')}
      >
        ⌨️
      </button>

      {/* 快捷键面板 */}
      {isVisible && (
        <div 
          className="shortcut-panel-overlay"
          onClick={() => setIsVisible(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="shortcut-panel-title"
        >
          <div className="shortcut-panel" onClick={e => e.stopPropagation()}>
            <div className="shortcut-panel-header">
              <h3 id="shortcut-panel-title" className="shortcut-panel-title">
                {t('shortcuts.title')}
              </h3>
              <button
                type="button"
                className="shortcut-panel-close"
                onClick={() => setIsVisible(false)}
                aria-label={t('common.close')}
              >
                ×
              </button>
            </div>
            
            <div className="shortcut-list">
              {shortcuts.map((shortcut, index) => (
                <div key={index} className="shortcut-item">
                  <span className="shortcut-description">{shortcut.description}</span>
                  <kbd className="shortcut-key">{shortcut.key}</kbd>
                </div>
              ))}
            </div>

            <div className="shortcut-panel-footer">
              {t('shortcuts.pressToShow')}
            </div>
          </div>
        </div>
      )}
    </>
  )
}

export default OnboardingGuide
