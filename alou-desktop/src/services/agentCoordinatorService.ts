/**
 * Agent Coordinator Service - 智能体协调和自主交流
 * 实现智能体间的消息路由、协作、自动回复、任务分配和状态管理
 */
import pubsubService, { PubSubMessage, MessageType } from './pubsubService';
import agentService from './agentService';

// 智能体能力类型
export const AgentCapability = {
  CHAT: 'chat',                 // 通用对话
  BLOCKCHAIN: 'blockchain',     // 区块链操作
  DEFI: 'defi',                 // DeFi 操作
  NFT: 'nft',                   // NFT 操作
  TRANSLATION: 'translation',   // 翻译
  CODE: 'code',                 // 代码生成
  RESEARCH: 'research',         // 研究分析
  WALLET: 'wallet',             // 钱包管理
} as const;

export type AgentCapabilityType = typeof AgentCapability[keyof typeof AgentCapability];

// 智能体状态
export const AgentStatus = {
  ONLINE: 'online',             // 在线
  OFFLINE: 'offline',           // 离线
  BUSY: 'busy',                 // 忙碌
  IDLE: 'idle',                 // 空闲
  ERROR: 'error',               // 错误状态
} as const;

export type AgentStatusType = typeof AgentStatus[keyof typeof AgentStatus];

// 任务类型
export const TaskType = {
  CHAT: 'chat',                 // 对话任务
  CODE: 'code',                 // 代码任务
  ANALYSIS: 'analysis',         // 分析任务
  TRANSACTION: 'transaction',   // 交易任务
  RESEARCH: 'research',         // 研究任务
  CUSTOM: 'custom',             // 自定义任务
} as const;

export type TaskTypeType = typeof TaskType[keyof typeof TaskType];

// 任务状态
export const TaskStatus = {
  PENDING: 'pending',           // 待处理
  ASSIGNED: 'assigned',         // 已分配
  IN_PROGRESS: 'in_progress',   // 进行中
  COMPLETED: 'completed',       // 已完成
  FAILED: 'failed',             // 失败
  CANCELLED: 'cancelled',       // 已取消
} as const;

export type TaskStatusType = typeof TaskStatus[keyof typeof TaskStatus];

// 意图分析结果
export interface IntentAnalysisResult {
  intent: string;
  confidence: number;
  capabilities: AgentCapabilityType[];
  keywords: string[];
  targetAgent: string | null;
}

// 智能体信息
export interface AgentInfo {
  id: string;
  did?: string;
  ipns?: string;
  name: string;
  description: string;
  capabilities: AgentCapabilityType[];
  pubsubTopics: string[];
  status: AgentStatusType;
  registeredAt: number;
  lastSeenAt: number;
  currentTask?: string | null;
  autoReplyEnabled: boolean;
  replyDelayMs: number;
}

// 任务信息
export interface TaskInfo {
  id: string;
  type: TaskTypeType;
  status: TaskStatusType;
  assigneeId: string | null;
  requesterId: string;
  content: string;
  result?: any;
  error?: string;
  createdAt: number;
  assignedAt?: number;
  completedAt?: number;
  priority: number;
  metadata?: Record<string, any>;
}

// 自动回复配置
export interface AutoReplyConfig {
  enabled: boolean;
  delayMs: number;
  maxConcurrentReplies: number;
  replyTemplates: Record<string, string[]>;
}

// 注册的智能体数据
interface RegisteredAgentData {
  id: string;
  did?: string;
  ipns?: string;
  display_name?: string;
  name?: string;
  role_description?: string;
  pubsub_topics?: string[];
  auto_reply?: boolean;
  reply_delay?: number;
  [key: string]: any;
}

class IntentAnalysis implements IntentAnalysisResult {
  intent: string;
  confidence: number;
  capabilities: AgentCapabilityType[];
  keywords: string[];
  targetAgent: string | null;

  constructor({ intent, confidence, capabilities, keywords, targetAgent }: Partial<IntentAnalysisResult> = {}) {
    this.intent = intent || 'general';
    this.confidence = confidence || 0.5;
    this.capabilities = capabilities || [];
    this.keywords = keywords || [];
    this.targetAgent = targetAgent || null;
  }
}

class AgentCoordinatorService {
  private registeredAgents: Map<string, AgentInfo> = new Map();
  private agentSubscriptions: Map<string, () => void> = new Map();
  private messageHandlers: Map<string, (message: PubSubMessage) => void> = new Map();
  private pendingRequests: Map<string, { resolve: (value: any) => void; reject: (reason?: any) => void; timeout: NodeJS.Timeout }> = new Map();
  
