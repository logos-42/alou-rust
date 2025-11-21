import agentAssetsService from '@/services/agentAssetsService'
import './AgentProfilePanel.css'

const fallbackAvatar = 'https://avatars.githubusercontent.com/u/16309930?v=4'

const resolveAvatar = (agent) => {
  if (!agent) return fallbackAvatar
  if (agent.avatar_url) return agent.avatar_url
  if (agent.avatar || agent.avatarCid || agent.avatar_cid) {
    return agent.avatar || agentAssetsService.resolveIpfsUri(agent.avatarCid || agent.avatar_cid)
  }
  return fallbackAvatar
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

function AgentProfilePanel({ agent, onInspect }) {
  const avatar = resolveAvatar(agent)
  const name = resolveName(agent)
  const ports = normalizePorts(agent)

  return (
    <div className="agent-profile-panel" onClick={onInspect}>
      <div className="agent-profile-panel__header">
        <img src={avatar} alt={name} />
        <div>
          <h3>{name}</h3>
        </div>
      </div>
      {ports.length > 0 && (
        <div className="agent-profile-panel__ports">
          <span>MCP 端口</span>
          <ul>
            {ports.map((port, index) => (
              <li key={`${port.label || 'port'}-${index}`}>
                <div>
                  <strong>{port.label || `端口 ${index + 1}`}</strong>
                  <small>{port.description || '未填写描述'}</small>
                </div>
                <code>{port.endpoint || '未配置 Endpoint'}</code>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}

export default AgentProfilePanel

