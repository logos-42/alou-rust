import React, { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { ethers } from 'ethers'
import asyncDiapCreationService, { DiapCreationProgress, DiapIdentityData } from '@/services/asyncDiapCreationService'
import agentService from '@/services/agentService'
import diapIntegrationService from '@/services/diapIntegrationService'
import useAgentStore from '@/stores/agentStore'
import { useI18n } from '@/hooks/useI18n'
import {
  saveDiapIdentityToFile,
  loadDiapIdentityFromFile,
  hasDiapIdentityFile,
} from '@/utils/diapAgentIdentityManager'
import { DiapIdentity } from '@/utils/diapIdentityManager'
import CloseIcon from '@/assets/关闭0.3.png'
import CopyIcon from '@/assets/复制.png'
import './DiapIdentityPanel.css'

// Props 类型定义
interface DiapIdentityPanelProps {
  sessionId: string
  selectedAgent: {
    id: string
    ipns?: string
    cid?: string
    did?: string
    [key: string]: any
  } | null
  onClose?: () => void
  isDarkMode?: boolean
}

interface RegisterInfo {
  encoded_call?: {
    data: string
  }
  [key: string]: any
}

const DiapIdentityPanel: React.FC<DiapIdentityPanelProps> = ({ 
  sessionId, 
  selectedAgent, 
  onClose, 
  isDarkMode = false 
}) => {
  const { t } = useI18n()
  const navigate = useNavigate()
  const [identity, setIdentity] = useState<DiapIdentity | null>(null)
  const [loading, setLoading] = useState(true)
  const [creating, setCreating] = useState(false)
  const [registering, setRegistering] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [registerInfo, setRegisterInfo] = useState<RegisterInfo | null>(null)
  const [toastMessage, setToastMessage] = useState<string | null>(null)
  const [txHash, setTxHash] = useState<string | null>(null)
  const [hasTestnetKey, setHasTestnetKey] = useState(false)
  const [diapProgress, setDiapProgress] = useState<DiapCreationProgress | null>(null)

  const updateAgent = useAgentStore((state) => state.updateAgent)

  const handleSubscribe = () => {
    navigate('/subscription')
  }

  // loadIdentity 必须在 useEffect 之前定义
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

      // 确定用于加载的 agent ID（只使用智能体 ID）
      const agentId = selectedAgent?.id
      
      if (!agentId) {
        console.log('[DiapIdentityPanel] 没有 agentId，无法加载')
        setLoading(false)
        return
      }

      console.log('[DiapIdentityPanel] 使用 agentId 加载:', agentId)

      // 优先级 1: 从 agent 文件加载
      console.log('[DiapIdentityPanel] 检查 agent 文件存储...')
      try {
        const agentIdentity = await loadDiapIdentityFromFile(agentId)
        if (agentIdentity) {
          console.log('[DiapIdentityPanel] ✅ 从 agent 文件加载成功:', agentId, agentIdentity.did)
          setIdentity(agentIdentity)

          if (selectedAgent?.id) {
            updateAgent(selectedAgent.id, {
              ipns: agentIdentity.ipns,
              cid: agentIdentity.cid,
              did: agentIdentity.did
            })
          }
          setLoading(false)
          console.log('[DiapIdentityPanel] ========== 加载完成（agent 文件）==========')
          return
        } else {
          console.log('[DiapIdentityPanel] 文件中没有 DIAP 身份:', agentId)
        }
      } catch (parseErr) {
        console.debug('[DiapIdentityPanel] agentId 加载失败:', agentId, parseErr)
      }
      
      console.log('[DiapIdentityPanel] ❌ agentId 未找到身份')

      // 未找到任何身份，保持当前状态
      console.log('[DiapIdentityPanel] ========== 加载完成（无身份，保持当前状态）==========')
      setLoading(false)
    } catch (err) {
      console.error('[DiapIdentityPanel] 加载 DIAP 身份失败:', err)
      setError(err instanceof Error ? err.message : '加载失败')
      setLoading(false)
    }
  }, [sessionId, selectedAgent, updateAgent])

  // Check if testnet private key is available
  useEffect(() => {
    const checkTestnetKey = async () => {
      try {
        await invoke('get_testnet_private_key')
        setHasTestnetKey(true)
      } catch {
        setHasTestnetKey(false)
      }
    }
    checkTestnetKey()
  }, [])

  // 组件初始化时加载身份
  useEffect(() => {
    if (sessionId && !identity && !loading) {
      console.log('[DiapIdentityPanel] 组件初始化，开始加载身份')
      loadIdentity()
    }
  }, [])

  // 当 selectedAgent 变化且有 cid 时，直接加载该 cid 的身份
  useEffect(() => {
    if (selectedAgent?.cid && !identity) {
      console.log('[DiapIdentityPanel] selectedAgent 有 cid，尝试加载:', selectedAgent.cid)
      const loadFromCid = async () => {
        try {
          const agentIdentity = await loadDiapIdentityFromFile(selectedAgent.cid)
          if (agentIdentity) {
            console.log('[DiapIdentityPanel] ✅ 从 CID 加载成功:', selectedAgent.cid, agentIdentity.did)
            setIdentity(agentIdentity)
            setLoading(false)
            return
          }
        } catch (err) {
          console.debug('[DiapIdentityPanel] 从 CID 加载失败:', selectedAgent.cid, err)
        }
      }
      loadFromCid()
    }
  }, [selectedAgent?.cid])

  // Load identity when sessionId changes
  useEffect(() => {
    if (sessionId && !identity) {
      console.log('[DiapIdentityPanel] sessionId 变化且无身份，开始加载')
      loadIdentity()
    } else if (sessionId && identity) {
      console.log('[DiapIdentityPanel] sessionId 变化但已有身份，跳过加载:', identity.did)
    }
  }, [sessionId])

  // 监听 DIAP 身份创建事件
  useEffect(() => {
    const handleDiapIdentityCreated = (event: CustomEvent) => {
      const { sessionId: createdSessionId, identity: eventIdentity } = event.detail
      console.log('[DiapIdentityPanel] 收到 DIAP 身份创建事件:', { createdSessionId, currentSessionId: sessionId })

      if (createdSessionId === sessionId) {
        console.log('[DiapIdentityPanel] 匹配当前 sessionId，立即刷新显示')
        if (eventIdentity) {
          setIdentity(eventIdentity)
          setLoading(false)
        } else {
          loadIdentity()
        }
      }
    }

    window.addEventListener('diap-identity-created', handleDiapIdentityCreated as EventListener)

    return () => {
      window.removeEventListener('diap-identity-created', handleDiapIdentityCreated as EventListener)
    }
  }, [sessionId, loadIdentity])

  // 检查异步 DIAP 创建进度和完成状态
  useEffect(() => {
    if (!sessionId) return

    const unsubscribeProgress = asyncDiapCreationService.subscribeProgress((progressSessionId: string, progress: DiapCreationProgress) => {
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
      
      console.log('[DiapIdentityPanel] DIAP 创建响应:', response)
      
      if (response.success && response.cid) {
        // 将扁平的响应转换为 identity 对象
        const newIdentity: DiapIdentity = {
          did: response.did || '',
          cid: response.cid || '',
          ipns: response.ipns || '',
          public_key: response.public_key || '',
          did_document: response.did_document,
          zkp_proof: response.zkp_proof,
          is_registered: false,
          created_at: Math.floor(Date.now() / 1000)
        }
        
        console.log('[DiapIdentityPanel] DIAP 身份创建成功:', newIdentity)
        
        // 使用智能体的 agentId 保存（优先使用 selectedAgent.id）
        const agentId = selectedAgent?.id
        
        if (!agentId) {
          console.error('[DiapIdentityPanel] 无法获取 agentId，无法保存 DIAP 身份')
          setError('无法获取智能体 ID')
          setCreating(false)
          return
        }
        
        // 保存到 agent 文件存储
        await saveDiapIdentityToFile(agentId, newIdentity)
        console.log('[DiapIdentityPanel] DIAP 身份已保存到 agent 文件:', agentId)

        // 立即设置身份显示
        setIdentity(newIdentity)
        setLoading(false)
        console.log('[DiapIdentityPanel] 身份已设置到状态:', newIdentity.did)

        // 更新 agentStore（确保 cid 被保存）
        const agentIdToUpdate = selectedAgent?.id || sessionId
        if (agentIdToUpdate) {
          updateAgent(agentIdToUpdate, {
            ipns: newIdentity.ipns,
            cid: newIdentity.cid,  // 关键：保存 CID
            did: newIdentity.did,
            sessionId: sessionId,
          })
          console.log('[DiapIdentityPanel] ✅ DIAP 身份引用已保存到智能体元数据:', agentIdToUpdate, 'cid:', newIdentity.cid)
        }

        setToastMessage(t('agent.diap.createSuccess'))
      } else {
        console.error('[DiapIdentityPanel] DIAP 创建失败:', response.error)
        setError(response.error || '创建失败')
      }
    } catch (err) {
      console.error('Failed to create DIAP identity:', err)
      const message = err instanceof Error ? err.message : t('agent.diap.createFailed')
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

      const result = await (agentService as any).registerAgentOnChain(
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
          const privateKey = await invoke<string>('get_testnet_private_key')

          let rpcUrl: string, chainId: number
          if (network === 'base_sepolia') {
            rpcUrl = 'https://sepolia.base.org'
            chainId = 84532
          } else if (network === 'sepolia') {
            rpcUrl = 'https://ethereum-sepolia-rpc.publicnode.com'
            chainId = 11155111
          } else {
            throw new Error(`不支持的测试网络：${network}`)
          }

          let contractAddress: string
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
          setError(`签名/广播失败：${signErr instanceof Error ? signErr.message : '未知错误'}`)
          setToastMessage(`签名/广播失败：${signErr instanceof Error ? signErr.message : '未知错误'}`)
          setRegisterInfo(result)
        }
      } else {
        setRegisterInfo(result)
      }
    } catch (err) {
      console.error('Failed to register agent on-chain:', err)
      const message = err instanceof Error ? err.message : t('agent.diap.registerFailed')
      setError(message)
      setToastMessage(message)
    } finally {
      setRegistering(false)
    }
  }

  const copyToClipboard = (text: string) => {
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
