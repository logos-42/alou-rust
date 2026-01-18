/**
 * DiapGroupChatExample - DIAP群聊使用示例
 * 展示如何在现有组件中集成DIAP群聊功能
 */

import React, { useState, useCallback } from 'react'
import DiapGroupChatManager from './DiapGroupChatManager'
import { useAgentInvite } from '../AgentChat/useAgentInvite'
import { useAuthStore } from '@/stores/authStore'
import './DiapGroupChatExample.css'

const DiapGroupChatExample = () => {
  const [showManager, setShowManager] = useState(false)
  const [showInviteModal, setShowInviteModal] = useState(false)
  
  // 获取本地身份
  const { user, localIdentity } = useAuthStore()
  
  // 智能体邀请Hook
  const {
    isInviteModalOpen,
    inviteTargetChannel,
    diapGroupChat,
    handleInviteToChannel,
    closeInviteModal,
    handleInviteSubmit
  } = useAgentInvite({
    localIdentity,
    recordInteraction: (type, data) => {
      console.log('[Example] 交互记录:', type, data)
    }
  })

  // 示例频道数据
  const exampleChannels = [
    {
      id: 'channel1',
      name: '开发团队',
      avatar: '',
      mode: 'agent',
      meta: { did: 'did:example:channel1' }
    },
    {
      id: 'channel2', 
      name: '产品设计',
      avatar: '',
      mode: 'agent',
      meta: { did: 'did:example:channel2' }
    }
  ]

  // 示例智能体数据
  const exampleAgents = [
    {
      id: 'claude',
      name: 'Claude',
      display_name: 'Claude Assistant',
      did: 'did:example:claude',
      avatar: '',
      mode: 'agent'
    },
    {
      id: 'gpt4',
      name: 'GPT-4',
      display_name: 'GPT-4 Assistant', 
      did: 'did:example:gpt4',
      avatar: '',
      mode: 'agent'
    },
    {
      id: 'gemini',
      name: 'Gemini',
      display_name: 'Gemini Pro',
      did: 'did:example:gemini',
      avatar: '',
      mode: 'agent'
    }
  ]

  // 处理群聊创建
  const handleGroupCreated = useCallback((group, agents) => {
    console.log('[Example] 群聊创建成功:', {
      groupId: group.groupId,
      groupName: group.groupName,
      agentsCount: agents.length
    })
    
    // 可以在这里添加额外的处理逻辑
    // 比如发送通知、更新UI状态等
  }, [])

  // 处理消息接收
  const handleMessage = useCallback((groupId, message) => {
    console.log('[Example] 收到消息:', {
      groupId,
      from: message.fromName,
      content: message.content,
      type: message.type
    })
    
    // 可以在这里处理消息显示、通知等
  }, [])

  // 处理智能体邀请
  const handleInviteAgents = useCallback((channel) => {
    console.log('[Example] 邀请智能体到频道:', channel.name)
    
    // 模拟选择所有智能体
    handleInviteSubmit(channel, exampleAgents, 'example')
  }, [handleInviteSubmit, exampleAgents])

  return (
    <div className="diap-group-chat-example">
      <div className="example-header">
        <h2>DIAP群聊功能示例</h2>
        <p>展示如何使用DIAP SDK创建和管理群聊</p>
      </div>

      {/* 状态信息 */}
      <div className="status-section">
        <h3>当前状态</h3>
        <div className="status-grid">
          <div className="status-item">
            <label>本地身份:</label>
            <span>{localIdentity ? localIdentity.did : '未设置'}</span>
          </div>
          <div className="status-item">
            <label>用户:</label>
            <span>{user?.name || '未登录'}</span>
          </div>
          <div className="status-item">
            <label>DIAP群聊状态:</label>
            <span className={`status-${diapGroupChat.status}`}>
              {diapGroupChat.status}
            </span>
          </div>
          <div className="status-item">
            <label>活跃群聊:</label>
            <span>{diapGroupChat.activeGroupId || '无'}</span>
          </div>
        </div>
      </div>

      {/* 操作按钮 */}
      <div className="actions-section">
        <h3>操作示例</h3>
        <div className="action-buttons">
          <button 
            className="action-btn primary"
            onClick={() => setShowManager(true)}
            disabled={!localIdentity}
          >
            打开群聊管理器
          </button>
          
          <button 
            className="action-btn secondary"
            onClick={() => setShowInviteModal(true)}
            disabled={!localIdentity}
          >
            智能体邀请示例
          </button>
        </div>
        
        {!localIdentity && (
          <p className="warning-text">
            ⚠️ 需要设置本地身份才能使用DIAP群聊功能
          </p>
        )}
      </div>

      {/* 频道列表 */}
      <div className="channels-section">
        <h3>示例频道</h3>
        <div className="channels-list">
          {exampleChannels.map(channel => (
            <div key={channel.id} className="channel-item">
              <div className="channel-info">
                <h4>{channel.name}</h4>
                <p>{channel.meta.did}</p>
              </div>
              <div className="channel-actions">
                <button 
                  className="invite-btn"
                  onClick={() => handleInviteAgents(channel)}
                >
                  邀请所有智能体
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 智能体列表 */}
      <div className="agents-section">
        <h3>可用智能体</h3>
        <div className="agents-grid">
          {exampleAgents.map(agent => (
            <div key={agent.id} className="agent-card">
              <div className="agent-avatar">
                {agent.avatar ? (
                  <img src={agent.avatar} alt={agent.name} />
                ) : (
                  <div className="avatar-placeholder">
                    {agent.name.charAt(0)}
                  </div>
                )}
              </div>
              <div className="agent-info">
                <h4>{agent.name}</h4>
                <p>{agent.display_name}</p>
                <small>{agent.did}</small>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* DIAP群聊管理器 */}
      <DiapGroupChatManager
        visible={showManager}
        onClose={() => setShowManager(false)}
        localIdentity={localIdentity}
        onGroupCreated={handleGroupCreated}
        onMessage={handleMessage}
      />

      {/* 智能体邀请模态框示例 */}
      {showInviteModal && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3>智能体邀请示例</h3>
              <button onClick={() => setShowInviteModal(false)}>×</button>
            </div>
            <div className="modal-body">
              <p>选择一个频道来邀请智能体：</p>
              <div className="channel-selection">
                {exampleChannels.map(channel => (
                  <button
                    key={channel.id}
                    className="channel-option"
                    onClick={() => {
                      handleInviteAgents(channel)
                      setShowInviteModal(false)
                    }}
                  >
                    {channel.name}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default DiapGroupChatExample
