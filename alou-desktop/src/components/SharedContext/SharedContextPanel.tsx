/**
 * SharedContextPanel - 共享上下文管理面板
 *
 * 提供智囊团上下文管理、记忆共享和 Session 管理的 UI
 */

import React, { useState, useEffect } from 'react'
import { useSharedContext, useSwarmContext } from '../../hooks/useSharedContext'
import {
  SwarmContext,
  SharedSession,
  ContextEntry,
  AgentInfo,
  ContextEntryType,
} from '../../types/sharedContext'
import { GroupChatMessage } from '../../types/groupchat'

interface SharedContextPanelProps {
  currentAgent?: AgentInfo
  className?: string
}

const SharedContextPanel: React.FC<SharedContextPanelProps> = ({
  currentAgent,
  className = '',
}) => {
  const {
    swarms,
    sessions,
    stats,
    loading,
    createSwarm,
    deleteSwarm,
    addMember,
    createSession,
    joinSession,
    leaveSession,
    syncSession,
    shareMemory,
    createSummary,
    getSwarmMemories,
  } = useSharedContext()

  const [activeTab, setActiveTab] = useState<'swarms' | 'sessions' | 'memory'>('swarms')
  const [selectedSwarm, setSelectedSwarm] = useState<SwarmContext | null>(null)
  const [newSwarmName, setNewSwarmName] = useState('')
  const [newSessionName, setNewSessionName] = useState('')
  const [memoryContent, setMemoryContent] = useState('')
  const [memoryTags, setMemoryTags] = useState('')

  const handleCreateSwarm = async () => {
    if (!newSwarmName.trim() || !currentAgent) return

    const result = await createSwarm(
      newSwarmName,
      [currentAgent],
      `智囊团 ${newSwarmName} 的描述`
    )

    if (result.success && result.data) {
      setSelectedSwarm(result.data)
      setNewSwarmName('')
    }
  }

  const handleCreateSession = async () => {
    if (!newSessionName.trim() || !currentAgent) return

    const swarmIds = selectedSwarm ? [selectedSwarm.id] : []
    await createSession(newSessionName, currentAgent, swarmIds)
    setNewSessionName('')
  }

  const handleShareMemory = async () => {
    if (!selectedSwarm || !memoryContent.trim() || !currentAgent) return

    const tags = memoryTags
      .split(',')
      .map(t => t.trim())
      .filter(t => t.length > 0)

    await shareMemory(selectedSwarm.id, memoryContent, currentAgent, tags)
    setMemoryContent('')
    setMemoryTags('')
  }

  const swarmMemories = selectedSwarm ? getSwarmMemories(selectedSwarm.id) : []

  return (
    <div className={`shared-context-panel ${className}`}>
      <div className="panel-header">
        <h3>共享上下文</h3>
        <div className="stats">
          <span>智囊团: {stats.swarmCount}</span>
          <span>记忆: {stats.sharedEntries}</span>
          <span>IPFS: {stats.ipfsEntries}</span>
        </div>
      </div>

      <div className="tab-nav">
        <button
          className={activeTab === 'swarms' ? 'active' : ''}
          onClick={() => setActiveTab('swarms')}
        >
          智囊团
        </button>
        <button
          className={activeTab === 'sessions' ? 'active' : ''}
          onClick={() => setActiveTab('sessions')}
        >
          会话
        </button>
        <button
          className={activeTab === 'memory' ? 'active' : ''}
          onClick={() => setActiveTab('memory')}
        >
          记忆
        </button>
      </div>

      <div className="panel-content">
        {activeTab === 'swarms' && (
          <div className="swarms-tab">
            <div className="create-form">
              <input
                type="text"
                placeholder="新智囊团名称"
                value={newSwarmName}
                onChange={e => setNewSwarmName(e.target.value)}
              />
              <button onClick={handleCreateSwarm} disabled={loading}>
                创建
              </button>
            </div>

            <div className="swarms-list">
              {swarms.map(swarm => (
                <div
                  key={swarm.id}
                  className={`swarm-item ${selectedSwarm?.id === swarm.id ? 'selected' : ''}`}
                  onClick={() => setSelectedSwarm(swarm)}
                >
                  <div className="swarm-name">{swarm.name}</div>
                  <div className="swarm-meta">
                    {swarm.members.length} 成员 |{' '}
                    {swarm.sharedMemories.length} 记忆
                  </div>
                </div>
              ))}
            </div>

            {selectedSwarm && (
              <div className="swarm-detail">
                <h4>{selectedSwarm.name}</h4>
                <p>{selectedSwarm.description}</p>
                <div className="members">
                  <h5>成员</h5>
                  {selectedSwarm.members.map(member => (
                    <div key={member.id} className="member">
                      {member.name}
                    </div>
                  ))}
                </div>
                <button
                  className="delete-btn"
                  onClick={() => deleteSwarm(selectedSwarm.id)}
                >
                  删除智囊团
                </button>
              </div>
            )}
          </div>
        )}

        {activeTab === 'sessions' && (
          <div className="sessions-tab">
            <div className="create-form">
              <input
                type="text"
                placeholder="新会话名称"
                value={newSessionName}
                onChange={e => setNewSessionName(e.target.value)}
              />
              <button onClick={handleCreateSession} disabled={loading}>
                创建
              </button>
            </div>

            <div className="sessions-list">
              {sessions.map(session => (
                <div key={session.id} className="session-item">
                  <div className="session-name">{session.name}</div>
                  <div className="session-meta">
                    {session.state.active ? '活跃' : '非活跃'} |{' '}
                    {session.state.participants.length} 参与者
                  </div>
                  <div className="session-actions">
                    {session.state.active && currentAgent && (
                      <button
                        onClick={() => leaveSession(session.id, currentAgent.id)}
                      >
                        离开
                      </button>
                    )}
                    <button onClick={() => syncSession(session.id)}>
                      同步
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'memory' && (
          <div className="memory-tab">
            {selectedSwarm ? (
              <>
                <div className="create-form">
                  <textarea
                    placeholder="分享记忆内容..."
                    value={memoryContent}
                    onChange={e => setMemoryContent(e.target.value)}
                  />
                  <input
                    type="text"
                    placeholder="标签 (逗号分隔)"
                    value={memoryTags}
                    onChange={e => setMemoryTags(e.target.value)}
                  />
                  <button onClick={handleShareMemory} disabled={loading}>
                    分享记忆
                  </button>
                </div>

                <div className="memories-list">
                  <h4>{selectedSwarm.name} 的共享记忆</h4>
                  {swarmMemories.map(memory => (
                    <div key={memory.id} className="memory-item">
                      <div className="memory-content">{memory.content.text}</div>
                      <div className="memory-meta">
                        <span className="memory-creator">
                          {memory.creator.name}
                        </span>
                        <span className="memory-time">
                          {new Date(memory.createdAt).toLocaleString()}
                        </span>
                        <div className="memory-tags">
                          {memory.tags.map(tag => (
                            <span key={tag} className="tag">
                              {tag}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <div className="no-swarm">
                请先选择一个智囊团来分享记忆
              </div>
            )}
          </div>
        )}
      </div>

      {loading && <div className="loading-overlay">加载中...</div>}
    </div>
  )
}

export default SharedContextPanel