  // 任务管理
  private tasks: Map<string, TaskInfo> = new Map();
  private agentTasks: Map<string, Set<string>> = new Map(); // agentId -> Set<taskId>
  
  // 自动回复管理
  private autoReplyQueue: Array<{ agentId: string; message: PubSubMessage; scheduledTime: number }> = [];
  private autoReplyTimer: NodeJS.Timeout | null = null;
  private readonly DEFAULT_AUTO_REPLY_DELAY = 2000;
  private readonly MAX_CONCURRENT_REPLIES = 3;
  private processingReplies: Set<string> = new Set();
  
  // 状态管理
  private statusCheckTimer: NodeJS.Timeout | null = null;
  private readonly STATUS_CHECK_INTERVAL = 30000; // 30秒
  private readonly AGENT_TIMEOUT_MS = 120000; // 2分钟无响应视为离线

  constructor() {
    this.startAutoReplyProcessor();
    this.startStatusChecker();
  }

  /**
   * 记录日志
   */
  private log(message: string, data?: any): void {
    const prefix = '[AgentCoordinator]';
    if (data) {
      console.log(prefix, message, data);
    } else {
      console.log(prefix, message);
    }
  }

  // ==================== Agent 注册与管理 ====================

  /**
   * 注册智能体到协调器
   */
  registerAgent(agent: RegisteredAgentData): boolean {
    if (!agent?.id && !agent?.did && !agent?.ipns) {
      console.warn('[AgentCoordinator] 无法注册智能体：缺少标识');
      return false;
    }

    const agentId = agent.id || agent.did || agent.ipns || '';
    
    // 确保名称不为 undefined
    let agentName = agent.display_name || agent.name;
    if (!agentName && agent.did) {
      const didParts = agent.did.split(':');
      if (didParts.length > 2) {
        const lastPart = didParts[didParts.length - 1];
        agentName = lastPart.length > 16 ? lastPart.substring(0, 8) : lastPart;
      }
    }
    if (!agentName) {
      agentName = agentId || 'Unknown Agent';
    }

    const now = Date.now();
    const agentInfo: AgentInfo = {
      id: agentId,
      did: agent.did,
      ipns: agent.ipns,
      name: agentName,
      description: agent.role_description || '',
      capabilities: this._extractCapabilities(agent),
      pubsubTopics: agent.pubsub_topics || [],
      status: AgentStatus.ONLINE,
      registeredAt: now,
      lastSeenAt: now,
      currentTask: null,
      autoReplyEnabled: agent.auto_reply !== false, // 默认开启
      replyDelayMs: agent.reply_delay || this.DEFAULT_AUTO_REPLY_DELAY,
    };

    this.registeredAgents.set(agentId, agentInfo);
    this.agentTasks.set(agentId, new Set());
    
    this.log('注册智能体:', { name: agentInfo.name, id: agentId, status: agentInfo.status });

    // 订阅智能体的 PubSub 主题
    if (agentInfo.pubsubTopics.length > 0) {
      this._subscribeToAgentTopics(agentId, agentInfo.pubsubTopics);
    }

    // 广播智能体上线消息
    this.broadcastAgentStatus(agentId, AgentStatus.ONLINE);

    return true;
  }

  /**
   * 注销智能体
   */
  unregisterAgent(agentId: string): void {
    this.log('注销智能体:', { agentId });
    
    // 取消订阅
    const unsubscribe = this.agentSubscriptions.get(agentId);
    if (unsubscribe) {
      unsubscribe();
      this.agentSubscriptions.delete(agentId);
    }

    // 清理任务分配
    const taskIds = this.agentTasks.get(agentId);
    if (taskIds) {
      taskIds.forEach(taskId => {
        const task = this.tasks.get(taskId);
        if (task && task.status === TaskStatus.IN_PROGRESS) {
          this.updateTaskStatus(taskId, TaskStatus.PENDING, '智能体离线，任务重新分配');
        }
      });
    }

    // 广播离线消息
    this.broadcastAgentStatus(agentId, AgentStatus.OFFLINE);

    this.registeredAgents.delete(agentId);
    this.agentTasks.delete(agentId);
    this.messageHandlers.delete(agentId);
  }

