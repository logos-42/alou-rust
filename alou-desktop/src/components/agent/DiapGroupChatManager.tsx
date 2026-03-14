/**
 * DiapGroupChatManager - DIAP群聊管理器组件
 * 提供DIAP群聊的创建、管理、状态显示功能
 */

import React, { useState, useCallback } from 'react'
import { useDiapGroupChat, GroupChatStatus } from '@/hooks/useDiapGroupChat'
import { DiapGroupConfig } from '@/services/diapGroupChatService'
import './DiapGroupChatManager.css'

const DiapGroupChatManager = ({
  localIdentity,
  onGroupCreated,
  onMessage,
  visible = false,
  onClose
}) => {
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [groupName, setGroupName] = useState('')
  const [groupDescription, setGroupDescription] = useState('')
  const [selectedAgents, setSelectedAgents] = useState([])
  const [isPublic, setIsPublic] = useState(false)

  // DIAP群聊Hook
  const {
    status,
    error,
    groups,
    activeGroupId,
    isLoading,
    createGroupWithAgents,
    sendMessage,
    leaveGroup,
    switchActiveGroup,
    clearError
  } = useDiapGroupChat({
    localIdentity,
    onGroupCreated,
    onMessage,
    onError: (error) => {
      console.error('[DiapGroupChatManager] 群聊错误:', error)
    }
  })

  // 创建群聊
  const handleCreateGroup = useCallback(async () => {
    if (!groupName.trim()) {
      alert('请输入群聊名称')
      return
    }

    if (selectedAgents.length === 0) {
      alert('请至少选择一个智能体')
      return
    }

    try {
      await createGroupWithAgents({
        groupName: groupName.trim(),
        description: groupDescription.trim(),
        agents: selectedAgents,
        metadata: {
          isPublic,
          type: 'manual_create'
        }
      })

      // 重置表单
      setGroupName('')
      setGroupDescription('')
      setSelectedAgents([])
      setIsPublic(false)
      setIsCreateModalOpen(false)

    } catch (error) {
      alert(`创建群聊失败: ${error.message}`)
    }
  }, [groupName, groupDescription, selectedAgents, isPublic, createGroupWithAgents])

  // 切换智能体选择
  const toggleAgentSelection = useCallback((agent) => {
    setSelectedAgents(prev => {
      const isSelected = prev.some(a => a.id === agent.id)
      if (isSelected) {
        return prev.filter(a => a.id !== agent.id)
      } else {
        return [...prev, agent]
      }
    })
  }, [])

  // 示例智能体列表（实际应该从props获取）
  const availableAgents = [
    { id: 'agent1', name: 'Claude', did: 'did:example:claude', avatar: '' },
    { id: 'agent2', name: 'GPT-4', did: 'did:example:gpt4', avatar: '' },
    { id: 'agent3', name: 'Gemini', did: 'did:example:gemini', avatar: '' },
  ]

  if (!visible) {
    return null
  }

  return (
    <div className="diap-group-chat-manager">
      {/* 头部 */}
      <div className="manager-header">
        <h3>DIAP群聊管理</h3>
        <button className="close-btn" onClick={onClose}>×</button>
      </div>

      {/* 错误显示 */}
      {error && (
        <div className="error-message">
          <span>{error}</span>
          <button onClick={clearError}>清除</button>
        </div>
      )}

      {/* 状态显示 */}
      <div className="status-bar">
        <span className="status-label">状态:</span>
        <span className={`status-value status-${status}`}>
          {getStatusText(status)}
        </span>
        {isLoading && <span className="loading-indicator">处理中...</span>}
      </div>

      {/* 操作按钮 */}
      <div className="action-buttons">
        <button 
          className="create-btn"
          onClick={() => setIsCreateModalOpen(true)}
          disabled={isLoading}
        >
          创建新群聊
        </button>
      </div>

      {/* 群聊列表 */}
      <div className="groups-section">
        <h4>我的群聊 ({groups.length})</h4>
        {groups.length === 0 ? (
          <div className="empty-groups">
            <p>还没有群聊，点击"创建新群聊"开始</p>
          </div>
        ) : (
          <div className="groups-list">
            {groups.map(group => (
              <div 
                key={group.groupId}
                className={`group-item ${activeGroupId === group.groupId ? 'active' : ''}`}
                onClick={() => switchActiveGroup(group.groupId)}
              >
                <div className="group-info">
                  <h5>{group.groupName}</h5>
                  <p>{group.description}</p>
                  <div className="group-meta">
                    <span>{group.members.length} 成员</span>
                    <span>{new Date(group.createdAt).toLocaleString()}</span>
                  </div>
                </div>
                <div className="group-actions">
                  {activeGroupId === group.groupId && (
                    <button 
                      className="leave-btn"
                      onClick={(e) => {
                        e.stopPropagation()
                        leaveGroup(group.groupId)
                      }}
                    >
                      退出
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 创建群聊模态框 */}
      {isCreateModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h4>创建DIAP群聊</h4>
              <button onClick={() => setIsCreateModalOpen(false)}>×</button>
            </div>

            <div className="modal-body">
              <div className="form-group">
                <label>群聊名称 *</label>
                <input
                  type="text"
                  value={groupName}
                  onChange={(e) => setGroupName(e.target.value)}
                  placeholder="输入群聊名称"
                  maxLength={50}
                />
              </div>

              <div className="form-group">
                <label>群聊描述</label>
                <textarea
                  value={groupDescription}
                  onChange={(e) => setGroupDescription(e.target.value)}
                  placeholder="输入群聊描述（可选）"
                  rows={3}
                  maxLength={200}
                />
              </div>

              <div className="form-group">
                <label>选择智能体 *</label>
                <div className="agents-selection">
                  {availableAgents.map(agent => (
                    <div
                      key={agent.id}
                      className={`agent-option ${selectedAgents.some(a => a.id === agent.id) ? 'selected' : ''}`}
                      onClick={() => toggleAgentSelection(agent)}
                    >
                      <div className="agent-avatar">
                        {agent.avatar ? (
                          <img src={agent.avatar} alt={agent.name} />
                        ) : (
                          <div className="avatar-placeholder">{agent.name.charAt(0)}</div>
                        )}
                      </div>
                      <div className="agent-info">
                        <div className="agent-name">{agent.name}</div>
                        <div className="agent-did">{agent.did}</div>
                      </div>
                      <div className="selection-indicator">
                        {selectedAgents.some(a => a.id === agent.id) && '✓'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="form-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={isPublic}
                    onChange={(e) => setIsPublic(e.target.checked)}
                  />
                  <span>公开群聊（任何人都可以加入）</span>
                </label>
              </div>
            </div>

            <div className="modal-footer">
              <button 
                className="cancel-btn"
                onClick={() => setIsCreateModalOpen(false)}
              >
                取消
              </button>
              <button 
                className="confirm-btn"
                onClick={handleCreateGroup}
                disabled={isLoading || !groupName.trim() || selectedAgents.length === 0}
              >
                {isLoading ? '创建中...' : '创建群聊'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

// 获取状态文本
const getStatusText = (status) => {
  switch (status) {
    case GroupChatStatus.IDLE:
      return '空闲'
    case GroupChatStatus.CREATING:
      return '创建中'
    case GroupChatStatus.JOINING:
      return '加入中'
    case GroupChatStatus.ACTIVE:
      return '活跃'
    case GroupChatStatus.LEAVING:
      return '离开中'
    case GroupChatStatus.ERROR:
      return '错误'
    default:
      return '未知'
  }
}

export default DiapGroupChatManager
