import React, { useState, useCallback, useMemo, useEffect, useRef } from 'react'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import './InviteAgentModal.css'

/**
 * 邀请智能体到群组的模态框
 * 支持两种方式：
 * 1. 内部选择：从已保存的智能体列表中选择
 * 2. 外部输入：通过 IPNS/CID/DID 邀请
 */
const InviteAgentModal = ({
  isOpen,
  onClose,
  targetChannel,
  onInvite,
  onResolve,
}) => {
  const { t } = useI18n()
  const [activeTab, setActiveTab] = useState('internal') // 'internal' | 'external'
  const [externalTarget, setExternalTarget] = useState('')
  const [selectedAgents, setSelectedAgents] = useState([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)

  // 当模态框打开时，重置状态（使用 useRef 确保只执行一次）
  const hasResetRef = useRef(false)
  
  useEffect(() => {
    if (isOpen && !hasResetRef.current) {
      hasResetRef.current = true
      // 重置所有状态
      setSelectedAgents([])
      setExternalTarget('')
      setError(null)
      setActiveTab('internal')
      setIsLoading(false)
    } else if (!isOpen && hasResetRef.current) {
      hasResetRef.current = false
      // 关闭时也重置状态
      setSelectedAgents([])
      setExternalTarget('')
      setError(null)
      setActiveTab('internal')
      setIsLoading(false)
    }
  }, [isOpen])

  // 获取本地存储的智能体列表（排除当前频道的智能体）
  const storedAgents = useAgentStore((state) => state.agents)
  const availableAgents = useMemo(() => {
    if (!targetChannel) return storedAgents
    const targetId = targetChannel.meta?.ipns || targetChannel.meta?.cid || targetChannel.meta?.did || targetChannel.id
    return storedAgents.filter(agent => {
      const agentId = agent.ipns || agent.cid || agent.did || agent.id
      return agentId !== targetId
    })
  }, [storedAgents, targetChannel])

  const toggleAgentSelection = useCallback((agent, e) => {
    if (e) {
      e.preventDefault()
      e.stopPropagation()
    }
    setSelectedAgents((prev) => {
      const agentId = agent.ipns || agent.cid || agent.did || agent.id
      const isSelected = prev.some(a => (a.ipns || a.cid || a.did || a.id) === agentId)
      return isSelected
        ? prev.filter(a => (a.ipns || a.cid || a.did || a.id) !== agentId)
        : [...prev, agent]
    })
  }, [])

  const handleInternalInvite = useCallback(async () => {
    if (selectedAgents.length === 0) {
      setError(t('agent.invite.error.selectAtLeastOne'))
      return
    }
    
    setIsLoading(true)
    setError(null)
    
    try {
      await onInvite?.(targetChannel, selectedAgents, 'internal')
      onClose()
    } catch (err) {
      console.error('[InviteAgentModal] 邀请失败:', err)
      setError(err.message || t('agent.invite.error.inviteFailed'))
    } finally {
      setIsLoading(false)
    }
  }, [selectedAgents, targetChannel, onInvite, onClose, t])

  const handleExternalInvite = useCallback(async () => {
    const target = externalTarget.trim()
    if (!target) {
      setError(t('agent.invite.error.enterIdentifier'))
      return
    }
    
    setIsLoading(true)
    setError(null)
    
    try {
      // 先解析智能体
      const resolvedAgent = await onResolve?.(target)
      if (resolvedAgent) {
        await onInvite?.(targetChannel, [resolvedAgent], 'external')
        onClose()
      }
    } catch (err) {
      setError(err.message || t('agent.invite.error.resolveFailed'))
    } finally {
      setIsLoading(false)
    }
  }, [externalTarget, targetChannel, onInvite, onResolve, onClose, t])

  const handleClose = useCallback(() => {
    setSelectedAgents([])
    setExternalTarget('')
    setError(null)
    setActiveTab('internal')
    onClose()
  }, [onClose])

  if (!isOpen) {
    return null
  }

  return (
    <div 
      className="invite-modal-overlay" 
      onClick={handleClose}
    >
      <div 
        className="invite-modal" 
        onClick={(e) => e.stopPropagation()}
      >
        <header className="invite-modal-header">
          <h2>{t('agent.invite.title')}</h2>
          <span className="target-channel">
            {t('agent.invite.channel')}: {targetChannel?.name || t('agent.invite.channel.unknown')}
          </span>
          <button type="button" className="close-btn" onClick={handleClose}>
            <img src={CloseIcon} alt="关闭" />
          </button>
        </header>

        <div className="invite-tabs">
          <button
            type="button"
            className={`tab-btn ${activeTab === 'internal' ? 'active' : ''}`}
            onClick={() => setActiveTab('internal')}
          >
            {t('agent.invite.tab.local')}
          </button>
          <button
            type="button"
            className={`tab-btn ${activeTab === 'external' ? 'active' : ''}`}
            onClick={() => setActiveTab('external')}
          >
            {t('agent.invite.tab.external')}
          </button>
        </div>

        <div className="invite-modal-content">
          {activeTab === 'internal' && (
            <div className="internal-invite">
              {availableAgents.length === 0 ? (
                <div className="empty-state">
                  <p>{t('agent.invite.local.empty')}</p>
                  <small>{t('agent.invite.local.emptyHint')}</small>
                </div>
              ) : (
                <div className="agent-list">
                  {availableAgents.map((agent) => {
                    const agentId = agent.ipns || agent.cid || agent.did || agent.id
                    const isSelected = selectedAgents.some(a => 
                      (a.ipns || a.cid || a.did || a.id) === agentId
                    )
                    return (
                      <div
                        key={agentId}
                        className={`agent-item ${isSelected ? 'selected' : ''}`}
                        onClick={(e) => {
                          e.preventDefault()
                          e.stopPropagation()
                          toggleAgentSelection(agent, e)
                        }}
                        role="button"
                        tabIndex={0}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault()
                            e.stopPropagation()
                            toggleAgentSelection(agent, e)
                          }
                        }}
                      >
                        <div className="agent-avatar">
                          {agent.avatar_url ? (
                            <img src={agent.avatar_url} alt={agent.display_name || agent.name} />
                          ) : (
                            <span>{(agent.display_name || agent.name || 'A')[0].toUpperCase()}</span>
                          )}
                        </div>
                        <div className="agent-info">
                          <div className="agent-name">{agent.display_name || agent.name}</div>
                          <div className="agent-id">{agentId.slice(0, 20)}...</div>
                        </div>
                        <div className="agent-check">
                          {isSelected && (
                            <svg viewBox="0 0 20 20" fill="currentColor">
                              <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                            </svg>
                          )}
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
              
              {selectedAgents.length > 0 && (
                <div className="selected-count">
                  {t('agent.invite.local.selected', { count: selectedAgents.length })}
                </div>
              )}
            </div>
          )}

          {activeTab === 'external' && (
            <div className="external-invite">
              <label htmlFor="external-target">
                {t('agent.invite.external.label')}
              </label>
              <input
                id="external-target"
                type="text"
                value={externalTarget}
                onChange={(e) => setExternalTarget(e.target.value)}
                placeholder={t('agent.invite.external.placeholder')}
              />
              <small>
                {t('agent.invite.external.hint')}
              </small>
            </div>
          )}

          {error && <div className="invite-error">{error}</div>}
        </div>

        <footer className="invite-modal-footer">
          <button 
            type="button" 
            className="cancel-btn" 
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              handleClose()
            }}
          >
            {t('agent.invite.cancel')}
          </button>
          <button
            type="button"
            className="invite-btn"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              
              // 检查按钮是否被禁用
              const isDisabled = isLoading || 
                (activeTab === 'internal' && (!selectedAgents || selectedAgents.length === 0)) || 
                (activeTab === 'external' && !externalTarget.trim())
              
              if (isDisabled) {
                return
              }
              
              if (activeTab === 'internal') {
                handleInternalInvite()
              } else {
                handleExternalInvite()
              }
            }}
            disabled={isLoading || (activeTab === 'internal' && (!selectedAgents || selectedAgents.length === 0)) || (activeTab === 'external' && !externalTarget.trim())}
          >
            {isLoading ? t('agent.invite.processing') : t('agent.invite.submit')}
          </button>
        </footer>
      </div>
    </div>
  )
}

export default InviteAgentModal

