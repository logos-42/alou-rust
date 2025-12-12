import React, { useState, useEffect } from 'react'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import { invoke } from '@tauri-apps/api/core'
import { ethers } from 'ethers'
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
  const [txHash, setTxHash] = useState(null)
  const [hasTestnetKey, setHasTestnetKey] = useState(false)
  
  const updateAgent = useAgentStore((state) => state.updateAgent)
  
  // Check if testnet private key is available
  useEffect(() => {
    const checkTestnetKey = async () => {
      try {
        await invoke('get_testnet_private_key')
        setHasTestnetKey(true)
      } catch (err) {
        setHasTestnetKey(false)
      }
    }
    checkTestnetKey()
  }, [])

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
      // 注意：404 错误是正常的，表示身份尚未创建，不应该显示错误
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
        // 404 错误是正常的（身份尚未创建），其他错误才记录警告
        const is404 = networkErr?.response?.status === 404 || 
                     networkErr?.message?.includes('404') ||
                     networkErr?.message?.includes('not found')
        if (!is404) {
          console.warn('[DiapIdentityPanel] 网络加载失败:', networkErr.message)
        } else {
          console.log('[DiapIdentityPanel] 身份尚未创建（这是正常的）')
        }
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

    const network = window.prompt('请输入网络名称（如：base_sepolia, sepolia）', 'base_sepolia')
    if (!network) return

    const stakeAmount = window.prompt('请输入质押金额（单位：wei，例如 100000000000000000000 表示 100 代币）', '100000000000000000000')
    if (!stakeAmount) return

    const useAa = window.confirm('是否使用 ERC-4337 AA 账户？\n\n点击"确定"使用 AA 账户\n点击"取消"使用传统 EOA')

    try {
      setRegistering(true)
      setError(null)
      setTxHash(null)
      
      // Get encoded transaction from backend
      const result = await agentService.registerAgentOnChain(
        identity,
        network,
        stakeAmount,
        useAa,
        0, // salt
      )
      
      // Check if we should auto-sign and broadcast
      let shouldAutoSign = false
      if (hasTestnetKey) {
        shouldAutoSign = window.confirm(
          '检测到测试网私钥环境变量（DIAP_TESTNET_PRIVATE_KEY）\n\n' +
          '是否自动签名并广播交易？\n\n' +
          '点击"确定"自动签名并广播\n' +
          '点击"取消"仅生成编码交易（需要手动签名）'
        )
      }
      
      if (shouldAutoSign && result.encoded_call?.data) {
        try {
          // Get private key from environment
          const privateKey = await invoke('get_testnet_private_key')
          
          // Get RPC URL based on network
          let rpcUrl, chainId
          if (network === 'base_sepolia') {
            rpcUrl = 'https://sepolia.base.org'
            chainId = 84532
          } else if (network === 'sepolia') {
            rpcUrl = 'https://ethereum-sepolia-rpc.publicnode.com'
            chainId = 11155111
          } else {
            throw new Error(`不支持的测试网络: ${network}`)
          }
          
          // Get contract address based on network
          // DIAPAgentNetwork contract addresses
          let contractAddress
          if (network === 'base_sepolia') {
            contractAddress = '0xA960cf9053FA76278e16f9D4BA35225f7634DC54' // Base Sepolia DIAPAgentNetwork
          } else if (network === 'sepolia') {
            contractAddress = '0x9eF71FD5be68ebab2ABE20c5Fab826b14BfBc089' // Sepolia DIAPAgentNetwork
          } else {
            throw new Error(`不支持的测试网络: ${network}`)
          }
          
          // Create provider and wallet
          const provider = new ethers.JsonRpcProvider(rpcUrl)
          const wallet = new ethers.Wallet(privateKey, provider)
          
          // Get transaction parameters
          const nonce = await provider.getTransactionCount(wallet.address, 'latest')
          const gasPrice = await provider.getFeeData()
          
          // Build transaction
          const tx = {
            to: contractAddress,
            data: result.encoded_call.data,
            value: 0,
            nonce: nonce,
            gasLimit: 500000, // You may want to estimate this properly
            gasPrice: gasPrice.gasPrice,
            chainId: chainId,
          }
          
          // Sign and send transaction
          console.log('[DiapIdentityPanel] 签名并广播交易...', tx)
          const txResponse = await wallet.sendTransaction(tx)
          console.log('[DiapIdentityPanel] 交易已发送:', txResponse.hash)
          
          setTxHash(txResponse.hash)
          setToastMessage(`交易已广播: ${txResponse.hash}`)
          
          // Wait for transaction to be mined
          const receipt = await txResponse.wait()
          console.log('[DiapIdentityPanel] 交易已确认:', receipt)
          
          setToastMessage(`交易已确认: ${txResponse.hash}`)
          
          // Reload identity to check registration status
          await loadIdentity()
        } catch (signErr) {
          console.error('Failed to sign and broadcast transaction:', signErr)
          setError(`签名/广播失败: ${signErr.message}`)
          setToastMessage(`签名/广播失败: ${signErr.message}`)
          // Still show the encoded call so user can manually sign
          setRegisterInfo(result)
        }
      } else {
        // Just show the encoded call
        setRegisterInfo(result)
      }
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

            {txHash && (
              <div className="diap-field register-info">
                <label>交易哈希</label>
                <div className="diap-value">
                  <p className="info-text success">
                    ✓ 交易已广播并确认
                  </p>
                  <div className="encoded-call">
                    <code>{txHash}</code>
                    <button
                      type="button"
                      className="copy-btn"
                      onClick={() => copyToClipboard(txHash)}
                      title={t('agent.diap.copy')}
                    >
                      📋
                    </button>
                  </div>
                </div>
              </div>
            )}
            
            {registerInfo && !txHash && (
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

