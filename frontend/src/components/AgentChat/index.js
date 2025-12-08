// Export all custom hooks
export { useAgentConnection } from './useAgentConnection'
export { useAgentWallet } from './useAgentWallet'
export { useAgentMessages } from './useAgentMessages'
export { useAgentDrag } from './useAgentDrag'
export { useAgentUI } from './useAgentUI'
export { useChannelManager } from './useChannelManager'
export { useAgentPersistence } from './useAgentPersistence'

// Export utility functions
export {
  resolveAgentAvatar,
  buildChannelFromAgent,
  extractAgentTarget,
  extractErrorMessage,
  computeAgentProfile,
} from './agentUtils'

// Export constants if needed
export * from './agentConstants'
