/**
 * 服务层类型定义
 */

// 通用服务响应类型
export interface ServiceResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

// 钱包信息类型
export interface WalletInfo {
  address: string;
  chainId: string;
  walletType: 'walletconnect' | 'local' | 'browser';
  name?: string;
  icon?: string;
}

// 网络信息类型
export interface NetworkInfo {
  chainId: string;
  name: string;
  rpcUrl: string;
  nativeCurrency?: {
    name: string;
    symbol: string;
    decimals: number;
  };
  blockExplorerUrl?: string;
}

// 交易信息类型
export interface TransactionInfo {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed?: string;
  gasPrice?: string;
  status: 'pending' | 'success' | 'failed';
  timestamp: string;
  blockNumber?: number;
}

// 技能配置相关类型
export interface SkillSettings {
  [key: string]: any;
  updatedAt?: string;
}

export interface SkillPreferences {
  autoExecute: boolean;
  showExamples: boolean;
  enableNotifications: boolean;
  defaultCategory: string;
}

export interface CustomSkill {
  name: string;
  description?: string;
  category?: string;
  parameters?: Record<string, any>;
  createdAt?: string;
  updatedAt?: string;
}

export interface AgentSkillsConfig {
  enabledSkills: string[];
  disabledSkills: string[];
  skillSettings: Record<string, SkillSettings>;
  customSkills: CustomSkill[];
  preferences: SkillPreferences;
  createdAt: string;
  updatedAt: string;
  agentId?: string;
}

export interface AllAgentConfigs {
  [agentId: string]: AgentSkillsConfig;
}

// 智能体服务相关类型
export interface Agent {
  id: string;
  name: string;
  description?: string;
  avatar?: string;
  status: 'online' | 'offline' | 'busy';
  capabilities: string[];
  createdAt: string;
  updatedAt: string;
}

export interface AgentMessage {
  id: string;
  agentId: string;
  content: string;
  type: 'text' | 'image' | 'file' | 'system';
  timestamp: string;
  metadata?: Record<string, any>;
}

export interface AgentConnection {
  agentId: string;
  status: 'connecting' | 'connected' | 'disconnected' | 'error';
  lastActivity?: string;
  error?: string;
}

// 区块链服务相关类型
export interface BlockchainWalletInfo {
  address: string;
  chainId: number;
  balance: string;
  symbol: string;
  name?: string;
  walletType?: 'walletconnect' | 'local' | 'browser';
}

export interface BlockchainTransactionInfo {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed?: string;
  gasPrice?: string;
  status: 'pending' | 'failed' | 'confirmed';
  timestamp: string;
  blockNumber?: number;
}

export interface BlockchainNetworkInfo {
  chainId: number;
  name: string;
  rpcUrl: string;
  nativeCurrency?: {
    name: string;
    symbol: string;
    decimals: number;
  };
  blockExplorerUrl?: string;
}

// DIAP 服务相关类型
export interface DIAPIdentity {
  did: string;
  publicKey: string;
  document: any;
  verified: boolean;
  createdAt: string;
}

export interface DIAPDocument {
  '@context': string[];
  id: string;
  type: string[];
  issuer: string;
  issuanceDate: string;
  verificationMethod: string[];
  proof: any;
  [key: string]: any;
}

// 工作流服务相关类型
export interface WorkflowStep {
  id: string;
  name: string;
  type: 'manual' | 'automated';
  description?: string;
  parameters?: Record<string, any>;
  dependencies?: string[];
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: any;
  error?: string;
  startedAt?: string;
  completedAt?: string;
}

export interface Workflow {
  id: string;
  name: string;
  description?: string;
  steps: WorkflowStep[];
  status: 'draft' | 'active' | 'paused' | 'completed' | 'failed';
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
}

// 异步任务服务相关类型
export interface AsyncTask {
  id: string;
  type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  result?: any;
  error?: string;
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
}

export interface TaskProgress {
  taskId: string;
  progress: number;
  message?: string;
  data?: any;
}

// 集群操作服务相关类型
export interface ClusterAction {
  id: string;
  type: string;
  target: string;
  parameters: Record<string, any>;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: any;
  error?: string;
  createdAt: string;
  updatedAt: string;
}

// 图片代理服务相关类型
export interface ImageProxyOptions {
  width?: number;
  height?: number;
  quality?: number;
  format?: 'jpeg' | 'png' | 'webp';
  fit?: 'cover' | 'contain' | 'fill';
}