  /**
   * 更新智能体状态
   */
  updateAgentStatus(agentId: string, status: AgentStatusType, reason?: string): void {
    const agent = this.registeredAgents.get(agentId);
    if (!agent) return;

    const oldStatus = agent.status;
    agent.status = status;
    agent.lastSeenAt = Date.now();

    this.log('智能体状态更新:', { 
      agentId, 
      name: agent.name,
      oldStatus, 
      newStatus: status,
      reason 
    });

    // 广播状态变更
    this.broadcastAgentStatus(agentId, status, reason);

    // 如果变为忙碌，暂停自动回复
    if (status === AgentStatus.BUSY) {
      this.pauseAutoReply(agentId);
    }
  }

  /**
   * 广播智能体状态
   */
  private async broadcastAgentStatus(agentId: string, status: AgentStatusType, reason?: string): Promise<void> {
    const agent = this.registeredAgents.get(agentId);
    if (!agent) return;

    const statusMessage = new PubSubMessage({
      type: MessageType.STATUS_UPDATE,
      from: agent.did || agentId,
      content: `Agent ${agent.name} is ${status}`,
      topic: 'alou/agents/status',
      metadata: {
        agentId,
        agentName: agent.name,
        status,
        reason,
        timestamp: Date.now(),
        capabilities: agent.capabilities,
      },
    });

    try {
      await pubsubService.publish('alou/agents/status', statusMessage);
    } catch (error: any) {
      this.log('广播状态失败:', { error: error.message });
    }
  }

  /**
   * 设置智能体消息处理器
   */
  setMessageHandler(agentId: string, handler: (message: PubSubMessage) => void): void {
    this.messageHandlers.set(agentId, handler);
  }

  /**
   * 启用/禁用自动回复
   */
  setAutoReply(agentId: string, enabled: boolean, delayMs?: number): boolean {
    const agent = this.registeredAgents.get(agentId);
    if (!agent) {
      console.warn('[AgentCoordinator] 智能体不存在:', agentId);
      return false;
    }

    agent.autoReplyEnabled = enabled;
    if (delayMs !== undefined) {
      agent.replyDelayMs = delayMs;
    }

    this.log('设置自动回复:', { agentId, enabled, delayMs: agent.replyDelayMs });
    return true;
  }

  /**
   * 暂停自动回复
   */
  private pauseAutoReply(agentId: string): void {
    // 从队列中移除该智能体的待回复
    this.autoReplyQueue = this.autoReplyQueue.filter(item => item.agentId !== agentId);
  }

  // ==================== 意图分析与路由 ====================

  /**
   * 分析用户意图
   */
  analyzeIntent(message: string): IntentAnalysis {
    const lowerMessage = message.toLowerCase();
    const analysis = new IntentAnalysis({ intent: 'general' });

    // 区块链相关
    if (this._matchKeywords(lowerMessage, ['转账', '发送', 'transfer', 'send', '余额', 'balance', '交易', 'transaction'])) {
      analysis.intent = 'blockchain';
      analysis.capabilities.push(AgentCapability.BLOCKCHAIN, AgentCapability.WALLET);
      analysis.confidence = 0.8;
    }

    // DeFi 相关
    if (this._matchKeywords(lowerMessage, ['swap', '兑换', 'stake', '质押', 'liquidity', '流动性', 'yield', '收益', 'defi'])) {
      analysis.intent = 'defi';
      analysis.capabilities.push(AgentCapability.DEFI, AgentCapability.BLOCKCHAIN);
      analysis.confidence = 0.85;
    }

    // NFT 相关
    if (this._matchKeywords(lowerMessage, ['nft', '铸造', 'mint', '收藏品', 'collectible', 'opensea'])) {
      analysis.intent = 'nft';
      analysis.capabilities.push(AgentCapability.NFT);
      analysis.confidence = 0.85;
    }

    // 翻译相关
    if (this._matchKeywords(lowerMessage, ['翻译', 'translate', '英文', '中文', 'english', 'chinese'])) {
      analysis.intent = 'translation';
      analysis.capabilities.push(AgentCapability.TRANSLATION);
      analysis.confidence = 0.9;
    }

    // 代码相关
    if (this._matchKeywords(lowerMessage, ['代码', 'code', '编程', 'program', '函数', 'function', 'bug', '调试'])) {
      analysis.intent = 'code';
      analysis.capabilities.push(AgentCapability.CODE);
      analysis.confidence = 0.85;
    }

    // 研究分析
    if (this._matchKeywords(lowerMessage, ['分析', 'analyze', '研究', 'research', '报告', 'report', '数据'])) {
      analysis.intent = 'research';
      analysis.capabilities.push(AgentCapability.RESEARCH);
      analysis.confidence = 0.75;
    }

    return analysis;
  }

