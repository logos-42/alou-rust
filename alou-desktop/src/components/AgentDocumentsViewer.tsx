import { useState } from 'react'
import { Document } from '../services/agentDocumentService'
import './AgentDocumentsViewer.css'

interface AgentDocumentsViewerProps {
  documents: {
    soul?: Document
    identity?: Document
    capabilities?: Document
    constraints?: Document
    tools?: Document
    memory?: Document
    agents?: Document
  }
  onEdit?: (type: string, document: Document) => void
}

interface DocumentTab {
  id: string
  label: string
  icon: string
  description: string
}

const AgentDocumentsViewer: React.FC<AgentDocumentsViewerProps> = ({ documents, onEdit }) => {
  const [activeTab, setActiveTab] = useState<string>('soul')

  const tabs: DocumentTab[] = [
    { id: 'soul', label: 'SOUL.md', icon: '🧠', description: '性格与哲学' },
    { id: 'identity', label: 'IDENTITY.md', icon: '👤', description: '身份信息' },
    { id: 'capabilities', label: 'CAPABILITIES.md', icon: '⚡', description: '能力列表' },
    { id: 'constraints', label: 'CONSTRAINTS.md', icon: '🚫', description: '约束条件' },
    { id: 'tools', label: 'TOOLS.md', icon: '🛠️', description: '工具配置' },
    { id: 'memory', label: 'MEMORY.md', icon: '📝', description: '记忆配置' },
    { id: 'agents', label: 'AGENTS.md', icon: '🏠', description: '工作空间规则' },
  ]

  const activeDocument = documents[activeTab as keyof typeof documents]

  if (!activeDocument) {
    return (
      <div className="documents-viewer documents-viewer--empty">
        <p>暂无文档</p>
      </div>
    )
  }

  const handleTabClick = (tabId: string, hasDocument: boolean): void => {
    if (hasDocument) {
      setActiveTab(tabId)
    }
  }

  const handleEditClick = (): void => {
    if (onEdit && activeDocument) {
      onEdit(activeTab, activeDocument)
    }
  }

  return (
    <div className="documents-viewer">
      <div className="documents-viewer__tabs">
        {tabs.map((tab) => {
          const hasDocument = !!documents[tab.id as keyof typeof documents]
          return (
            <button
              key={tab.id}
              type="button"
              className={`documents-viewer__tab ${activeTab === tab.id ? 'documents-viewer__tab--active' : ''} ${!hasDocument ? 'documents-viewer__tab--disabled' : ''}`}
              onClick={() => handleTabClick(tab.id, hasDocument)}
              disabled={!hasDocument}
              title={tab.description}
            >
              <span className="documents-viewer__tab-icon">{tab.icon}</span>
              <span className="documents-viewer__tab-label">{tab.label}</span>
              {!hasDocument && <span className="documents-viewer__tab-status">✗</span>}
            </button>
          )
        })}
      </div>

      <div className="documents-viewer__content">
        <div className="documents-viewer__header">
          <h3 className="documents-viewer__title">
            <span className="documents-viewer__title-icon">{tabs.find(t => t.id === activeTab)?.icon}</span>
            {tabs.find(t => t.id === activeTab)?.label}
          </h3>
          {activeDocument.cid && (
            <div className="documents-viewer__cid">
              <span className="documents-viewer__cid-label">IPFS CID:</span>
              <code className="documents-viewer__cid-value">{activeDocument.cid.slice(0, 16)}...</code>
            </div>
          )}
          {onEdit && (
            <button
              type="button"
              className="documents-viewer__edit-btn"
              onClick={handleEditClick}
            >
              编辑
            </button>
          )}
        </div>

        <div className="documents-viewer__body">
          <pre className="documents-viewer__markdown">
            {activeDocument.content}
          </pre>
        </div>

        {activeDocument.metadata && (
          <div className="documents-viewer__metadata">
            <details>
              <summary>文档元数据</summary>
              <pre className="documents-viewer__metadata-content">
                {JSON.stringify(activeDocument.metadata, null, 2)}
              </pre>
            </details>
          </div>
        )}
      </div>
    </div>
  )
}

export default AgentDocumentsViewer
