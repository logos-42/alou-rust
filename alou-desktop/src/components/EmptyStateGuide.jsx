import React from 'react'
import { useI18n } from '@/hooks/useI18n'
import './EmptyStateGuide.css'

/**
 * EmptyStateGuide - 空状态引导组件
 * 
 * 当用户没有创建任何智能体时显示友好的引导信息
 * 支持直接输入指令创建智能体
 */
const EmptyStateGuide = ({ onQuickCreate }) => {
  const { t } = useI18n()

  const quickActions = [
    {
      icon: '🤖',
      title: t('agent.emptyState.quickActions.createAgent'),
      description: t('agent.emptyState.quickActions.createAgentDesc'),
      action: () => onQuickCreate?.('agent'),
    },
    {
      icon: '💼',
      title: t('agent.emptyState.quickActions.workAssistant'),
      description: t('agent.emptyState.quickActions.workAssistantDesc'),
      action: () => onQuickCreate?.(t('agent.emptyState.quickActions.workAssistant')),
    },
    {
      icon: '📝',
      title: t('agent.emptyState.quickActions.writingAssistant'),
      description: t('agent.emptyState.quickActions.writingAssistantDesc'),
      action: () => onQuickCreate?.(t('agent.emptyState.quickActions.writingAssistant')),
    },
    {
      icon: '💻',
      title: t('agent.emptyState.quickActions.codingAssistant'),
      description: t('agent.emptyState.quickActions.codingAssistantDesc'),
      action: () => onQuickCreate?.(t('agent.emptyState.quickActions.codingAssistant')),
    },
  ]

  return (
    <div className="empty-state-guide">
      <div className="empty-state-content">
        <div className="empty-state-header">
          <div className="empty-state-icon">👋</div>
          <h2 className="empty-state-title">{t('agent.emptyState.title')}</h2>
          <p className="empty-state-subtitle">{t('agent.emptyState.subtitle')}</p>
        </div>

        <div className="empty-state-input-hint">
          <div className="hint-icon">💡</div>
          <p className="hint-text">
            {t('agent.emptyState.inputHint')}
          </p>
          <div className="hint-examples">
            <span className="example-tag">"创建一个写作助手"</span>
            <span className="example-tag">"帮我分析数据"</span>
            <span className="example-tag">"创建一个编程助手"</span>
          </div>
        </div>

        <div className="empty-state-actions">
          <h3 className="actions-title">{t('agent.emptyState.quickActions.title')}</h3>
          <div className="actions-grid">
            {quickActions.map((action, index) => (
              <button
                key={index}
                type="button"
                className="action-card"
                onClick={action.action}
              >
                <div className="action-icon">{action.icon}</div>
                <div className="action-content">
                  <h4 className="action-title">{action.title}</h4>
                  <p className="action-description">{action.description}</p>
                </div>
              </button>
            ))}
          </div>
        </div>

        <div className="empty-state-tips">
          <h3 className="tips-title">{t('agent.emptyState.tips.title')}</h3>
          <ul className="tips-list">
            <li>{t('agent.emptyState.tips.tip1')}</li>
            <li>{t('agent.emptyState.tips.tip2')}</li>
            <li>{t('agent.emptyState.tips.tip3')}</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default EmptyStateGuide