  /**
   * 根据意图匹配最合适的智能体
   */
  matchAgentByIntent(intent: IntentAnalysis): AgentInfo | null {
    const agents = Array.from(this.registeredAgents.values()).filter(
      a => a.status !== AgentStatus.OFFLINE && a.status !== AgentStatus.ERROR
    );

    if (agents.length === 0) {
      return null;
    }

    // 计算每个智能体的匹配分数
    const scoredAgents = agents.map((agent) => {
      let score = 0;

      // 能力匹配
      for (const cap of intent.capabilities) {
        if (agent.capabilities.includes(cap)) {
          score += 10;
        }
      }

      // 关键词匹配（在描述中）
      const description = (agent.description || '').toLowerCase();
      for (const keyword of intent.keywords) {
        if (description.includes(keyword)) {
          score += 5;
        }
      }

      // 状态评分
      if (agent.status === AgentStatus.IDLE) {
        score += 5;
      } else if (agent.status === AgentStatus.ONLINE) {
        score += 3;
      }

      // 当前任务负载（任务越少越好）
      const taskCount = this.agentTasks.get(agent.id)?.size || 0;
      score -= taskCount * 2;

      return { agent, score };
    });

    // 按分数排序
    scoredAgents.sort((a, b) => b.score - a.score);

    // 返回最高分的智能体（如果分数大于0）
    if (scoredAgents[0]?.score > 0) {
      return scoredAgents[0].agent;
    }

    // 没有匹配的，返回第一个在线的智能体
    return agents.find((a) => a.status === AgentStatus.ONLINE || a.status === AgentStatus.IDLE) || agents[0];
  }

  /**
   * 路由消息到合适的智能体
   */
  async routeMessage(message: string, fromAgentId: string | null = null): Promise<AgentInfo | null> {
    const intent = this.analyzeIntent(message);
    this.log('意图分析:', { intent: intent.intent, confidence: intent.confidence });

    const targetAgent = this.matchAgentByIntent(intent);
    if (!targetAgent) {
      console.warn('[AgentCoordinator] 没有找到合适的智能体');
      return null;
    }

    this.log('路由到智能体:', { name: targetAgent.name, id: targetAgent.id });
    return targetAgent;
  }

  // ==================== 任务管理 ====================

