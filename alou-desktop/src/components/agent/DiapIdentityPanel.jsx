import React, { useState, useEffect, useCallback } from 'react'
import asyncDiapCreationService from '@/services/asyncDiapCreationService'
import { useNavigate } from 'react-router-dom'
import agentService from '@/services/agentService'
import diapIntegrationService from '@/services/diapIntegrationService'
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
  const [diapProgress, setDiapProgress] = useState(null) // DIAP 创建进度

  const updateAgent = useAgentStore((state) => state.updateAgent)

  const handleSubscribe = () => {
    navigate('/subscription')
  }

  // loadIdentity 必须在 useEffect 之前定义，避免暂时性死区
  const loadIdentity = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)

      console.log('[DiapIdentityPanel] ========== 开始加载 DIAP 身份 ==========')
      console.log('[DiapIdentityPanel] sessionId:', sessionId)
      console.log('[DiapIdentityPanel] selectedAgent:', selectedAgent ? {
        id: selectedAgent.id,
        ipns: selectedAgent.ipns,
        cid: selectedAgent.cid,
        did: selectedAgent.did,
      } : null)

      // 优先级 1: 从统一内存存储加载（使用 sessionId）
      console.log('[DiapIdentityPanel] 检查统一内存存储...')
      if (await hasDiapIdentitySafe(sessionId)) {
        try {
          const storedIdentity = await getDiapIdentitySafe(sessionId)
          console.log('[DiapIdentityPanel] ✅ 从统一内存存储加载成功:', storedIdentity)
          setIdentity(storedIdentity)

          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, {
              ipns: storedIdentity.ipns,
              cid: storedIdentity.cid,
              did: storedIdentity.did
            })
          }
          setLoading(false)
          console.log('[DiapIdentityPanel] ========== 加载完成（统一存储）==========')
          return
        } catch (parseErr) {
          console.error('[DiapIdentityPanel] 解析统一存储中的 DIAP 身份失败:', parseErr)
        }
      } else {
        console.log('[DiapIdentityPanel] 统一内存存储中没有此 sessionId 的身份')
      }

      // 优先级 2: 从 memoryStorage 加载（向后兼容）
      console.log('[DiapIdentityPanel] 检查 memoryStorage...')
      if (hasDiapIdentity(sessionId)) {
        try {
          const identity = getDiapIdentity(sessionId)
          console.log('[DiapIdentityPanel] ✅ 从 memoryStorage 加载成功:', identity)
          setIdentity(identity)

          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, {
              ipns: identity.ipns,
              cid: identity.cid,
              did: identity.did
            })
          }
          setLoading(false)
          console.log('[DiapIdentityPanel] ========== 加载完成（memoryStorage）==========')
          return
        } catch (parseErr) {
          console.error('[DiapIdentityPanel] 解析 memoryStorage 中的 DIAP 身份失败:', parseErr)
          removeDiapIdentity(sessionId)
        }
      } else {
        console.log('[DiapIdentityPanel] memoryStorage 中没有此 sessionId 的身份')
      }

      // 如果没有身份，显示创建按钮
      console.log('[DiapIdentityPanel] ❌ 未找到任何 DIAP 身份，需要创建')
      setIdentity(null)
      setLoading(false)
      console.log('[DiapIdentityPanel] ========== 加载完成（无身份）==========')
    } catch (error) {
      console.error('[DiapIdentityPanel] 加载 DIAP 身份失败:', error)
      setError(error.message)
      setLoading(false)
    }
  }, [sessionId, selectedAgent, updateAgent])

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

  // Load identity when sessionId changes
  useEffect(() => {
    if (sessionId) {
      loadIdentity()
    }
  }, [sessionId, loadIdentity])

  // 监听 DIAP 身份创建事件，立即刷新显示
  useEffect(() => {
    const handleDiapIdentityCreated = (event) => {
      const { sessionId: createdSessionId, identity } = event.detail
      console.log('[DiapIdentityPanel] 收到 DIAP 身份创建事件:', { createdSessionId, currentSessionId: sessionId })

      if (createdSessionId === sessionId) {
        console.log('[DiapIdentityPanel] 匹配当前 sessionId，立即刷新显示')
        loadIdentity()
      }
    }

    window.addEventListener('diap-identity-created', handleDiapIdentityCreated)

    return () => {
      window.removeEventListener('diap-identity-created', handleDiapIdentityCreated)
    }
  }, [sessionId, loadIdentity])

  // 检查异步 DIAP 创建进度和完成状态
  useEffect(() => {
    if (!sessionId) return

    const unsubscribeProgress = asyncDiapCreationService.subscribeProgress((progressSessionId, progress) => {
      console.log('[DiapIdentityPanel] 收到进度更新:', { progressSessionId, stage: progress.stage, progress: progress.progress })
      if (progressSessionId === sessionId) {
        setDiapProgress(progress)
        if (progress.stage === 'completed') {
          console.log('[DiapIdentityPanel] DIAP 创建完成，刷新显示')
          loadIdentity()
        }
      }
    })

    const checkExistingTask = async () => {
      const task = asyncDiapCreationService.getTaskStatus(sessionId)
      if (task) {
        console.log('[DiapIdentityPanel] 发现 DIAP 任务:', { status: task.status, stage: task.progress.stage })
        if (task.status === 'running' || task.status === 'pending' || task.status === 'completed') {
          setDiapProgress(task.progress)
          if (task.status === 'completed' && !identity) {
            console.log('[DiapIdentityPanel] 任务已完成但未显示，刷新')
            await loadIdentity()
          }
        }
      }

      if (!task && !identity) {
        console.log('[DiapIdentityPanel] 没有进行中的任务，检查是否有已保存的 DIAP 身份')
        const diapIdentity = await asyncDiapCreationService.getDiapIdentity(sessionId)
        if (diapIdentity) {
          console.log('[DiapIdentityPanel] 发现已保存的 DIAP 身份:', diapIdentity.did)
          loadIdentity()
        }
      }
    }

    checkExistingTask()

    return () => {
      unsubscribeProgress()
    }
  }, [sessionId, loadIdentity, identity])

  const handleCreateIdentity = async () => {
    try {
      setCreating(true)
      setError(null)
      const response = await diapIntegrationService.createDiapIdentity(sessionId)
      if (response.identity) {
        console.log('[DiapIdentityPanel] DIAP 身份创建成功:', response.identity)
        setIdentity(response.identity)

        await setDiapIdentitySafe(sessionId, response.identity)
        console.log('[DiapIdentityPanel] DIAP 身份已保存到统一内存存储:', sessionId)

        const agentIdToUpdate = selectedAgent?.id ||
                                (response.identity.ipns ? response.identity.ipns.replace(/^\/?ipns\//, '') : null) ||
                                response.identity.cid ||
                                response.identity.did

        if (agentIdToUpdate) {
          updateAgent(agentIdToUpdate, {
            ipns: response.identity.ipns,
            cid: response.identity.cid,
            did: response.identity.did,
            sessionId: sessionId,
          })
          console.log('[DiapIdentityPanel] DIAP 身份引用已保存到智能体元数据:', agentIdToUpdate)
        }

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

    const useAa = window.confirm('是否使用 ERC-4337 智能体 AA 账户？\n\n点击"确定"使用 AA 账户\n点击"取消"使用传统 EOA')

    try {
      setRegistering(true)
      setError(null)
      setTxHash(null)

      const result = await agentService.registerAgentOnChain(
        identity,
        network,
        stakeAmount,
        useAa,
        0,
      )

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
          const privateKey = await invoke('get_testnet_private_key')

          let rpcUrl, chainId
          if (network === 'base_sepolia') {
            rpcUrl = 'https://sepolia.base.org'
            chainId = 84532
          } else if (network === 'sepolia') {
            rpcUrl = 'https://ethereum-sepolia-rpc.publicnode.com'
            chainId = 11155111
          } else {
            throw new Error(`不支持的测试网络：${network}`)
          }

          let contractAddress
          if (network === 'base_sepolia') {
            contractAddress = '0xA960cf9053FA76278e16f9D4BA35225f7634DC54'
          } else if (network === 'sepolia') {
            contractAddress = '0x9eF71FD5be68ebab2ABE20c5Fab826b14BfBc089'
          } else {
            throw new Error(`不支持的测试网络：${network}`)
          }

          const provider = new ethers.JsonRpcProvider(rpcUrl)
          const wallet = new ethers.Wallet(privateKey, provider)

          const nonce = await provider.getTransactionCount(wallet.address, 'latest')
          const gasPrice = await provider.getFeeData()

          const tx = {
            to: contractAddress,
            data: result.encoded_call.data,
            value: 0,
            nonce: nonce,
            gasLimit: 500000,
            gasPrice: gasPrice.gasPrice,
            chainId: chainId,
          }

          console.log('[DiapIdentityPanel] 签名并广播交易...', tx)
          const txResponse = await wallet.sendTransaction(tx)
          console.log('[DiapIdentityPanel] 交易已发送:', txResponse.hash)

          setTxHash(txResponse.hash)
          setToastMessage(`交易已广播：${txResponse.hash}`)

          const receipt = await txResponse.wait()
          console.log('[DiapIdentityPanel] 交易已确认:', receipt)

          setToastMessage(`交易已确认：${txResponse.hash}`)

          await loadIdentity()
        } catch (signErr) {
          console.error('Failed to sign and broadcast transaction:', signErr)
          setError(`签名/广播失败：${signErr.message}`)
          setToastMessage(`签名/广播失败：${signErr.message}`)
          setRegisterInfo(result)
        }
      } else {
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
        {/* 创建按钮 - 始终显示在顶部 */}
        <div className="diap-create-section">
          {!identity ? (
            <button
              type="button"
              className="create-identity-btn prominent"
              onClick={handleCreateIdentity}
              disabled={creating}
            >
              {creating ? (
                <>
                  <span className="spinner"></span>
                  {t('agent.diap.creating')}
                </>
              ) : (
                <>
                  <span className="icon">⚡</span>
                  {t('agent.diap.create')}
                </>
              )}
            </button>
          ) : (
            <div className="diap-status-badge success">
              <span className="checkmark">✓</span>
              {t('agent.diap.hasIdentity') || '身份已创建'}
            </div>
          )}
        </div>

        {/* 进度显示 */}
        {diapProgress && diapProgress.stage !== 'idle' && diapProgress.stage !== 'completed' && (
          <div className="diap-creation-progress">
            <div className="diap-progress-header">
              <span className="diap-progress-title">
                {diapProgress.stage === 'failed' ? '⚠️ 创建失败' : '⏳ 创建 DIAP 身份中...'}
              </span>
              <span className="diap-progress-percent">{diapProgress.progress}%</span>
            </div>
            <div className="diap-progress-bar">
              <div
                className={`diap-progress-fill ${diapProgress.stage === 'failed' ? 'failed' : ''}`}
                style={{ width: `${diapProgress.progress}%` }}
              />
            </div>
            <div className="diap-progress-message">
              {diapProgress.message}
              {diapProgress.error && (
                <span className="diap-progress-error"> - {diapProgress.error}</span>
              )}
            </div>
          </div>
        )}

        {/* 身份信息 - 当有身份时显示 */}
        {identity && (
          <div className="diap-identity-info">
            <div className="diap-field">
              <label>{t('agent.diap.ipns')}</label>
              <div className="diap-value">
                <code>{(identity.ipns || selectedAgent?.ipns) || 'N/A'}</code>
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
                <code>{(identity.cid || selectedAgent?.cid) || 'N/A'}</code>
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
                <code>{(identity.did || selectedAgent?.did) || 'N/A'}</code>
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
                  <code className="tx-hash">{txHash}</code>
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
