import React from 'react'
import { UIResourceRenderer } from '@mcp-ui/client'
import CloseIcon from '@/assets/关闭0.3.png'
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
            <img src={CloseIcon} alt="关闭" />
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
