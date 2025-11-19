import { useMemo, useState } from 'react'
import agentAssetsService from '@/services/agentAssetsService'
import diapService from '@/services/diapService'
import './CreateAgentModal.css'

const emptyPort = () => ({
  label: '',
  endpoint: '',
  port: '',
  description: '',
  protocol: 'http',
})

const MAX_PORTS = 6

function CreateAgentModal({ isOpen, onClose, onSubmit, onResolve, sessionId }) {
  const [name, setName] = useState('')
  const [roleDescription, setRoleDescription] = useState('Web3 多代理协调智能体')
  const [avatarFile, setAvatarFile] = useState(null)
  const [avatarPreview, setAvatarPreview] = useState(null)
  const [mcpPorts, setMcpPorts] = useState([emptyPort()])
  const [existingAgentTarget, setExistingAgentTarget] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)

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
      let avatarCid = null
      if (avatarFile) {
        const uploaded = await agentAssetsService.uploadAvatar(avatarFile, { sessionId })
        avatarCid = uploaded?.cid || null
      }

      const filteredPorts = mcpPorts
        .filter((port) => port.label.trim() || port.endpoint.trim())
        .map((port) => ({
          ...port,
          port: port.port ? Number(port.port) : undefined,
        }))

      let mcpConfigCid = null
      if (filteredPorts.length > 0) {
        const uploadedConfig = await agentAssetsService.uploadMcpConfig(
          {
            ports: filteredPorts,
            generatedAt: Date.now(),
          },
          { sessionId },
        )
        mcpConfigCid = uploadedConfig?.cid || null
      }

      const fallbackName = name.trim() || 'agent'

      const diapIdentity = await diapService.createLocalIdentity({
        name: fallbackName,
        description: roleDescription.trim(),
      })

      await onSubmit({
        name: fallbackName,
        roleDescription: roleDescription.trim() || 'Web3 多代理协调智能体',
        avatarCid,
        mcpConfigCid,
        mcpPorts: filteredPorts,
        diapIdentity,
      })
      setIsLoading(false)
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
    setIsLoading(true)
    try {
      await onResolve(target)
      setIsLoading(false)
    } catch (err) {
      setIsLoading(false)
      setError(err?.message || '解析失败')
    }
  }

  return (
    <div className="agent-modal-backdrop">
      <div className="agent-modal">
        <div className="agent-modal__header">
          <div>
            <h2>创建新的智能体</h2>
            <p>上传头像、配置 MCP 端口，并保存到 IPFS</p>
          </div>
          <button type="button" onClick={onClose} className="agent-modal__close">
            ✕
          </button>
        </div>

        <form className="agent-modal__form" onSubmit={handleInternalSubmit}>
          <div className="agent-modal__field">
            <span>智能体头像（IPFS）</span>
            <div className="agent-modal__avatar-upload">
              {avatarPreview ? (
                <img src={avatarPreview} alt="预览" className="agent-modal__avatar-preview" />
              ) : (
                <div className="agent-modal__avatar-placeholder">预览</div>
              )}
              <label className="agent-modal__upload-button">
                选择图片
                <input type="file" accept="image/*" onChange={handleAvatarChange} />
              </label>
            </div>
          </div>

          <label className="agent-modal__field">
            <span>智能体名称</span>
            <input
              type="text"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="例如：Alou Web3 调度"
            />
          </label>

          <label className="agent-modal__field">
            <span>角色设计说明</span>
            <textarea
              value={roleDescription}
              onChange={(event) => setRoleDescription(event.target.value)}
              rows={4}
              placeholder="描述该智能体的职责、语气与工具使用策略"
            />
          </label>

          <div className="agent-modal__field">
            <div className="agent-modal__field-header">
              <span>MCP 端口配置</span>
              {canAddMorePorts && (
                <button type="button" onClick={handleAddPort}>
                  + 新增端口
                </button>
              )}
            </div>
            <div className="agent-modal__port-list">
              {mcpPorts.map((port, index) => (
                <div key={`port-${index}`} className="agent-modal__port-item">
                  <input
                    type="text"
                    placeholder="端口名称"
                    value={port.label}
                    onChange={(event) => handlePortChange(index, 'label', event.target.value)}
                  />
                  <input
                    type="text"
                    placeholder="Endpoint URL (wss:// 或 http://)"
                    value={port.endpoint}
                    onChange={(event) => handlePortChange(index, 'endpoint', event.target.value)}
                  />
                  <input
                    type="number"
                    placeholder="端口"
                    value={port.port}
                    onChange={(event) => handlePortChange(index, 'port', event.target.value)}
                  />
                  <input
                    type="text"
                    placeholder="描述"
                    value={port.description}
                    onChange={(event) => handlePortChange(index, 'description', event.target.value)}
                  />
                  <button type="button" onClick={() => handleRemovePort(index)}>
                    移除
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="agent-modal__field agent-modal__field--resolve">
            <span>解析已有智能体</span>
            <div className="agent-modal__resolve-row">
              <input
                type="text"
                placeholder="输入 IPNS / CID / DID"
                value={existingAgentTarget}
                onChange={(event) => setExistingAgentTarget(event.target.value)}
              />
              <button type="button" onClick={handleResolveExisting}>
                解析
              </button>
            </div>
          </div>

          {error && <div className="agent-modal__error">{error}</div>}

          <div className="agent-modal__actions">
            <button type="button" className="ghost" onClick={onClose}>
              取消
            </button>
            <button type="submit" disabled={isLoading}>
              {isLoading ? '创建中...' : '创建智能体'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default CreateAgentModal

