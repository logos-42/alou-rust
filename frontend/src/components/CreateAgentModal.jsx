import { useMemo, useState } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import agentService from '@/services/agentService'
import diapService from '@/services/diapService'
import ipfsService from '@/services/ipfsService'
import { useI18n } from '@/hooks/useI18n'
import './CreateAgentModal.css'

const emptyPort = () => ({
  label: '',
  endpoint: '',
  port: '',
  description: '',
  protocol: 'http',
})

const MAX_PORTS = 6

function CreateAgentModal({ isOpen, onClose, onSubmit, onResolve, onImportAgent, sessionId, onEarlyChannel }) {
  const { t } = useI18n()
  const [name, setName] = useState('')
  const [roleDescription, setRoleDescription] = useState(t('agent.create.role.default'))
  const [avatarFile, setAvatarFile] = useState(null)
  const [avatarPreview, setAvatarPreview] = useState(null)
  const [mcpPorts, setMcpPorts] = useState([emptyPort()])
  const [existingAgentTarget, setExistingAgentTarget] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  // 解析预览状态
  const [resolvedAgent, setResolvedAgent] = useState(null)
  const [showImportConfirm, setShowImportConfirm] = useState(false)

  const canAddMorePorts = useMemo(() => mcpPorts.length < MAX_PORTS, [mcpPorts])

  if (!isOpen) {
    return null
  }

  const handleAvatarChange = (event) => {
    const file = event.target.files?.[0]
    if (!file) {
      setAvatarFile(null)
      setAvatarPreview(null)
      return
    }
    setAvatarFile(file)
    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        setAvatarPreview(reader.result)
      }
    }
    reader.readAsDataURL(file)
  }

  const handlePortChange = (index, field, value) => {
    setMcpPorts((prev) =>
      prev.map((port, idx) => {
        if (idx !== index) return port
        return {
          ...port,
          [field]: value,
        }
      }),
    )
  }

  const handleAddPort = () => {
    if (!canAddMorePorts) return
    setMcpPorts((prev) => [...prev, emptyPort()])
  }

  const handleRemovePort = (index) => {
    if (mcpPorts.length === 1) {
      setMcpPorts([emptyPort()])
      return
    }
    setMcpPorts((prev) => prev.filter((_, idx) => idx !== index))
  }

  const handleInternalSubmit = async (event) => {
    event.preventDefault()
    setError(null)
    setIsLoading(true)

    try {
      // 检查 IPFS 节点是否运行
      let isRunning = await ipfsService.isNodeRunning()
      if (!isRunning) {
        // 尝试自动启动节点
        const startResult = await ipfsService.startNode(true)
        if (!startResult.success) {
          setError(t('agent.create.error.ipfsNotRunning'))
          setIsLoading(false)
          return
        }
        // 节点已启动，等待 API 就绪
        console.log('[IPFS] 节点已自动启动，等待 API 就绪...')
      }

      // 等待 IPFS API 完全就绪（最多等待 15 秒）
      const apiReady = await ipfsService.waitForApiReady(15, 1000)
      if (!apiReady.success) {
        setError(
          apiReady.error ||
            'IPFS API 未就绪。请确保 IPFS 节点正常运行，然后重试。'
        )
        setIsLoading(false)
        return
      }
      console.log(`[IPFS] API 已就绪 (尝试 ${apiReady.attempts} 次)`)

      let avatarCid = null
      if (avatarFile) {
        try {
          console.log('[CreateAgentModal] 开始上传头像...')
          const uploaded = await agentAssetsService.uploadAvatar(avatarFile, { sessionId })
          avatarCid = uploaded?.cid || null
          console.log('[CreateAgentModal] 头像上传成功:', avatarCid)
          
          // 头像上传成功后，立即显示频道
          if (avatarCid && onEarlyChannel) {
            const fallbackName = name.trim() || 'agent'
            // 使用 avatar_cid 作为临时标识符，确保后续完整创建时可以匹配到同一个频道
            const earlyMetadata = {
              display_name: fallbackName,
              name: fallbackName,
              role_description: roleDescription.trim() || t('agent.create.role.default'),
              avatar_cid: avatarCid,
              agent_type: 'claude_agent_sdk',
              cid: `temp_${avatarCid}`, // 临时 CID，用于匹配
              status: 'loading', // 标记为加载中
            }
            console.log('[CreateAgentModal] 立即显示频道，头像CID:', avatarCid)
            onEarlyChannel(earlyMetadata)
          }
        } catch (err) {
          console.error('[CreateAgentModal] 头像上传失败:', err)
          setError(
            `上传头像失败: ${err?.message || err?.toString() || '未知错误'}. 请检查 IPFS 节点是否正常运行。`
          )
          setIsLoading(false)
          return
        }
      }

      const fallbackName = name.trim() || 'agent'
      
      // 头像上传成功后，立即提交基本信息（不等待MCP和DIAP）
      await onSubmit({
        name: fallbackName,
        roleDescription: roleDescription.trim() || t('agent.create.role.default'),
        avatarCid,
        mcpConfigCid: null, // 将在后台加载
        mcpPorts: [],
        diapIdentity: null, // 将在后台加载
      })
      
      // 关闭模态框，让用户看到频道
      setIsLoading(false)
      
      // 在后台继续加载 MCP 配置和 DIAP Identity
      void (async () => {
        try {
      const filteredPorts = mcpPorts
        .filter((port) => port.label.trim() || port.endpoint.trim())
        .map((port) => ({
          ...port,
          port: port.port ? Number(port.port) : undefined,
        }))

      let mcpConfigCid = null
      if (filteredPorts.length > 0) {
        try {
              console.log('[CreateAgentModal] 后台开始上传 MCP 配置...')
          const uploadedConfig = await agentAssetsService.uploadMcpConfig(
            {
              ports: filteredPorts,
              generatedAt: Date.now(),
            },
            { sessionId },
          )
          mcpConfigCid = uploadedConfig?.cid || null
          console.log('[CreateAgentModal] MCP 配置上传成功:', mcpConfigCid)
        } catch (err) {
          console.error('[CreateAgentModal] MCP 配置上传失败:', err)
              // 不阻塞，只记录错误
        }
      }

      let diapIdentity
      try {
            console.log('[CreateAgentModal] 后台开始创建 DIAP Identity...')
        diapIdentity = await diapService.createLocalIdentity({
          name: fallbackName,
          description: roleDescription.trim(),
        })
        console.log('[CreateAgentModal] DIAP Identity 创建成功:', diapIdentity?.did)
        
        // 保存 sessionId → IPNS 映射到 localStorage
        if (diapIdentity && sessionId && typeof window !== 'undefined' && window.localStorage) {
          localStorage.setItem(
            `diap_identity_${sessionId}`,
            JSON.stringify({
              ipns: diapIdentity.ipns,
              did: diapIdentity.did,
              cid: diapIdentity.cid,
              created_at: Date.now()
            })
          )
          console.log('[CreateAgentModal] 已保存 identity 映射到 localStorage')
        }
            
            // 更新频道和智能体信息（通过重新调用 onSubmit 或更新现有频道）
      await onSubmit({
        name: fallbackName,
        roleDescription: roleDescription.trim() || t('agent.create.role.default'),
        avatarCid,
        mcpConfigCid,
        mcpPorts: filteredPorts,
        diapIdentity,
      })
          } catch (err) {
            console.error('[CreateAgentModal] DIAP Identity 创建失败:', err)
            // 不阻塞，只记录错误，但至少提交MCP配置
            if (mcpConfigCid) {
              await onSubmit({
                name: fallbackName,
                roleDescription: roleDescription.trim() || t('agent.create.role.default'),
                avatarCid,
                mcpConfigCid,
                mcpPorts: filteredPorts,
                diapIdentity: null,
              })
            }
          }
        } catch (err) {
          console.error('[CreateAgentModal] 后台加载失败:', err)
        }
      })()
    } catch (err) {
      console.error('[CreateAgentModal] 创建智能体失败', err)
      setIsLoading(false)
      setError(err?.message || '创建失败，请稍后重试')
    }
  }

  const handleResolveExisting = async () => {
    const target = existingAgentTarget.trim()
    if (!target) {
      setError('请输入 IPNS / CID / DID 标识')
      return
    }
    setError(null)
    setResolvedAgent(null)
    setShowImportConfirm(false)
    setIsLoading(true)
    
    try {
      // 使用 agentService 从网络加载智能体
      console.log('[CreateAgentModal] 开始解析智能体:', target)
      const result = await agentService.loadAgentFromNetwork(target)
      
      if (result.success && result.agent) {
        console.log('[CreateAgentModal] 解析成功:', result.agent)
        setResolvedAgent(result.agent)
        setShowImportConfirm(true)
        setIsLoading(false)
      } else {
        // 降级到原有的 onResolve 方法
        if (onResolve) {
          await onResolve(target)
        }
        setIsLoading(false)
        if (!result.success) {
          setError(result.error || '解析失败，请检查标识是否正确')
        }
      }
    } catch (err) {
      console.error('[CreateAgentModal] 解析失败:', err)
      setIsLoading(false)
      setError(err?.message || '解析失败，请检查标识是否正确')
    }
  }

  const handleConfirmImport = async () => {
    if (!resolvedAgent) return
    
    setIsLoading(true)
    setError(null)
    
    try {
      console.log('[CreateAgentModal] 确认导入智能体:', resolvedAgent)
      
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
      console.error('[CreateAgentModal] 导入失败:', err)
      setError(err?.message || t('agent.create.error.importFailed'))
      setIsLoading(false)
    }
  }

  const handleCancelImport = () => {
    setResolvedAgent(null)
    setShowImportConfirm(false)
  }

  return (
    <div className="agent-modal-backdrop">
      <div className="agent-modal">
        <div className="agent-modal__header">
          <div>
            <h2>{t('agent.create.title')}</h2>
            <p>{t('agent.create.subtitle')}</p>
          </div>
          <button type="button" onClick={onClose} className="agent-modal__close">
            ✕
          </button>
        </div>

        <form className="agent-modal__form" onSubmit={handleInternalSubmit}>
          <div className="agent-modal__field">
            <span>{t('agent.create.avatar.label')}</span>
            <div className="agent-modal__avatar-upload">
              {avatarPreview ? (
                <img src={avatarPreview} alt={t('agent.create.avatar.preview')} className="agent-modal__avatar-preview" />
              ) : (
                <div className="agent-modal__avatar-placeholder">{t('agent.create.avatar.preview')}</div>
              )}
              <label className="agent-modal__upload-button">
                {t('agent.create.avatar.select')}
                <input type="file" accept="image/*" onChange={handleAvatarChange} />
              </label>
            </div>
          </div>

          <label className="agent-modal__field">
            <span>{t('agent.create.name.label')}</span>
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t('agent.create.name.placeholder')}
            />
          </label>

          <label className="agent-modal__field">
            <span>{t('agent.create.role.label')}</span>
            <textarea
              value={roleDescription}
              onChange={(event) => setRoleDescription(event.target.value)}
              rows={4}
              placeholder={t('agent.create.role.placeholder')}
            />
          </label>

          <div className="agent-modal__field">
            <div className="agent-modal__field-header">
              <span>{t('agent.create.mcp.label')}</span>
              {canAddMorePorts && (
                <button type="button" onClick={handleAddPort}>
                  {t('agent.create.mcp.add')}
                </button>
              )}
            </div>
            <div className="agent-modal__port-list">
              {mcpPorts.map((port, index) => (
                <div key={`port-${index}`} className="agent-modal__port-item">
                  <input
                    type="text"
                    placeholder={t('agent.create.mcp.portName')}
                    value={port.label}
                    onChange={(event) => handlePortChange(index, 'label', event.target.value)}
                  />
                  <input
                    type="text"
                    placeholder={t('agent.create.mcp.endpoint')}
                    value={port.endpoint}
                    onChange={(event) => handlePortChange(index, 'endpoint', event.target.value)}
                  />
                  <input
                    type="number"
                    placeholder={t('agent.create.mcp.port')}
                    value={port.port}
                    onChange={(event) => handlePortChange(index, 'port', event.target.value)}
                  />
                  <input
                    type="text"
                    placeholder={t('agent.create.mcp.description')}
                    value={port.description}
                    onChange={(event) => handlePortChange(index, 'description', event.target.value)}
                  />
                  <button type="button" onClick={() => handleRemovePort(index)}>
                    {t('agent.create.mcp.remove')}
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="agent-modal__field agent-modal__field--resolve">
            <span>{t('agent.create.resolve.label')}</span>
            <div className="agent-modal__resolve-row">
              <input
                type="text"
                placeholder={t('agent.create.resolve.placeholder')}
                value={existingAgentTarget}
                onChange={(event) => setExistingAgentTarget(event.target.value)}
                disabled={showImportConfirm}
              />
              <button type="button" onClick={handleResolveExisting} disabled={showImportConfirm || isLoading}>
                {isLoading && !showImportConfirm ? t('agent.create.resolve.resolving') : t('agent.create.resolve.button')}
              </button>
            </div>
          </div>

          {/* 导入确认预览 */}
          {showImportConfirm && resolvedAgent && (
            <div className="agent-modal__import-preview">
              <div className="agent-modal__import-preview-header">
                <span>{t('agent.create.resolve.success')}</span>
              </div>
              <div className="agent-modal__import-preview-content">
                <div className="agent-modal__import-preview-row">
                  <strong>{t('agent.create.import.name')}</strong>
                  <span>{resolvedAgent.name || resolvedAgent.display_name || t('agent.create.name.unnamed')}</span>
                </div>
                {resolvedAgent.role_description && (
                  <div className="agent-modal__import-preview-row">
                    <strong>{t('agent.create.import.description')}</strong>
                    <span>{resolvedAgent.role_description}</span>
                  </div>
                )}
                {resolvedAgent.did && (
                  <div className="agent-modal__import-preview-row">
                    <strong>{t('agent.create.import.did')}</strong>
                    <span className="agent-modal__import-preview-mono">{resolvedAgent.did}</span>
                  </div>
                )}
                {resolvedAgent.ipns && (
                  <div className="agent-modal__import-preview-row">
                    <strong>{t('agent.create.import.ipns')}</strong>
                    <span className="agent-modal__import-preview-mono">{resolvedAgent.ipns}</span>
                  </div>
                )}
                {resolvedAgent.cid && (
                  <div className="agent-modal__import-preview-row">
                    <strong>{t('agent.create.import.cid')}</strong>
                    <span className="agent-modal__import-preview-mono">{resolvedAgent.cid}</span>
                  </div>
                )}
                {resolvedAgent.pubsub_topics && resolvedAgent.pubsub_topics.length > 0 && (
                  <div className="agent-modal__import-preview-row">
                    <strong>{t('agent.create.import.pubsub')}</strong>
                    <span>{resolvedAgent.pubsub_topics.join(', ')}</span>
                  </div>
                )}
              </div>
              <div className="agent-modal__import-preview-actions">
                <button type="button" className="ghost" onClick={handleCancelImport} disabled={isLoading}>
                  {t('common.cancel')}
                </button>
                <button type="button" onClick={handleConfirmImport} disabled={isLoading}>
                  {isLoading ? t('agent.create.import.importing') : t('agent.create.import.confirm')}
                </button>
              </div>
            </div>
          )}

          {error && <div className="agent-modal__error">{error}</div>}

          <div className="agent-modal__actions">
            <button type="button" className="ghost" onClick={onClose}>
              {t('common.cancel')}
            </button>
            <button type="submit" disabled={isLoading}>
              {isLoading ? t('agent.create.creating') : t('agent.create.submit')}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default CreateAgentModal
