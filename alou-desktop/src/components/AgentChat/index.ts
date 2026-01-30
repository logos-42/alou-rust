// Export all custom hooks
export { useAgentConnection } from './useAgentConnection.jsx';
export { useAgentWallet } from './useAgentWallet';
export { useAgentMessages } from './useAgentMessages';
export { useAgentDrag } from './useAgentDrag';
export { useAgentUI } from './useAgentUI.jsx';
export { useChannelManager } from './useChannelManager';
export { useAgentPersistence } from './useAgentPersistence';

// Export utility functions
export {
  resolveAgentAvatar,
  buildChannelFromAgent,
  extractAgentTarget,
  extractErrorMessage,
  computeAgentProfile,
} from './agentUtils';

// Export constants
export * from './agentConstants';

// Export services
export { default as avatarManager } from './avatarManager';

// Export types
export type { AgentInfo, Message, Channel } from '@shared/types/services';
export type { 
  AgentStatus, 
  MessageRole, 
  ChatMode, 
  ConnectionStatus 
} from './agentConstants';
