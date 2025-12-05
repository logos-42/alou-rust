import React, { useState, useEffect, useCallback } from 'react'
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

  const loadIdentity = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      
      // 首先尝试从 localStorage 加载
      if (typeof window !== 'undefined' && window.localStorage) {
        const storedIdentity = localStorage.getItem(`diap_identity_${sessionId}`)
        if (storedIdentity) {
          try {
            const identity = JSON.parse(storedIdentity)
            console.log('[DiapIdentityPanel] 从 localStorage 加载 DIAP 身份:', sessionId, identity)
            setIdentity(identity)
            setLoading(false)
            return // 加载成功，直接返回
          } catch (parseErr) {
            console.warn('[DiapIdentityPanel] 解析 localStorage 数据失败:', parseErr)
            // 解析失败，继续尝试网络加载
          }
        } else {
          console.log('[DiapIdentityPanel] localStorage 中没有找到身份，key:', `diap_identity_${sessionId}`)
        }
      }
      
      // localStorage 没有，尝试从网络加载
      try {
        console.log('[DiapIdentityPanel] 尝试从网络加载身份，sessionId:', sessionId)
        const response = await agentService.getDiapIdentity(sessionId)
        if (response.identity) {
          console.log('[DiapIdentityPanel] 从网络加载成功:', response.identity)
          setIdentity(response.identity)
          // 保存到 localStorage 以便下次快速加载
          if (typeof window !== 'undefined' && window.localStorage) {
            localStorage.setItem(
              `diap_identity_${sessionId}`,
              JSON.stringify(response.identity)
            )
            console.log('[DiapIdentityPanel] 已保存到 localStorage')
          }
        } else {
          console.log('[DiapIdentityPanel] 网络响应中没有 identity')
          setIdentity(null)
        }
      } catch (networkErr) {
        // 网络加载失败不设置错误，因为身份可能确实不存在
        console.log('[DiapIdentityPanel] 网络加载失败，身份可能尚未创建:', networkErr.message)
        setIdentity(null)
      }
    } catch (err) {
      console.error('[DiapIdentityPanel] 加载 DIAP 身份失败:', err)
      setIdentity(null)
      // 只有在严重错误时才显示错误
      // 身份不存在不算错误，用户可以点击创建
    } finally {
      setLoading(false)
    }
  }, [sessionId])

  useEffect(() => {
    if (sessionId) {
      loadIdentity()
    }
  }, [sessionId, loadIdentity])

  const handleCreateIdentity = async () => {
    try {
      setCreating(true)
      setError(null)
      console.log('[DiapIdentityPanel] 开始创建 DIAP 身份，sessionId:', sessionId)
      const response = await agentService.createDiapIdentity(sessionId)
      console.log('[DiapIdentityPanel] 创建响应:', response)
      if (response.identity) {
        const identity = response.identity
        console.log('[DiapIdentityPanel] 创建的身份信息:', identity)
        
        // 保存到 localStorage，以便后续加载
        if (typeof window !== 'undefined' && window.localStorage) {
          localStorage.setItem(
            `diap_identity_${sessionId}`,
            JSON.stringify(identity)
          )
          console.log('[DiapIdentityPanel] DIAP 身份已保存到 localStorage，key:', `diap_identity_${sessionId}`)
          
          // 验证保存是否成功
          const saved = localStorage.getItem(`diap_identity_${sessionId}`)
          if (saved) {
            console.log('[DiapIdentityPanel] 验证：localStorage 保存成功')
          } else {
            console.error('[DiapIdentityPanel] 验证失败：localStorage 保存失败')
          }
        }
        
        // 更新状态以显示身份信息
        setIdentity(identity)
        setToastMessage('DIAP 身份创建成功！')
        
        // 重新加载以确保显示最新信息
        setTimeout(() => {
          loadIdentity()
        }, 100)
      } else {
        console.warn('[DiapIdentityPanel] 创建响应中没有 identity 字段')
        setToastMessage('创建成功，但未返回身份信息')
      }
    } catch (err) {
      console.error('[DiapIdentityPanel] 创建 DIAP 身份失败:', err)
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

