import { useState, useCallback, useRef, useEffect } from 'react'
import agentService from '@/services/agentService'
import { useI18n } from '@/hooks/useI18n'
import CloseIcon from '@/assets/关闭0.3.png'
import './ImportAgentModal.css'

function ImportAgentModal({ isOpen, onClose, onResolve, onImportAgent }) {
  const { t } = useI18n()
  const [existingAgentTarget, setExistingAgentTarget] = useState('')
  const [resolvedAgent, setResolvedAgent] = useState(null)
  const [showImportConfirm, setShowImportConfirm] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  const isCancelledRef = useRef(false) // 用于跟踪是否被取消
  const isMountedRef = useRef(true) // 用于跟踪组件是否仍然挂载

  const handleResolveExisting = async () => {
    const target = existingAgentTarget.trim()
    if (!target) {
      setError(t('agent.import.error.enterIdentifier') || '请输入 IPNS / CID / DID 标识')
      return
    }
    setError(null)
    setResolvedAgent(null)
    setShowImportConfirm(false)
    setIsLoading(true)
    isCancelledRef.current = false // 重置取消标志
    
    try {
      // 使用 agentService 从网络加载智能体（照搬三天前版本的解析逻辑）
      console.log('[ImportAgentModal] 开始解析智能体:', target)
      const result = await agentService.loadAgentFromNetwork(target)
      
      // 检查是否已被取消或组件已卸载
      if (isCancelledRef.current || !isMountedRef.current) {
        console.log('[ImportAgentModal] 解析已被取消或组件已卸载')
        return
      }
      
      if (result.success && result.agent) {
        console.log('[ImportAgentModal] 解析成功:', result.agent)
        // 确保 agent 对象包含 did_document，以便 resolveAgentAvatar 可以正确解析头像
        const agentWithDidDocument = {
          ...result.agent,
          did_document: result.didDocument || result.agent.did_document,
        }
        console.log('[ImportAgentModal] agent 对象（包含 did_document）:', {
          name: agentWithDidDocument.name,
          avatar_cid: agentWithDidDocument.avatar_cid,
          avatar_url: agentWithDidDocument.avatar_url,
          hasDidDocument: !!agentWithDidDocument.did_document,
        })
        // 只有在组件仍然挂载且未被取消时才更新状态
        if (!isCancelledRef.current && isMountedRef.current) {
          setResolvedAgent(agentWithDidDocument)
          setShowImportConfirm(true)
          setIsLoading(false)
        }
      } else {
        // 降级到原有的 onResolve 方法（照搬三天前版本的逻辑）
        if (onResolve) {
          await onResolve(target)
        }
        // 只有在组件仍然挂载且未被取消时才更新状态
        if (!isCancelledRef.current && isMountedRef.current) {
          setIsLoading(false)
          if (!result.success) {
            setError(result.error || t('agent.import.error.resolveFailed') || '解析失败，请检查标识是否正确')
          }
        }
      }
    } catch (err) {
      // 只有在组件仍然挂载且未被取消时才显示错误
      if (!isCancelledRef.current && isMountedRef.current) {
        console.error('[ImportAgentModal] 解析失败:', err)
        setIsLoading(false)
        setError(err?.message || t('agent.import.error.resolveFailed') || '解析失败，请检查标识是否正确')
      }
    }
  }

  const handleConfirmImport = async () => {
    if (!resolvedAgent) return
    
    setIsLoading(true)
    setError(null)
    
    try {
      console.log('[ImportAgentModal] 确认导入智能体:', resolvedAgent)
      
      // 调用父组件的导入方法
      if (onImportAgent) {
        await onImportAgent(resolvedAgent)
      }
      
      // 清理状态并关闭
      setResolvedAgent(null)
      setShowImportConfirm(false)
      setExistingAgentTarget('')
      setIsLoading(false)
      onClose()
    } catch (err) {
      console.error('[ImportAgentModal] 导入失败:', err)
      setError(err?.message || t('agent.import.error.importFailed') || '导入失败')
      setIsLoading(false)
    }
  }

  const handleCancelImport = () => {
    setResolvedAgent(null)
    setShowImportConfirm(false)
    setExistingAgentTarget('')
  }

  const handleClose = () => {
    // 关闭模态框（不中断解析，解析会在后台继续）
    // 注意：不设置 isCancelledRef.current = true，也不设置 setIsLoading(false)
    // 这样解析可以继续在后台执行，但不会更新已关闭的组件状态
    setResolvedAgent(null)
    setShowImportConfirm(false)
    setExistingAgentTarget('')
    setError(null)
    // 不重置 isLoading，让解析继续
    onClose()
  }

  const handleCancel = () => {
    // 中断解析并关闭
    if (isLoading) {
      isCancelledRef.current = true // 标记为已取消
    }
    setResolvedAgent(null)
    setShowImportConfirm(false)
    setExistingAgentTarget('')
    setError(null)
    setIsLoading(false)
    onClose()
  }

  // 当模态框打开/关闭时更新挂载状态
  useEffect(() => {
    if (isOpen) {
      isMountedRef.current = true
      isCancelledRef.current = false
    } else {
      // 模态框关闭时，标记为未挂载，但不中断正在进行的解析
      // 解析完成后会检查 isMountedRef，如果为 false 就不会更新状态
      isMountedRef.current = false
      // 重置状态（但保持 isLoading 不变，让解析继续）
      setResolvedAgent(null)
      setShowImportConfirm(false)
      setExistingAgentTarget('')
      setError(null)
    }
  }, [isOpen])
  
  // 组件卸载时清理
  useEffect(() => {
    return () => {
      isMountedRef.current = false
    }
  }, [])

  if (!isOpen) {
    return null
  }

  return (
    <div className="import-modal-backdrop">
      <div className="import-modal">
        <div className="import-modal__header">
          <div>
            <h2>{t('agent.import.title') || '导入已有智能体'}</h2>
            <p>{t('agent.import.subtitle') || '通过 IPNS / CID / DID 标识导入智能体'}</p>
          </div>
          <button 
            type="button" 
            onClick={handleClose} 
            className="import-modal__close"
          >
            <img src={CloseIcon} alt="关闭" />
          </button>
        </div>

        <div className="import-modal__content">
          <div className="import-modal__field">
            <span>{t('agent.import.label') || '智能体标识 (IPNS/CID/DID)'}</span>
            <div className="import-modal__resolve-row">
              <input
                type="text"
                placeholder={t('agent.import.placeholder') || '输入 IPNS / CID / DID 标识'}
                value={existingAgentTarget}
                onChange={(event) => setExistingAgentTarget(event.target.value)}
                disabled={showImportConfirm}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !showImportConfirm && !isLoading) {
                    handleResolveExisting()
                  }
                }}
              />
              <button 
                type="button" 
                onClick={handleResolveExisting} 
                disabled={showImportConfirm || isLoading}
                className="import-modal__resolve-btn"
              >
                {isLoading && !showImportConfirm ? (t('agent.import.resolving') || '解析中...') : (t('agent.import.button') || '解析')}
              </button>
            </div>
          </div>

          {/* 导入确认预览 */}
          {showImportConfirm && resolvedAgent && (
            <div className="import-modal__preview">
              <div className="import-modal__preview-header">
                <span className="import-modal__preview-icon">✓</span>
                <span>{t('agent.import.success') || '解析成功！确认导入以下智能体？'}</span>
              </div>
              <div className="import-modal__preview-content">
                <div className="import-modal__preview-row">
                  <strong>{t('agent.import.name') || '名称'}</strong>
                  <span>{resolvedAgent.name || resolvedAgent.display_name || t('agent.import.unnamed') || '未命名'}</span>
                </div>
                {resolvedAgent.role_description && (
                  <div className="import-modal__preview-row">
                    <strong>{t('agent.import.description') || '描述'}</strong>
                    <span>{resolvedAgent.role_description}</span>
                  </div>
                )}
                {resolvedAgent.did && (
                  <div className="import-modal__preview-row">
                    <strong>{t('agent.import.did') || 'DID'}</strong>
                    <span className="import-modal__preview-mono">{resolvedAgent.did}</span>
                  </div>
                )}
                {resolvedAgent.ipns && (
                  <div className="import-modal__preview-row">
                    <strong>{t('agent.import.ipns') || 'IPNS'}</strong>
                    <span className="import-modal__preview-mono">{resolvedAgent.ipns}</span>
                  </div>
                )}
                {resolvedAgent.cid && (
                  <div className="import-modal__preview-row">
                    <strong>{t('agent.import.cid') || 'CID'}</strong>
                    <span className="import-modal__preview-mono">{resolvedAgent.cid}</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {error && <div className="import-modal__error">{error}</div>}
        </div>

        <div className="import-modal__actions">
          <button type="button" className="ghost" onClick={handleCancel} disabled={isLoading && showImportConfirm}>
            {t('common.cancel')}
          </button>
          {showImportConfirm && (
            <button 
              type="button" 
              onClick={handleConfirmImport} 
              disabled={isLoading}
              className="import-modal__confirm-btn"
            >
              {isLoading ? (t('agent.import.importing') || '导入中...') : (t('agent.import.confirm') || '确认导入')}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export default ImportAgentModal