// DID 文档解析器类型
export interface DIDDocument {
  '@context': string[];
  id: string;
  verificationMethod: Array<{
    id: string;
    type: string;
    controller: string;
    publicKeyBase58?: string;
    publicKeyJwk?: any;
  }>;
  authentication?: string[];
  assertionMethod?: string[];
  keyAgreement?: string[];
  capabilityInvocation?: string[];
  service?: Array<{
    id: string;
    type: string;
    serviceEndpoint: string | { [key: string]: any };
  }>;
}

// IPFS 服务相关类型
export interface IPFSNodeInfo {
  version: string;
  commit?: string;
  repo?: string;
  system?: string;
  golang?: string;
}

export interface IPFSPeerInfo {
  id: string;
  addresses: string[];
  agentVersion?: string;
  protocols?: string[];
}

export interface IPFSUploadResult {
  cid: string;
  size?: number;
  path?: string;
}

export interface IPFSPinResult {
  cid: string;
  pinned: boolean;
  message?: string;
}

// Agent 服务相关扩展类型
export interface AgentSession {
  id: string;
  wallet_address: string;
  created_at: string;
  updated_at: string;
  status: 'active' | 'inactive' | 'expired';
}

export interface SendMessageOptions {
  chain?: string;
  contextEvents?: any[];
  eventSummary?: string;
  useAsync?: boolean;
  timeout?: number;
  mode?: 'agent' | 'alou';
  agentName?: string;
  roleDescription?: string;
  customInstructions?: string;
  customPrompt?: string;
  stream?: boolean;
}

export interface AgentInfo {
  id?: string;
  name?: string;
  display_name?: string;
  description?: string;
  role_description?: string;
  avatar?: string;
  avatar_url?: string;
  avatar_cid?: string;
  avatarCid?: string;
  ipns?: string;
  did?: string;
  cid?: string;
  mode?: 'agent' | 'alou';
  diapIdentity?: {
    did?: string;
    ipns?: string;
    cid?: string;
    avatar_cid?: string;
    public_key?: string;
  };
  serviceEndpoint?: {
    avatar_cid?: string;
    [key: string]: any;
  };
  meta?: AgentInfo;
  did_document?: DIDDocument;
  mcp_ports?: McpPort[];
  mcp_config?: {
    ports?: McpPort[];
  };
  status?: string;
  agent_type?: string;
}

export interface McpPort {
  label?: string;
  endpoint?: string;
  port?: string | number;
  description?: string;
}

export interface Channel {
  id: string;
  name: string;
  status: 'online' | 'offline' | 'creating';
  statusLabel: string;
  icon: string;
  avatar: string;
  color: string;
  updatedAt: number;
  meta: AgentInfo & { mode?: string };
}

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: number;
  sessionId?: string;
  metadata?: {
    tool_calls?: any[];
    tool_results?: any[];
    [key: string]: any;
  };
}

// Prompt 服务相关类型
export interface PromptContext {
  agentName?: string;
  roleDescription?: string;
  customInstructions?: string;
  [key: string]: any;
}

export interface BasePrompts {
  general: string;
  alou: string;
  system: string;
}

// 解析器服务类型
export interface ResolverResult<T = any> {
  success: boolean;
  data?: T;
  source?: 'cache' | 'network' | 'ipfs' | 'did';
  error?: string;
}

// IPFS 内容服务类型
export interface IPFSContent {
  cid: string;
  content: any;
  contentType?: string;
  size?: number;
  timestamp?: string;
}

// 订阅服务类型
export interface SubscriptionPlan {
  id: string;
  name: string;
  description: string;
  price: number;
  currency: string;
  interval: 'month' | 'year';
  features: string[];
}

export interface UserSubscription {
  id: string;
  planId: string;
  status: 'active' | 'canceled' | 'expired' | 'pending';
  currentPeriodStart: string;
  currentPeriodEnd: string;
  cancelAtPeriodEnd: boolean;
}

// 钱包同步服务类型
export interface WalletSyncStatus {
  walletId: string;
  lastSync: string;
  status: 'synced' | 'syncing' | 'error';
  error?: string;
}

// 分页请求类型
export interface ServicePaginationParams {
  page?: number;
  pageSize?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface ServicePaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}
