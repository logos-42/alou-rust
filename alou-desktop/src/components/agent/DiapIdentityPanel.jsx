import React, { useState, useEffect } from 'react'
import agentService from '@/services/agentService'
import './DiapIdentityPanel.css'

const DiapIdentityPanel = ({ sessionId, onClose, isDarkMode = false }) => {
  const [identity, setIdentity] = useState(null)
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [registering, setRegistering] = useState(false)
  const [error, setError] = useState(null)
  const [registerInfo, setRegisterInfo] = useState(null)
  const [toastMessage, setToastMessage] = useState(null)

  useEffect(() => {
    if (sessionId) {
      loadIdentity()
    }
  }, [sessionId])

  const loadIdentity = async () => {
    try {
      setLoading(true)
      setError(null)
      const response = await agentService.getDiapIdentity(sessionId)
      if (response.identity) {
        setIdentity(response.identity)
      }
    } catch (err) {
      console.error('Failed to load DIAP identity:', err)
      const message = err.message || '加载身份信息失败'
      setError(message)
      setToastMessage(message)
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
      }
    } catch (err) {
      console.error('Failed to create DIAP identity:', err)
      const message = err.message || '创建身份失败'
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
      const message = err.message || '注册到链上失败'
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
          <h3>DIAP 身份</h3>
          {onClose && (
            <button type="button" className="close-btn" onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className="diap-panel-content loading">加载中...</div>
      </div>
    )
  }

  return (
    <div className={`diap-identity-panel ${isDarkMode ? 'dark' : 'light'}`}>
      <div className="diap-panel-header">
        <h3>DIAP 身份</h3>
        {onClose && (
          <button type="button" className="close-btn" onClick={onClose}>
            ×
          </button>
        )}
      </div>
      <div className="diap-panel-content">
        {!identity ? (
          <div className="diap-no-identity">
            <p>此智能体尚未创建 DIAP 身份</p>
            <button
              type="button"
              className="create-identity-btn"
              onClick={handleCreateIdentity}
              disabled={creating}
            >
              {creating ? '创建中...' : '创建 DIAP 身份'}
            </button>
          </div>
        ) : (
          <div className="diap-identity-info">
            <div className="diap-field">
              <label>DID</label>
              <div className="diap-value">
                <code>{identity.did}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.did)}
                  title="复制"
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>IPNS</label>
              <div className="diap-value">
                <code>{identity.ipns}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.ipns)}
                  title="复制"
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>CID</label>
              <div className="diap-value">
                <code>{identity.cid}</code>
                <button
                  type="button"
                  className="copy-btn"
                  onClick={() => copyToClipboard(identity.cid)}
                  title="复制"
                >
                  📋
                </button>
              </div>
            </div>

            <div className="diap-field">
              <label>链上注册状态</label>
              <div className="diap-value">
                {identity.is_registered ? (
                  <span className="status-registered">
                    ✓ 已注册
                    {identity.registered_address && (
                      <code className="registered-address">
                        {identity.registered_address}
                      </code>
                    )}
                  </span>
                ) : (
                  <div className="register-section">
                    <span className="status-unregistered">未注册</span>
                    <button
                      type="button"
                      className="register-btn"
                      onClick={handleRegisterOnChain}
                      disabled={registering}
                    >
                      {registering ? '注册中...' : '注册到链上'}
                    </button>
                  </div>
                )}
              </div>
            </div>

            {registerInfo && (
              <div className="diap-field register-info">
                <label>注册交易信息</label>
                <div className="diap-value">
                  <p className="info-text">
                    已生成编码交易，请使用钱包签名并广播此交易：
                  </p>
                  <div className="encoded-call">
                    <code>{registerInfo.encoded_call?.data || 'N/A'}</code>
                    <button
                      type="button"
                      className="copy-btn"
                      onClick={() => copyToClipboard(registerInfo.encoded_call?.data || '')}
                      title="复制"
                    >
                      📋
                    </button>
                  </div>
                  {registerInfo.registration_fee && (
                    <p className="info-text">
                      注册费用: {registerInfo.registration_fee} wei
                    </p>
                  )}
                  {registerInfo.min_stake_amount && (
                    <p className="info-text">
                      最小质押: {registerInfo.min_stake_amount} wei
                    </p>
                  )}
                </div>
              </div>
            )}

            {identity.created_at && (
              <div className="diap-field">
                <label>创建时间</label>
                <div className="diap-value">
                  {new Date(identity.created_at * 1000).toLocaleString('zh-CN')}
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

