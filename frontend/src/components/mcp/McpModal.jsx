import React from 'react'
import { UIResourceRenderer } from '@mcp-ui/client'
import './McpModal.css'

const McpModal = ({ resource, onClose, onUIAction }) => {
  if (!resource) {
    return null
  }

  const embeddedResource = resource.resource ? resource : { resource }

  return (
    <div className="mcp-modal-overlay" onClick={onClose}>
      <div className="mcp-modal-content" onClick={(event) => event.stopPropagation()}>
        <div className="mcp-modal-header">
          <div className="mcp-modal-title">
            <span>🧩</span>
            <span>MCP UI 内容</span>
          </div>
          <button type="button" className="mcp-modal-close" onClick={onClose}>
            ✕
          </button>
        </div>
        <div className="mcp-modal-body">
          <UIResourceRenderer
            resource={embeddedResource.resource}
            onUIAction={onUIAction}
            supportedContentTypes={['rawHtml', 'remoteDom', 'externalUrl']}
          />
        </div>
      </div>
    </div>
  )
}

export default McpModal