  /**
   * 创建任务
   */
  createTask(
    type: TaskTypeType,
    content: string,
    requesterId: string,
    priority: number = 1,
    metadata?: Record<string, any>
  ): TaskInfo {
    const taskId = `task_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    
    const task: TaskInfo = {
      id: taskId,
      type,
      status: TaskStatus.PENDING,
      assigneeId: null,
      requesterId,
      content,
      createdAt: Date.now(),
      priority,
      metadata,
    };

    this.tasks.set(taskId, task);
    this.log('创建任务:', { taskId, type, priority, requesterId });

    // 尝试自动分配任务
    this.tryAssignTask(taskId);

    return task;
  }

  /**
   * 尝试分配任务
   */
  private async tryAssignTask(taskId: string): Promise<boolean> {
    const task = this.tasks.get(taskId);
    if (!task || task.status !== TaskStatus.PENDING) return false;

    // 根据任务类型和意图匹配合适的智能体
    const intent = this.analyzeIntent(task.content);
    const targetAgent = this.matchAgentByIntent(intent);

    if (!targetAgent) {
      this.log('无可用的智能体分配任务:', { taskId });
      return false;
    }

    return this.assignTask(taskId, targetAgent.id);
  }

  /**
   * 分配任务给智能体
   */
  async assignTask(taskId: string, agentId: string): Promise<boolean> {
    const task = this.tasks.get(taskId);
    const agent = this.registeredAgents.get(agentId);

    if (!task) {
      console.warn('[AgentCoordinator] 任务不存在:', taskId);
      return false;
    }

    if (!agent) {
      console.warn('[AgentCoordinator] 智能体不存在:', agentId);
      return false;
    }

    if (agent.status === AgentStatus.OFFLINE || agent.status === AgentStatus.BUSY) {
      this.log('智能体不可用:', { agentId, status: agent.status });
      return false;
    }

    // 更新任务状态
    task.assigneeId = agentId;
    task.status = TaskStatus.ASSIGNED;
    task.assignedAt = Date.now();

    // 更新智能体状态
    agent.status = AgentStatus.BUSY;
    agent.currentTask = taskId;

    // 记录任务分配
    const agentTaskSet = this.agentTasks.get(agentId);
    if (agentTaskSet) {
      agentTaskSet.add(taskId);
    }

    this.log('任务已分配:', { taskId, agentId: agent.name, type: task.type });

    // 发送任务分配消息
    await this.sendTaskAssignment(task, agent);

    return true;
  }

  /**
   * 发送任务分配消息
   */
  private async sendTaskAssignment(task: TaskInfo, agent: AgentInfo): Promise<void> {
    const assignmentMessage = new PubSubMessage({
      type: MessageType.TASK_ASSIGN,
      from: task.requesterId,
      to: agent.did || agent.id,
      content: task.content,
      topic: agent.pubsubTopics[0] || pubsubService.generateAgentTopic(agent.id),
      metadata: {
        taskId: task.id,
        taskType: task.type,
        priority: task.priority,
        assignedAt: task.assignedAt,
      },
    });

    try {
      await pubsubService.publish(assignmentMessage.topic, assignmentMessage);
    } catch (error: any) {
      this.log('发送任务分配消息失败:', { error: error.message });
    }
  }

  /**
   * 更新任务状态
   */
  updateTaskStatus(taskId: string, status: TaskStatusType, result?: any, error?: string): TaskInfo | null {
    const task = this.tasks.get(taskId);
    if (!task) return null;

    const oldStatus = task.status;
    task.status = status;

    if (status === TaskStatus.COMPLETED) {
      task.result = result;
      task.completedAt = Date.now();
      
      // 释放智能体
      if (task.assigneeId) {
        this.releaseAgent(task.assigneeId, taskId);
      }
    } else if (status === TaskStatus.FAILED) {
      task.error = error;
      
      // 释放智能体
      if (task.assigneeId) {
        this.releaseAgent(task.assigneeId, taskId);
      }
    }

    this.log('任务状态更新:', { 
      taskId, 
      oldStatus, 
      newStatus: status,
      assignee: task.assigneeId 
    });

    return task;
  }

  /**
   * 释放智能体（完成任务后）
   */
  private releaseAgent(agentId: string, taskId: string): void {
    const agent = this.registeredAgents.get(agentId);
    const agentTaskSet = this.agentTasks.get(agentId);

    if (agentTaskSet) {
      agentTaskSet.delete(taskId);
    }

    if (agent) {
      // 检查是否还有其他任务
      const remainingTasks = agentTaskSet?.size || 0;
      if (remainingTasks === 0) {
        agent.status = AgentStatus.IDLE;
        agent.currentTask = null;
      }
      
      this.log('释放智能体:', { agentId, name: agent.name, remainingTasks });
    }
  }

  /**
   * 获取任务信息
   */
  getTask(taskId: string): TaskInfo | undefined {
    return this.tasks.get(taskId);
  }

  /**
   * 获取智能体的所有任务
   */
  getAgentTasks(agentId: string): TaskInfo[] {
    const taskIds = this.agentTasks.get(agentId);
    if (!taskIds) return [];
    
    return Array.from(taskIds)
      .map(id => this.tasks.get(id))
      .filter((task): task is TaskInfo => task !== undefined);
  }

  /**
   * 获取待处理任务列表
   */
  getPendingTasks(): TaskInfo[] {
    return Array.from(this.tasks.values())
      .filter(task => task.status === TaskStatus.PENDING)
      .sort((a, b) => b.priority - a.priority);
  }

  // ==================== 消息通信 ====================

  /**
   * 智能体间发送消息
   */
  async sendAgentMessage(fromAgentId: string, toAgentId: string, message: string, metadata: Record<string, any> = {}): Promise<boolean> {
    const fromAgent = this.registeredAgents.get(fromAgentId);
    const toAgent = this.registeredAgents.get(toAgentId);

    if (!toAgent) {
      console.warn('[AgentCoordinator] 目标智能体未注册:', toAgentId);
      return false;
    }

    // 生成请求 ID
    const requestId = `req_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;

    // 构建消息
    const pubsubMessage = new PubSubMessage({
      type: MessageType.AGENT_REQUEST,
      from: fromAgent?.did || fromAgentId,
      to: toAgent.did || toAgentId,
      content: message,
      topic: toAgent.pubsubTopics[0] || pubsubService.generateAgentTopic(toAgentId),
      metadata: {
        ...metadata,
        requestId,
        fromAgentName: fromAgent?.name,
        toAgentName: toAgent.name,
      },
    });

    // 发布消息
    const topic = pubsubMessage.topic;
    const success = await pubsubService.publish(topic, pubsubMessage);

    if (success) {
      this.log('智能体消息已发送:', { from: fromAgentId, to: toAgentId });
    }

    return success;
  }

