import React, { useState, useEffect, useRef, useCallback } from 'react'
import { useI18n } from '@/hooks/useI18n'
import agentService from '@/services/agentService'
import useAgentStore from '@/stores/agentStore'
import avatarManager from '@/components/AgentChat/avatarManager'
import CloseIcon from '@/assets/关闭0.3.png'
import CopyIcon from '@/assets/复制.png'
import './AgentDetailPanel.css'

const fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'

// 简单的防抖函数
const debounce = (func, wait) => {
  let timeout
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout)
      func(...args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}

const resolveAvatar = (agent) => {
  // 同步版本：先尝试从缓存读取，如果没有缓存再返回默认值
  // 异步加载会在 useEffect 中处理
  if (!agent) return fallbackAvatar
  
  // 直接使用 agent 的头像字段
  return agent.avatar_url || agent.avatar || fallbackAvatar
}

const resolveName = (agent) => {
  if (!agent) return 'alou'
  return (
    agent.display_name ||
    agent.name ||
    agent.ipns?.replace(/^\/?ipns\//, '') ||
    agent.did?.split(':').filter(Boolean).slice(-1)[0] ||
    agent.cid ||
    '解析智能体'
  )
}

const resolveRole = (agent) => {
  if (!agent) return 'Web3 Multi-Agent Coordinator'
  return agent.role_description || 'Web3 Multi-Agent Coordinator'
}

const normalizePorts = (agent) => {
  if (!agent) return []
  if (Array.isArray(agent.mcp_ports)) {
    return agent.mcp_ports
  }
  if (agent.mcp_config?.ports) {
    return agent.mcp_config.ports
  }
  return []
}

const AgentDetailPanel = ({ agent, sessionId, onClose, isDarkMode = false, onAgentUpdated }) => {
  const { t } = useI18n()
  const panelRef = useRef(null)
  const [activeTab, setActiveTab] = useState('overview')
  const [identity, setIdentity] = useState(null)
  const [loadingIdentity, setLoadingIdentity] = useState(true)
  const [apiConfig, setApiConfig] = useState(null)
  const [toastMessage, setToastMessage] = useState(null)
  const [localName, setLocalName] = useState('')
  const [localRole, setLocalRole] = useState('')
  const [localAvatar, setLocalAvatar] = useState('')
  const [isSaving, setIsSaving] = useState(false)

  const updateAgent = useAgentStore((state) => state.updateAgent)

  // 点击外部关闭面板
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (panelRef.current && !panelRef.current.contains(event.target)) {
        onClose?.()
      }
    }

    const handleEscape = (event) => {
      if (event.key === 'Escape') {
        onClose?.()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleEscape)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [onClose])

  // 加载 DIAP 身份信息
  useEffect(() => {
    if (sessionId || agent) {
      loadIdentity()
    }
  }, [sessionId, agent])

  // 加载 API 配置
  useEffect(() => {
    loadApiConfig()
  }, [])

  // 初始化本地状态
  useEffect(() => {
    if (agent) {
      setLocalName(resolveName(agent))
      setLocalRole(resolveRole(agent))
      // 异步加载头像
      resolveAvatar(agent).then(setLocalAvatar)
    }
  }, [agent])

  const loadIdentity = async () => {
    try {
      setLoadingIdentity(true)

      // 优先级1: 从 agent 元数据中查找
      if (agent?.diapIdentity) {
        const identity = {
          ...agent.diapIdentity,
          ipns: agent.diapIdentity.ipns || agent.ipns || null,
          cid: agent.diapIdentity.cid || agent.cid || null,
          did: agent.diapIdentity.did || agent.did || null,
        }
        setIdentity(identity)
        setLoadingIdentity(false)
        return
      }

      // 优先级2: 从 IPNS/CID/DID 查找
      const isTempId = (id) => id && typeof id === 'string' && id.startsWith('temp_')
      const agentTarget = agent?.ipns || 
                         (agent?.cid && !isTempId(agent.cid) ? agent.cid : null) ||
                         agent?.did

      if (agentTarget && !isTempId(agentTarget)) {
        try {
          const response = await agentService.getDiapIdentity(agentTarget)
          if (response.identity) {
            const identity = {
              ...response.identity,
              ipns: response.identity.ipns || agent?.ipns || null,
              cid: response.identity.cid || agent?.cid || null,
              did: response.identity.did || agent?.did || null,
            }
            setIdentity(identity)
            if (agent?.id) {
              updateAgent(agent.id, { diapIdentity: identity })
            }
            setLoadingIdentity(false)
            return
          }
        } catch (targetErr) {
          console.log('[AgentDetailPanel] 从 IPNS/CID/DID 加载失败:', targetErr.message)
        }
      }

      // 优先级3: 从网络加载（使用 sessionId）
      if (sessionId) {
        try {
          const response = await agentService.getDiapIdentity(sessionId)
          if (response.identity) {
            setIdentity(response.identity)
            if (agent?.id) {
              updateAgent(agent.id, { diapIdentity: response.identity })
            }
          }
        } catch (networkErr) {
          const is404 = networkErr?.response?.status === 404 || 
                       networkErr?.message?.includes('404') ||
                       networkErr?.message?.includes('not found')
          if (!is404) {
            console.warn('[AgentDetailPanel] 网络加载失败:', networkErr.message)
          }
        }
      }
    } catch (err) {
      console.error('[AgentDetailPanel] 加载身份失败:', err)
    } finally {
      setLoadingIdentity(false)
    }
  }

  const loadApiConfig = () => {
    try {
      const config = {
        apiBaseUrl: import.meta.env.VITE_API_BASE_URL || 
                   (import.meta.env.DEV ? '' : 'https://alou-edge.yuanjieliu65.workers.dev'),
        aiProvider: import.meta.env.AI_PROVIDER || 'deepseek',
        aiModel: import.meta.env.AI_MODEL || 'deepseek-chat',
        ethRpcUrl: import.meta.env.ETH_RPC_URL || 'https://eth.llamarpc.com',
        ethTestnetRpcUrl: import.meta.env.ETH_TESTNET_RPC_URL || 'https://ethereum-sepolia-rpc.publicnode.com',
        solRpcUrl: import.meta.env.SOL_RPC_URL || 'https://api.mainnet-beta.solana.com',
      }

      // 尝试从 localStorage 读取用户配置
      if (typeof window !== 'undefined' && window.localStorage) {
        const storedApiKey = localStorage.getItem('user_api_key')
        const storedProvider = localStorage.getItem('ai_provider')
        const storedModel = localStorage.getItem('ai_model')

        if (storedApiKey) {
          config.apiKey = storedApiKey
        }
        if (storedProvider) {
          config.aiProvider = storedProvider
        }
        if (storedModel) {
          config.aiModel = storedModel
        }
      }

      setApiConfig(config)
    } catch (err) {
      console.error('[AgentDetailPanel] 加载 API 配置失败:', err)
    }
  }

  const copyToClipboard = (text) => {
    if (!text) return
    navigator.clipboard.writeText(text).then(() => {
      setToastMessage('已复制到剪贴板')
      setTimeout(() => setToastMessage(null), 2000)
    }).catch((err) => {
      console.error('复制失败:', err)
      setToastMessage('复制失败')
      setTimeout(() => setToastMessage(null), 2000)
    })
  }

  const formatApiKey = (key) => {
    if (!key) return '未配置'
    if (key.length <= 8) return key
    return `${key.substring(0, 8)}...`
  }

  // 实时保存函数
  const saveChanges = async (field, value) => {
    try {
      if (!agent?.id) {
        console.warn('[AgentDetailPanel] 无法保存：智能体ID不存在')
        return false
      }

      if (isSaving) {
        console.log('[AgentDetailPanel] 正在保存中，跳过重复保存')
        return false
      }

      setIsSaving(true)
      
      let updatedAgent = null
      
      if (field === 'name') {
        // 更新名称 - 使用头像管理模块确保同步
        const currentAvatar = agent.avatar || agent.avatar_url
        updatedAgent = await avatarManager.updateAvatar(agent.id, currentAvatar, value)
      } else if (field === 'role') {
        // 更新角色
        updatedAgent = updateAgent(agent.id, {
          role_description: value,
          updated_at: Date.now()
        })
      } else if (field === 'avatar') {
        // 使用头像管理模块更新头像
        // 同时传递当前名称，确保同步更新
        const currentName = agent.display_name || agent.name
        updatedAgent = await avatarManager.updateAvatar(agent.id, value, currentName)
      }

      console.log('[AgentDetailPanel] 保存更改:', { 
        agentId: agent.id, 
        field, 
        value,
        updatedAgent
      })
      
      if (updatedAgent) {
        setToastMessage('已保存')
        console.log('[AgentDetailPanel] 保存成功，更新后的头像:', updatedAgent.avatar)
        
        // 通知父组件更新
        if (onAgentUpdated) {
          onAgentUpdated(updatedAgent)
        }
        
        return true
      } else {
        setToastMessage('保存失败')
        console.warn('[AgentDetailPanel] 保存失败')
        return false
      }
    } catch (error) {
      console.error('[AgentDetailPanel] 保存失败:', error)
      setToastMessage('保存失败：' + (error.message || '未知错误'))
      return false
    } finally {
      setIsSaving(false)
    }
  }

  // 处理名称变化（带防抖）
  const handleNameChange = useCallback(
    debounce(async (newName) => {
      if (newName && newName !== resolveName(agent)) {
        await saveChanges('name', newName)
      }
    }, 1000),
    [agent, saveChanges]
  )

  // 处理角色变化（带防抖）
  const handleRoleChange = useCallback(
    debounce(async (newRole) => {
      if (newRole && newRole !== resolveRole(agent)) {
        await saveChanges('role', newRole)
      }
    }, 1000),
    [agent, saveChanges]
  )

  // 处理头像变化
  const handleAvatarChange = async (newAvatar) => {
    // 使用当前 localAvatar 状态进行比较，而不是异步解析
    if (newAvatar && newAvatar !== localAvatar) {
      await saveChanges('avatar', newAvatar)
    }
  }

  const handleFileUpload = async (e) => {
    const file = e.target.files?.[0]
    if (!file) return

    try {
      // 使用头像管理模块处理文件上传
      const newAvatar = await avatarManager.processFileUpload(file)
      setLocalAvatar(newAvatar)
      await handleAvatarChange(newAvatar)
    } catch (error) {
      setToastMessage(error.message)
    }
  }

  const avatar = resolveAvatar(agent)
  const name = resolveName(agent)
  const role = resolveRole(agent)
  const ports = normalizePorts(agent)

  const tabs = [
    { id: 'overview', label: '概览' },
    { id: 'diap', label: 'DIAP 身份' },
    { id: 'api', label: 'API 配置' },
    { id: 'mcp', label: 'MCP 端口' },
  ]

  return (
    <div className={`agent-detail-panel-overlay ${isDarkMode ? 'dark' : 'light'}`}>
      <div ref={panelRef} className={`agent-detail-panel ${isDarkMode ? 'dark' : 'light'}`}>
        <div className="agent-detail-panel-header">
          <h3>智能体详情</h3>
          {onClose && (
            <button type="button" className="close-btn" onClick={onClose}>
              <img src={CloseIcon} alt="关闭" />
            </button>
          )}
        </div>

        <div className="agent-detail-panel-tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              type="button"
              className={`tab-btn ${activeTab === tab.id ? 'active' : ''}`}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="agent-detail-panel-content">
          {activeTab === 'overview' && (
            <div className="tab-content">
              <div className="agent-overview">
                <div className="agent-avatar-large">
                  <img src={localAvatar} alt={localName} />
                  <div className="avatar-edit-overlay">
                    <label htmlFor="avatar-upload" className="avatar-upload-btn">
                      更换头像
                    </label>
                    <input
                      id="avatar-upload"
                      type="file"
                      accept="image/*"
                      onChange={handleFileUpload}
                      style={{ display: 'none' }}
                    />
                  </div>
                </div>
                <div className="agent-info">
                  <div className="edit-field">
                    <label htmlFor="agent-name">名称</label>
                    <input
                      id="agent-name"
                      type="text"
                      value={localName}
                      onChange={(e) => {
                        const newName = e.target.value
                        setLocalName(newName)
                        handleNameChange(newName)
                      }}
                      placeholder="输入智能体名称"
                      className="edit-input"
                      disabled={isSaving}
                    />
                    {isSaving && <div className="saving-indicator">保存中...</div>}
                  </div>
                  <div className="edit-field">
                    <label htmlFor="agent-role">角色设计</label>
                    <textarea
                      id="agent-role"
                      value={localRole}
                      onChange={(e) => {
                        const newRole = e.target.value
                        setLocalRole(newRole)
                        handleRoleChange(newRole)
                      }}
                      placeholder="描述智能体的角色和功能"
                      className="edit-textarea"
                      rows={4}
                      disabled={isSaving}
                    />
                    {isSaving && <div className="saving-indicator">保存中...</div>}
                  </div>
                </div>
              </div>
              
              <div className="hint-text">
                修改信息会自动保存
              </div>
            </div>
          )}

          {activeTab === 'diap' && (
            <div className="tab-content">
              {loadingIdentity ? (
                <div className="loading-state">加载中...</div>
              ) : !identity ? (
                <div className="empty-state">
                  <p>未找到 DIAP 身份信息</p>
                </div>
              ) : (
                <div className="diap-info">
                  <div className="info-field">
                    <label>IPNS</label>
                    <div className="info-value-row">
                      <code>{identity.ipns || agent?.ipns || 'N/A'}</code>
                      {(identity.ipns || agent?.ipns) && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(identity.ipns || agent?.ipns || '')}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>CID</label>
                    <div className="info-value-row">
                      <code>{identity.cid || 'N/A'}</code>
                      {identity.cid && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(identity.cid)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>DID</label>
                    <div className="info-value-row">
                      <code>{identity.did || 'N/A'}</code>
                      {identity.did && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(identity.did)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>注册状态</label>
                    <div className="info-value-row">
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
                        <span className="status-unregistered">未注册</span>
                      )}
                    </div>
                  </div>

                  {identity.created_at && (
                    <div className="info-field">
                      <label>创建时间</label>
                      <div className="info-value-row">
                        <span>{new Date(identity.created_at * 1000).toLocaleString()}</span>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {activeTab === 'api' && (
            <div className="tab-content">
              {!apiConfig ? (
                <div className="loading-state">加载中...</div>
              ) : (
                <div className="api-config">
                  <div className="info-field">
                    <label>AI API Key</label>
                    <div className="info-value-row">
                      <code>{formatApiKey(apiConfig.apiKey)}</code>
                      {apiConfig.apiKey && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(apiConfig.apiKey)}
                          title="复制完整 API Key"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                    <small className="field-hint">API Key 仅显示前8个字符</small>
                  </div>

                  <div className="info-field">
                    <label>AI Provider</label>
                    <div className="info-value-row">
                      <code>{apiConfig.aiProvider || 'N/A'}</code>
                    </div>
                  </div>

                  <div className="info-field">
                    <label>AI Model</label>
                    <div className="info-value-row">
                      <code>{apiConfig.aiModel || 'N/A'}</code>
                    </div>
                  </div>

                  <div className="info-field">
                    <label>API Base URL</label>
                    <div className="info-value-row">
                      <code>{apiConfig.apiBaseUrl || 'N/A'}</code>
                      {apiConfig.apiBaseUrl && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(apiConfig.apiBaseUrl)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>ETH RPC URL</label>
                    <div className="info-value-row">
                      <code>{apiConfig.ethRpcUrl || 'N/A'}</code>
                      {apiConfig.ethRpcUrl && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(apiConfig.ethRpcUrl)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>ETH Testnet RPC URL</label>
                    <div className="info-value-row">
                      <code>{apiConfig.ethTestnetRpcUrl || 'N/A'}</code>
                      {apiConfig.ethTestnetRpcUrl && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(apiConfig.ethTestnetRpcUrl)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="info-field">
                    <label>SOL RPC URL</label>
                    <div className="info-value-row">
                      <code>{apiConfig.solRpcUrl || 'N/A'}</code>
                      {apiConfig.solRpcUrl && (
                        <button
                          type="button"
                          className="copy-btn"
                          onClick={() => copyToClipboard(apiConfig.solRpcUrl)}
                          title="复制"
                        >
                          <img src={CopyIcon} alt="复制" />
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'mcp' && (
            <div className="tab-content">
              {ports.length === 0 ? (
                <div className="empty-state">
                  <p>未配置 MCP 端口</p>
                </div>
              ) : (
                <div className="mcp-ports">
                  {ports.map((port, index) => (
                    <div key={`${port.label || 'port'}-${index}`} className="mcp-port-item">
                      <div className="port-header">
                        <strong>{port.label || `端口 ${index + 1}`}</strong>
                        {port.description && <small>{port.description}</small>}
                      </div>
                      <div className="port-endpoint">
                        <code>{port.endpoint || '未配置 Endpoint'}</code>
                        {port.endpoint && (
                          <button
                            type="button"
                            className="copy-btn"
                            onClick={() => copyToClipboard(port.endpoint)}
                            title="复制"
                          >
                            <img src={CopyIcon} alt="复制" />
                          </button>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {toastMessage && (
          <div className="toast-message" role="status" aria-live="polite">
            {toastMessage}
          </div>
        )}
      </div>
    </div>
  )
}

export default AgentDetailPanel

