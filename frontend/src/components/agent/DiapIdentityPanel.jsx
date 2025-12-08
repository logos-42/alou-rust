import React, { useState, useEffect } from 'react'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import './DiapIdentityPanel.css'

const DiapIdentityPanel = ({ sessionId, selectedAgent, onClose, isDarkMode = false }) => {
  const { t } = useI18n()
  const [identity, setIdentity] = useState(null)
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [registering, setRegistering] = useState(false)
  const [error, setError] = useState(null)
  const [registerInfo, setRegisterInfo] = useState(null)
  const [toastMessage, setToastMessage] = useState(null)
  
  const updateAgent = useAgentStore((state) => state.updateAgent)

  useEffect(() => {
    if (sessionId) {
      loadIdentity()
    }
  }, [sessionId, selectedAgent])

  const loadIdentity = async () => {
    try {
      setLoading(true)
      setError(null)
      
      // 优先级1: 从 selectedAgent 元数据中查找
      if (selectedAgent?.diapIdentity) {
        console.log('[DiapIdentityPanel] 从智能体元数据加载 DIAP 身份:', selectedAgent.diapIdentity.ipns || selectedAgent.diapIdentity.did)
        console.log('[DiapIdentityPanel] 完整身份信息:', selectedAgent.diapIdentity)
        setIdentity(selectedAgent.diapIdentity)
        setLoading(false)
        return
      } else {
        console.log('[DiapIdentityPanel] selectedAgent 信息:', {
          hasSelectedAgent: !!selectedAgent,
          hasDiapIdentity: !!selectedAgent?.diapIdentity,
          agentId: selectedAgent?.id,
          ipns: selectedAgent?.ipns,
          cid: selectedAgent?.cid,
          did: selectedAgent?.did,
        })
      }
      
      // 优先级2: 从 IPNS/CID/DID 查找（如果智能体有这些标识）
      const agentTarget = selectedAgent?.ipns || selectedAgent?.cid || selectedAgent?.did
      if (agentTarget) {
        try {
          const response = await agentService.getDiapIdentity(agentTarget)
          if (response.identity) {
            console.log('[DiapIdentityPanel] 从 IPNS/CID/DID 加载 DIAP 身份:', agentTarget)
            setIdentity(response.identity)
            // 更新智能体元数据
            if (selectedAgent?.id) {
              updateAgent(selectedAgent.id, { diapIdentity: response.identity })
            }
            setLoading(false)
            return
          }
        } catch (targetErr) {
          console.log('[DiapIdentityPanel] 从 IPNS/CID/DID 加载失败:', targetErr.message)
        }
      }
      
      // 优先级3: 从 localStorage 加载（使用 sessionId）
      if (typeof window !== 'undefined' && window.localStorage) {
        const storedIdentity = localStorage.getItem(`diap_identity_${sessionId}`)
        if (storedIdentity) {
          try {
            const identity = JSON.parse(storedIdentity)
            console.log('[DiapIdentityPanel] 从 localStorage 加载 DIAP 身份:', sessionId)
            setIdentity(identity)
            // 如果智能体存在，更新其元数据
            if (selectedAgent?.id) {
              updateAgent(selectedAgent.id, { diapIdentity: identity })
            }
            setLoading(false)
            return
          } catch (parseErr) {
            console.warn('[DiapIdentityPanel] 解析 localStorage 数据失败:', parseErr)
          }
        }
      }
      
      // 优先级4: 从网络加载（使用 sessionId）
      try {
      const response = await agentService.getDiapIdentity(sessionId)
      if (response.identity) {
        setIdentity(response.identity)
          // 保存到 localStorage 和智能体元数据
          if (typeof window !== 'undefined' && window.localStorage) {
            localStorage.setItem(
              `diap_identity_${sessionId}`,
              JSON.stringify(response.identity)
            )
          }
          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, { diapIdentity: response.identity })
          }
        }
      } catch (networkErr) {
        // 网络加载失败不设置错误，因为身份可能确实不存在
        console.log('[DiapIdentityPanel] 网络加载失败，身份可能尚未创建:', networkErr.message)
      }
    } catch (err) {
      console.error('Failed to load DIAP identity:', err)
      // 只有在严重错误时才显示错误
      // 身份不存在不算错误，用户可以点击创建
    } finally {
      setLoading(false)
    }
  }

  const handleCreateIdentity = async () => {
    try {
      setCreating(true)
      setError(null)
      const response = await agentService.createDiapIdentity(sessionId)
      if (response.identity) {
        setIdentity(response.identity)
        
        // 保存到 localStorage（使用 sessionId）
        if (typeof window !== 'undefined' && window.localStorage) {
          localStorage.setItem(
            `diap_identity_${sessionId}`,
            JSON.stringify(response.identity)
          )
          console.log('[DiapIdentityPanel] DIAP 身份已保存到 localStorage:', sessionId)
        }
        
        // 保存到智能体元数据（使用 IPNS/CID/DID 作为 key）
        if (selectedAgent?.id) {
          updateAgent(selectedAgent.id, { 
            diapIdentity: response.identity,
            ipns: response.identity.ipns,
            cid: response.identity.cid,
            did: response.identity.did,
          })
          console.log('[DiapIdentityPanel] DIAP 身份已保存到智能体元数据:', selectedAgent.id)
        } else if (response.identity.ipns || response.identity.cid || response.identity.did) {
          // 如果没有 selectedAgent，使用 IPNS/CID/DID 作为 key 保存
          const agentId = response.identity.ipns || response.identity.cid || response.identity.did
          updateAgent(agentId, {
            id: agentId,
            sessionId,
            diapIdentity: response.identity,
            ipns: response.identity.ipns,
            cid: response.identity.cid,
            did: response.identity.did,
          })
          console.log('[DiapIdentityPanel] DIAP 身份已保存到智能体元数据（新）:', agentId)
        }
        
        setToastMessage(t('agent.diap.createSuccess'))
      }
    } catch (err) {
      console.error('Failed to create DIAP identity:', err)
      const message = err.message || t('agent.diap.createFailed')
      setError(message)
      setToastMessage(message)
    } finally {
      setCreating(false)
    }
  }

  const handleRegisterOnChain = async () => {
    if (!identity) return

    const network = window.prompt('请输入网络名称（如：base_sepolia, base_mainnet）', 'base_sepolia')
    if (!network) return

    const stakeAmount = window.prompt('请输入质押金额（单位：wei，例如 100000000000000000000 表示 100 代币）', '100000000000000000000')
    if (!stakeAmount) return

    const useAa = window.confirm('是否使用 ERC-4337 AA 账户？\n\n点击"确定"使用 AA 账户\n点击"取消"使用传统 EOA')

    try {
      setRegistering(true)
      setError(null)
      const result = await agentService.registerAgentOnChain(
        sessionId,
        network,
        stakeAmount,
        useAa,
        0, // salt
      )
      setRegisterInfo(result)
      // Reload identity to check registration status
      await loadIdentity()
    } catch (err) {
      console.error('Failed to register agent on-chain:', err)
      const message = err.message || t('agent.diap.registerFailed')
      setError(message)
      setToastMessage(message)
    } finally {
      setRegistering(false)
    }
  }

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text).then(() => {
      // Could show a toast notification here
    })
  }

  useEffect(() => {
    if (!toastMessage) return
    const timer = setTimeout(() => setToastMessage(null), 5000)
    return () => clearTimeout(timer)
  }, [toastMessage])

  if (loading) {
    return (
      <div className={`diap-identity-panel ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="diap-panel-header">
          <h3>{t('agent.diap.title')}</h3>
          {onClose && (
            <button type="button" className="close-btn" onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className="diap-panel-content loading">{t('common.loading')}</div>
      </div>
    )
  }

  return (
    <div className={`diap-identity-panel ${isDarkMode ? 'dark' : 'light'}`}>
      <div className="diap-panel-header">
        <h3>{t('agent.diap.title')}</h3>
        {onClose && (
          <button type="button" className="close-btn" onClick={onClose}>
            ×
          </button>
        )}
      </div>
      <div className="diap-panel-content">
        {!identity ? (
          <div className="diap-no-identity">
            <p>{t('agent.diap.noIdentity')}</p>
            <button
              type="button"
              className="create-identity-btn"
              onClick={handleCreateIdentity}
              disabled={creating}
            >
              {creating ? t('agent.diap.creating') : t('agent.diap.create')}
            </button>
          </div>
        ) : (
          <div className="diap-identity-info">
            <div className="diap-field">
              <label>{t('agent.diap.did')}</label>
              <div className="diap-value">
                <code>{identity.did}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.did)}
                  title={t('agent.diap.copy')}
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>{t('agent.diap.ipns')}</label>
              <div className="diap-value">
                <code>{identity.ipns}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.ipns)}
                  title={t('agent.diap.copy')}
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>{t('agent.diap.cid')}</label>
              <div className="diap-value">
                <code>{identity.cid}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.cid)}
                  title={t('agent.diap.copy')}
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>{t('agent.diap.registrationStatus')}</label>
              <div className="diap-value">
                {identity.is_registered ? (
                  <span className="status-registered">
                    ✓ {t('agent.diap.registered')}
                    {identity.registered_address && (
                      <code className="registered-address">
                        {identity.registered_address}
                      </code>
                    )}
                  </span>
                ) : (
                  <div className="register-section">
                    <span className="status-unregistered">{t('agent.diap.notRegistered')}</span>
                    <button
                      type="button"
                      className="register-btn"
                      onClick={handleRegisterOnChain}
                      disabled={registering}
                    >
                      {registering ? t('agent.diap.registering') : t('agent.diap.register')}
                    </button>
                  </div>
                )}
              </div>
            </div>

            {registerInfo && (
              <div className="diap-field register-info">
                <label>{t('agent.diap.registerTxInfo')}</label>
                <div className="diap-value">
                  <p className="info-text">
                    {t('agent.diap.registerTxHint')}
                  </p>
                  <div className="encoded-call">
                    <code>{registerInfo.encoded_call?.data || 'N/A'}</code>
                    <button
                      type="button"
                      className="copy-btn"
                      onClick={() => copyToClipboard(registerInfo.encoded_call?.data || '')}
                      title={t('agent.diap.copy')}
                    >
                      📋
                    </button>
                  </div>
                  {registerInfo.registration_fee && (
                    <p className="info-text">
                      {t('agent.diap.registrationFee')}: {registerInfo.registration_fee} wei
                    </p>
                  )}
                  {registerInfo.min_stake_amount && (
                    <p className="info-text">
                      {t('agent.diap.minStakeAmount')}: {registerInfo.min_stake_amount} wei
                    </p>
                  )}
                </div>
              </div>
            )}

            {identity.created_at && (
              <div className="diap-field">
                <label>{t('agent.diap.createdAt')}</label>
                <div className="diap-value">
                  {new Date(identity.created_at * 1000).toLocaleString()}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
      {toastMessage && (
        <div className="diap-toast" role="status" aria-live="polite">
          {toastMessage}
        </div>
      )}
    </div>
  )
}

export default DiapIdentityPanel