  /**
   * 智能体间请求（带响应等待）
   */
  async requestFromAgent(fromAgentId: string, toAgentId: string, message: string, timeoutMs = 30000): Promise<any> {
    const requestId = `req_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;

    return new Promise((resolve, reject) => {
      // 设置超时
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error('智能体响应超时'));
      }, timeoutMs);

      // 保存待处理请求
      this.pendingRequests.set(requestId, { resolve, reject, timeout: timeout as unknown as NodeJS.Timeout });

      // 发送请求
      this.sendAgentMessage(fromAgentId, toAgentId, message, { requestId }).catch((err) => {
        clearTimeout(timeout);
        this.pendingRequests.delete(requestId);
        reject(err);
      });
    });
  }

  /**
   * 处理智能体响应
   */
  handleAgentResponse(requestId: string, response: any): void {
    const pending = this.pendingRequests.get(requestId);
    if (pending) {
      clearTimeout(pending.timeout);
      this.pendingRequests.delete(requestId);
      pending.resolve(response);
    }
  }

  /**
   * 广播消息给所有智能体
   */
  async broadcastToAgents(message: string, excludeAgentId: string | null = null): Promise<Array<{ agentId: string; success: boolean }>> {
    const agents = Array.from(this.registeredAgents.values()).filter(
      a => a.status !== AgentStatus.OFFLINE
    );
    const results = [];

    for (const agent of agents) {
      if (agent.id === excludeAgentId) continue;

      const success = await this.sendAgentMessage(excludeAgentId || '', agent.id, message);
      results.push({ agentId: agent.id, success });
    }

    return results;
  }

  // ==================== 自动回复机制 ====================

  /**
   * 队列自动回复
   */
  queueAutoReply(agentId: string, message: PubSubMessage): void {
    const agent = this.registeredAgents.get(agentId);
    if (!agent || !agent.autoReplyEnabled) return;
    if (agent.status === AgentStatus.BUSY) return;

    // 检查是否已在队列中
    const exists = this.autoReplyQueue.some(
      item => item.agentId === agentId && item.message.metadata?.requestId === message.metadata?.requestId
    );
    if (exists) return;

    const scheduledTime = Date.now() + agent.replyDelayMs;
    this.autoReplyQueue.push({ agentId, message, scheduledTime });
    
    this.log('自动回复已排队:', { 
      agentId, 
      scheduledTime: new Date(scheduledTime).toISOString(),
      queueSize: this.autoReplyQueue.length 
    });
  }

  /**
   * 启动自动回复处理器
   */
  private startAutoReplyProcessor(): void {
    if (this.autoReplyTimer) {
      clearInterval(this.autoReplyTimer);
    }

    this.autoReplyTimer = setInterval(() => {
      this.processAutoReplyQueue();
    }, 500);

    this.log('自动回复处理器已启动');
  }

  /**
   * 处理自动回复队列
   */
  private async processAutoReplyQueue(): Promise<void> {
    if (this.autoReplyQueue.length === 0) return;
    if (this.processingReplies.size >= this.MAX_CONCURRENT_REPLIES) return;

    const now = Date.now();
    const toProcess = this.autoReplyQueue.filter(item => item.scheduledTime <= now);

    for (const item of toProcess) {
      if (this.processingReplies.size >= this.MAX_CONCURRENT_REPLIES) break;

      // 从队列中移除
      const index = this.autoReplyQueue.indexOf(item);
      if (index > -1) {
        this.autoReplyQueue.splice(index, 1);
      }

      // 执行自动回复
      this.processingReplies.add(item.agentId);
      this.executeAutoReply(item.agentId, item.message).finally(() => {
        this.processingReplies.delete(item.agentId);
      });
    }
  }

  /**
   * 执行自动回复
   */
  private async executeAutoReply(agentId: string, originalMessage: PubSubMessage): Promise<void> {
    const agent = this.registeredAgents.get(agentId);
    if (!agent) return;

    this.log('执行自动回复:', { agentId: agent.name, from: originalMessage.from });

    try {
      // 生成简单的回复内容（实际应用中应该由AI生成）
      const replyContent = this.generateReplyContent(agent, originalMessage);
      
      // 发送回复
      const fromAgentId = originalMessage.metadata?.fromAgentName || originalMessage.from;
      await this.sendAgentMessage(
        agentId,
        fromAgentId,
        replyContent,
        {
          replyTo: originalMessage.metadata?.requestId,
          isAutoReply: true,
        }
      );

      this.log('自动回复已发送:', { agentId: agent.name });
    } catch (error: any) {
      this.log('自动回复失败:', { error: error.message });
    }
  }

  /**
   * 生成回复内容
   */
  private generateReplyContent(agent: AgentInfo, message: PubSubMessage): string {
    // 根据智能体能力和消息内容生成回复
    const templates: Record<string, string[]> = {
      [AgentCapability.CODE]: [
        `我是${agent.name}，我来帮您处理代码相关的问题。`,
        `收到您的编程需求，让我来协助您。`,
      ],
      [AgentCapability.BLOCKCHAIN]: [
        `我是${agent.name}，我来帮您处理区块链操作。`,
        `收到您的区块链请求，让我来协助您。`,
      ],
      [AgentCapability.RESEARCH]: [
        `我是${agent.name}，我来帮您进行研究和分析。`,
        `收到您的研究需求，让我来协助您。`,
      ],
      default: [
        `我是${agent.name}，我已收到您的消息。`,
        `您好，我是${agent.name}，很高兴为您服务。`,
        `收到！我是${agent.name}。`,
      ],
    };

    // 根据能力选择模板
    let selectedTemplates = templates.default;
    for (const cap of agent.capabilities) {
      if (templates[cap]) {
        selectedTemplates = templates[cap];
        break;
      }
    }

    // 随机选择一条
    return selectedTemplates[Math.floor(Math.random() * selectedTemplates.length)];
  }

  // ==================== 状态检查 ====================

  /**
   * 启动状态检查器
   */
  private startStatusChecker(): void {
    if (this.statusCheckTimer) {
      clearInterval(this.statusCheckTimer);
    }

    this.statusCheckTimer = setInterval(() => {
      this.checkAgentsStatus();
    }, this.STATUS_CHECK_INTERVAL);

    this.log('状态检查器已启动');
  }

  /**
   * 检查智能体状态
   */
  private checkAgentsStatus(): void {
    const now = Date.now();
    
    for (const [agentId, agent] of this.registeredAgents.entries()) {
      // 检查是否超时
      if (now - agent.lastSeenAt > this.AGENT_TIMEOUT_MS) {
        if (agent.status !== AgentStatus.OFFLINE) {
          this.updateAgentStatus(agentId, AgentStatus.OFFLINE, '超时未响应');
        }
      }
    }
  }

  /**
   * 刷新智能体状态（心跳）
   */
  refreshAgentHeartbeat(agentId: string): void {
    const agent = this.registeredAgents.get(agentId);
    if (agent) {
      agent.lastSeenAt = Date.now();
      if (agent.status === AgentStatus.OFFLINE) {
        this.updateAgentStatus(agentId, AgentStatus.ONLINE, '心跳恢复');
      }
    }
  }

  // ==================== 查询接口 ====================

  /**
   * 获取所有已注册的智能体
   */
  getRegisteredAgents(): AgentInfo[] {
    return Array.from(this.registeredAgents.values());
  }

  /**
   * 获取在线的智能体
   */
  getOnlineAgents(): AgentInfo[] {
    return Array.from(this.registeredAgents.values()).filter(
      a => a.status !== AgentStatus.OFFLINE
    );
  }

  /**
   * 获取智能体信息
   */
  getAgent(agentId: string): AgentInfo | undefined {
    return this.registeredAgents.get(agentId);
  }

  /**
   * 获取服务统计
   */
  getStats(): {
    totalAgents: number;
    onlineAgents: number;
    busyAgents: number;
    totalTasks: number;
    pendingTasks: number;
    inProgressTasks: number;
    autoReplyQueueSize: number;
    pendingRequests: number;
  } {
    const agents = Array.from(this.registeredAgents.values());
    const tasks = Array.from(this.tasks.values());
    
    return {
      totalAgents: agents.length,
      onlineAgents: agents.filter(a => a.status !== AgentStatus.OFFLINE).length,
      busyAgents: agents.filter(a => a.status === AgentStatus.BUSY).length,
      totalTasks: tasks.length,
      pendingTasks: tasks.filter(t => t.status === TaskStatus.PENDING).length,
      inProgressTasks: tasks.filter(t => t.status === TaskStatus.IN_PROGRESS).length,
      autoReplyQueueSize: this.autoReplyQueue.length,
      pendingRequests: this.pendingRequests.size,
    };
  }

  // ==================== 辅助方法 ====================

  /**
   * 提取智能体能力
   */
  private _extractCapabilities(agent: RegisteredAgentData): AgentCapabilityType[] {
    const capabilities: AgentCapabilityType[] = [AgentCapability.CHAT]; // 默认都有聊天能力
    const description = (agent.role_description || agent.description || '').toLowerCase();

    if (this._matchKeywords(description, ['blockchain', '区块链', 'web3', 'crypto'])) {
      capabilities.push(AgentCapability.BLOCKCHAIN);
    }
    if (this._matchKeywords(description, ['defi', 'swap', 'stake', '质押'])) {
      capabilities.push(AgentCapability.DEFI);
    }
    if (this._matchKeywords(description, ['nft', '收藏品'])) {
      capabilities.push(AgentCapability.NFT);
    }
    if (this._matchKeywords(description, ['翻译', 'translate', 'translation'])) {
      capabilities.push(AgentCapability.TRANSLATION);
    }
    if (this._matchKeywords(description, ['代码', 'code', '编程', 'developer'])) {
      capabilities.push(AgentCapability.CODE);
    }
    if (this._matchKeywords(description, ['研究', 'research', '分析', 'analyst'])) {
      capabilities.push(AgentCapability.RESEARCH);
    }
    if (this._matchKeywords(description, ['钱包', 'wallet'])) {
      capabilities.push(AgentCapability.WALLET);
    }

    return [...new Set(capabilities)];
  }

  /**
   * 关键词匹配
   */
  private _matchKeywords(text: string, keywords: string[]): boolean {
    return keywords.some((kw) => text.includes(kw));
  }

  /**
   * 订阅智能体的 PubSub 主题
   */
  private _subscribeToAgentTopics(agentId: string, topics: string[]): void {
    const unsubscribes: (() => void)[] = [];

    for (const topic of topics) {
      const unsubscribe = pubsubService.subscribe(topic, (message: PubSubMessage) => {
        this._handleIncomingMessage(agentId, message);
      });
      unsubscribes.push(unsubscribe);
    }

    // 保存取消订阅函数
    this.agentSubscriptions.set(agentId, () => {
      unsubscribes.forEach((fn) => fn());
    });
  }

  /**
   * 处理收到的消息
   */
  private _handleIncomingMessage(agentId: string, message: PubSubMessage): void {
    this.log('收到消息:', { agentId, type: message.type });

    // 更新智能体最后活跃时间
    this.refreshAgentHeartbeat(agentId);

    // 如果是响应消息，处理待处理请求
    if (message.type === MessageType.AGENT_RESPONSE && message.metadata?.requestId) {
      this.handleAgentResponse(message.metadata.requestId, message);
      return;
    }

    // 如果是任务分配相关消息
    if (message.type === MessageType.TASK_ASSIGN) {
      this.handleTaskAssignmentMessage(agentId, message);
      return;
    }

    // 队列自动回复
    if (message.type === MessageType.AGENT_REQUEST) {
      this.queueAutoReply(agentId, message);
    }

    // 调用智能体的消息处理器
    const handler = this.messageHandlers.get(agentId);
    if (handler) {
      handler(message);
    }
  }

  /**
   * 处理任务分配消息
   */
  private handleTaskAssignmentMessage(agentId: string, message: PubSubMessage): void {
    const taskId = message.metadata?.taskId;
    if (!taskId) return;

    const task = this.tasks.get(taskId);
    if (task) {
      // 更新任务状态为进行中
      this.updateTaskStatus(taskId, TaskStatus.IN_PROGRESS);
      
      this.log('任务开始执行:', { taskId, agentId });
    }
  }

  /**
   * 清理所有资源
   */
  cleanup(): void {
    this.log('开始清理资源');
    
    // 停止定时器
    if (this.autoReplyTimer) {
      clearInterval(this.autoReplyTimer);
      this.autoReplyTimer = null;
    }
    
    if (this.statusCheckTimer) {
      clearInterval(this.statusCheckTimer);
      this.statusCheckTimer = null;
    }

    // 取消所有订阅
    this.agentSubscriptions.forEach((unsubscribe) => unsubscribe());
    this.agentSubscriptions.clear();

    // 清理待处理请求
    this.pendingRequests.forEach(({ timeout }) => clearTimeout(timeout));
    this.pendingRequests.clear();

    // 清理数据
    this.registeredAgents.clear();
    this.messageHandlers.clear();
    this.tasks.clear();
    this.agentTasks.clear();
    this.autoReplyQueue = [];
    this.processingReplies.clear();

    this.log('资源已清理');
  }
}

export default new AgentCoordinatorService();
