import React, { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import { invoke } from '@tauri-apps/api/core'
import { ethers } from 'ethers'
import { 
  setDiapIdentitySafe, 
  getDiapIdentitySafe, 
  hasDiapIdentitySafe 
} from '@/utils/diapIdentityManager'
import { 
  setDiapIdentity, 
  getDiapIdentity, 
  removeDiapIdentity,
  hasDiapIdentity 
} from '@/utils/memoryStorage'
import CloseIcon from '@/assets/关闭0.3.png'
import CopyIcon from '@/assets/复制.png'
import './DiapIdentityPanel.css'

const DiapIdentityPanel = ({ sessionId, selectedAgent, onClose, isDarkMode = false }) => {
  const { t } = useI18n()
  const navigate = useNavigate()
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
  
  const handleSubscribe = () => {
    navigate('/subscription')
  }
  
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
  }, [sessionId]) // 移除 selectedAgent 依赖，避免循环依赖

  // 监听DIAP身份创建事件，立即刷新显示
  useEffect(() => {
    const handleDiapIdentityCreated = (event) => {
      const { sessionId: createdSessionId, identity } = event.detail
      console.log('[DiapIdentityPanel] 收到DIAP身份创建事件:', { createdSessionId, currentSessionId: sessionId })
      
      if (createdSessionId === sessionId) {
        console.log('[DiapIdentityPanel] 匹配当前sessionId，立即刷新显示')
        // 立即刷新显示
        loadIdentity()
      }
    }

    // 添加事件监听
    window.addEventListener('diap-identity-created', handleDiapIdentityCreated)
    
    // 清理事件监听
    return () => {
      window.removeEventListener('diap-identity-created', handleDiapIdentityCreated)
    }
  }, [sessionId]) // 移除 loadIdentity 依赖，避免循环依赖

  const loadIdentity = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      
      console.log('[DiapIdentityPanel] 开始加载DIAP身份，sessionId:', sessionId)
      console.log('[DiapIdentityPanel] selectedAgent信息:', {
        hasSelectedAgent: !!selectedAgent,
        agentId: selectedAgent?.id,
        diapIdentity: !!selectedAgent?.diapIdentity,
        ipns: selectedAgent?.ipns,
        cid: selectedAgent?.cid,
        did: selectedAgent?.did,
      })
      
      // 优先级1: 从统一内存存储加载（使用 sessionId）
      if (await hasDiapIdentitySafe(sessionId)) {
        try {
          const storedIdentity = await getDiapIdentitySafe(sessionId)
          console.log('[DiapIdentityPanel] ✅ 从统一内存存储加载 DIAP 身份:', sessionId)
          console.log('[DiapIdentityPanel] 身份详情:', storedIdentity)
          setIdentity(storedIdentity)
          
          // 如果智能体存在，更新其元数据（但不重复存储DIAP身份）
          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, { 
              // 只更新引用信息，不存储完整的DIAP身份
              ipns: storedIdentity.ipns,
              cid: storedIdentity.cid,
              did: storedIdentity.did
            })
          }
          setLoading(false)
          return
        } catch (parseErr) {
          console.error('[DiapIdentityPanel] 解析统一存储中的DIAP身份失败:', parseErr)
          // 继续尝试其他方法
        }
      }
      
      // 优先级2: 从 memoryStorage 加载（向后兼容）
      if (hasDiapIdentity(sessionId)) {
        try {
          const identity = getDiapIdentity(sessionId)
          console.log('[DiapIdentityPanel] ✅ 从 memoryStorage 加载 DIAP 身份:', identity)
          setIdentity(identity)
          
          // 如果智能体存在，更新其元数据
          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, { 
              ipns: identity.ipns,
              cid: identity.cid,
              did: identity.did
            })
          }
          setLoading(false)
          return
        } catch (parseErr) {
          console.error('[DiapIdentityPanel] 解析 memoryStorage 中的 DIAP 身份失败:', parseErr)
          removeDiapIdentity(sessionId)
        }
      }
      
      // 优先级3: 从 IPNS/CID/DID 查找（如果智能体有这些标识）
      const isTempId = (id) => id && typeof id === 'string' && id.startsWith('temp_')
      const agentTarget = selectedAgent?.ipns || 
                         (selectedAgent?.cid && !isTempId(selectedAgent.cid) ? selectedAgent.cid : null) ||
                         selectedAgent?.did
      
      if (agentTarget && !isTempId(agentTarget)) {
        try {
          console.log('[DiapIdentityPanel] 尝试从 IPNS/CID/DID 加载 DIAP 身份:', agentTarget)
          const response = await agentService.getDiapIdentity(agentTarget)
          if (response.identity) {
            console.log('[DiapIdentityPanel] ✅ 从 IPNS/CID/DID 加载 DIAP 身份成功:', agentTarget)
            
            // 同步到统一存储系统
            await setDiapIdentitySafe(sessionId, response.identity)
            
            const identity = {
              ...response.identity,
              ipns: response.identity.ipns || selectedAgent?.ipns || null,
              cid: response.identity.cid || selectedAgent?.cid || null,
              did: response.identity.did || selectedAgent?.did || null,
            }
            console.log('[DiapIdentityPanel] 补充后的身份信息:', identity)
            setIdentity(identity)
            
            // 更新智能体元数据（仅引用信息）
            if (selectedAgent?.id) {
              updateAgent(selectedAgent.id, {
                ipns: identity.ipns,
                cid: identity.cid,
                did: identity.did
              })
            }
            setLoading(false)
            return
          }
        } catch (targetErr) {
          console.log('[DiapIdentityPanel] 从 IPNS/CID/DID 加载失败:', targetErr.message)
        }
      } else if (selectedAgent?.cid && isTempId(selectedAgent.cid)) {
        console.log('[DiapIdentityPanel] 跳过临时ID，尝试其他方式加载')
      }
      
      // 优先级4: 从旧的 localStorage 迁移数据（兼容性）
      if (typeof window !== 'undefined' && window.localStorage) {
        const oldStoredIdentity = localStorage.getItem(`diap_identity_${sessionId}`)
        if (oldStoredIdentity) {
          try {
            const identity = JSON.parse(oldStoredIdentity)
            console.log('[DiapIdentityPanel] 从 localStorage 迁移 DIAP 身份到统一存储:', sessionId)
            
            // 迁移到新的统一存储系统
            await setDiapIdentitySafe(sessionId, identity)
            // 同时保存到 memoryStorage
            setDiapIdentity(sessionId, identity)
            
            // 清理旧的 localStorage 数据
            localStorage.removeItem(`diap_identity_${sessionId}`)
            
            setIdentity(identity)
            // 更新智能体元数据（仅引用信息）
            if (selectedAgent?.id) {
              updateAgent(selectedAgent.id, {
                ipns: identity.ipns,
                cid: identity.cid,
                did: identity.did
              })
            }
            setLoading(false)
            return
          } catch (parseErr) {
            console.warn('[DiapIdentityPanel] 解析 localStorage 数据失败:', parseErr)
            // 清理损坏的数据
            localStorage.removeItem(`diap_identity_${sessionId}`)
          }
        }
      }
      
      // 优先级5: 从网络加载（使用 sessionId）
      // 注意：404 错误是正常的，表示身份尚未创建，不应该显示错误
      try {
        const response = await agentService.getDiapIdentity(sessionId)
        if (response.identity) {
          setIdentity(response.identity)
          // 同步到统一存储系统
          await setDiapIdentitySafe(sessionId, response.identity)
          console.log('[DiapIdentityPanel] DIAP 身份已同步到统一存储:', sessionId)
          
          // 更新智能体元数据（仅引用信息）
          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, {
              ipns: response.identity.ipns,
              cid: response.identity.cid,
              did: response.identity.did
            })
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
          console.log('[DiapIdentityPanel] DIAP身份尚未创建（正常状态）')
        }
      }
      
    } catch (error) {
      console.error('[DiapIdentityPanel] 加载DIAP身份失败:', error)
      setError('加载身份信息失败，请稍后重试')
    } finally {
      setLoading(false)
    }
  }, [sessionId, selectedAgent, updateAgent])

  const handleCreateIdentity = async () => {
    try {
      setCreating(true)
      setError(null)
      const response = await agentService.createDiapIdentity(sessionId)
      if (response.identity) {
        console.log('[DiapIdentityPanel] DIAP 身份创建成功:', response.identity)
        setIdentity(response.identity)
        
        // 保存到统一内存存储（使用 sessionId）
        await setDiapIdentitySafe(sessionId, response.identity)
        console.log('[DiapIdentityPanel] DIAP 身份已保存到统一内存存储:', sessionId)
        
        // 更新智能体元数据（仅引用信息，不存储完整DIAP身份）
        // 注意：优先使用 sessionId 关联的智能体ID，如果没有则使用身份标识符
        const agentIdToUpdate = selectedAgent?.id || 
                                (response.identity.ipns ? response.identity.ipns.replace(/^\/?ipns\//, '') : null) ||
                                response.identity.cid || 
                                response.identity.did
        
        if (agentIdToUpdate) {
          updateAgent(agentIdToUpdate, { 
            // 只存储引用信息，完整DIAP身份在统一存储系统中
            ipns: response.identity.ipns,
            cid: response.identity.cid,
            did: response.identity.did,
            sessionId: sessionId, // 确保 sessionId 被保存
          })
          console.log('[DiapIdentityPanel] DIAP 身份引用已保存到智能体元数据:', agentIdToUpdate)
        }
        
        // 重新加载身份以刷新显示（确保IPNS正确显示）
        setTimeout(() => {
          loadIdentity()
        }, 100)
        
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

    const stakeAmount = window.prompt('请输入质押金额（单位：wei，例如 100 表示 100 代币）', '100000000000000000000')
    if (!stakeAmount) return

    const useAa = window.confirm('是否使用 ERC-4337智能体 AA 账户？\n\n点击"确定"使用 AA 账户\n点击"取消"使用传统 EOA')

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
              <img src={CloseIcon} alt="关闭" />
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
            <img src={CloseIcon} alt="关闭" />
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
              <label>{t('agent.diap.ipns')}</label>
              <div className="diap-value">
                <code>{identity.ipns || selectedAgent?.ipns || 'N/A'}</code>
                {(identity.ipns || selectedAgent?.ipns) && (
                  <button
                    type="button"
                    className="copy-btn"
                    onClick={() => copyToClipboard(identity.ipns || selectedAgent?.ipns || '')}
                    title={t('agent.diap.copy')}
                  >
                    <img src={CopyIcon} alt="复制" />
                  </button>
                )}
              </div>
            </div>

            <div className="diap-field">
              <label>{t('agent.diap.cid')}</label>
              <div className="diap-value">
                <code>{identity.cid || selectedAgent?.cid || 'N/A'}</code>
                {(identity.cid || selectedAgent?.cid) && (
                  <button
                    type="button"
                    className="copy-btn"
                    onClick={() => copyToClipboard(identity.cid || selectedAgent?.cid || '')}
                    title={t('agent.diap.copy')}
                  >
                    <img src={CopyIcon} alt="复制" />
                  </button>
                )}
              </div>
            </div>

            <div className="diap-field">
              <label>{t('agent.diap.did')}</label>
              <div className="diap-value">
                <code>{identity.did || selectedAgent?.did || 'N/A'}</code>
                {(identity.did || selectedAgent?.did) && (
                  <button
                    type="button"
                    className="copy-btn"
                    onClick={() => copyToClipboard(identity.did || selectedAgent?.did || '')}
                    title={t('agent.diap.copy')}
                  >
                    <img src={CopyIcon} alt="复制" />
                  </button>
                )}
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
                    <div className="register-buttons">
                      <button
                        type="button"
                        className="register-btn"
                        onClick={handleRegisterOnChain}
                        disabled={registering}
                      >
                        {registering ? t('agent.diap.registering') : t('agent.diap.register')}
                      </button>
                    </div>
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
                      <img src={CopyIcon} alt="复制" />
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
                      <img src={CopyIcon} alt="复制" />
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
      
      {/* 右下角注册付费按钮 */}
      {identity && (
        <div className="diap-subscribe-footer">
          <button
            type="button"
            className="diap-subscribe-btn"
            onClick={handleSubscribe}
          >
            {t('agent.diap.upgradePro')}
          </button>
        </div>
      )}
      
      {toastMessage && (
        <div className="diap-toast" role="status" aria-live="polite">
          {toastMessage}
        </div>
      )}
    </div>
  )
}

export default DiapIdentityPanel

