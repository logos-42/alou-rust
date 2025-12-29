/**
 * Session 实体类
 * 表示用户会话
 */

// 简单的ID生成函数
function generateId(): string {
  return 'session_' + Date.now().toString(36) + Math.random().toString(36).substr(2, 9);
}

// 会话状态
export type SessionStatus = 'active' | 'inactive' | 'closed' | 'expired' | 'error';

// 会话类型
export type SessionType = 'user' | 'guest' | 'admin' | 'system' | 'api';

// 会话权限
export interface SessionPermissions {
  canChat: boolean;
  canCreateAgents: boolean;
  canManageChannels: boolean;
  canAccessSettings: boolean;
  canInviteUsers: boolean;
  canExportData: boolean;
  canDeleteContent: boolean;
  maxAgents?: number;
  maxChannels?: number;
  maxMessagesPerDay?: number;
}

// 会话统计信息
export interface SessionStats {
  messageCount: number;
  agentCount: number;
  channelCount: number;
  totalTokensUsed: number;
  lastActivity: Date;
  averageResponseTime?: number;
  successRate?: number;
}

// 会话元数据
export interface SessionMetadata {
  ipAddress?: string;
  userAgent?: string;
  deviceInfo?: {
    type: 'desktop' | 'mobile' | 'tablet';
    os: string;
    browser: string;
  };
  location?: {
    country?: string;
    region?: string;
    city?: string;
  };
  customData?: Record<string, any>;
  tags?: string[];
}

// 会话实体类
export class Session {
  // 核心属性
  readonly id: string;
  readonly userId: string;
  type: SessionType;
  status: SessionStatus;
  
  // 关联实体
  agentId?: string;
  channelId?: string;
  
  // 权限和配置
  permissions: SessionPermissions;
  
  // 统计信息
  stats: SessionStats;
  
  // 元数据
  metadata: SessionMetadata;
  
  // 时间戳
  readonly createdAt: Date;
  updatedAt: Date;
  expiresAt?: Date;
  lastActivityAt: Date;
  
  // 构造函数
  constructor(data: Partial<Session> & { userId: string }) {
    this.id = data.id || generateId();
    this.userId = data.userId;
    this.type = data.type || 'user';
    this.status = data.status || 'active';
    this.agentId = data.agentId;
    this.channelId = data.channelId;
    
    // 默认权限
    this.permissions = data.permissions || this.getDefaultPermissions(data.type);
    
    // 默认统计信息
    this.stats = data.stats || {
      messageCount: 0,
      agentCount: 0,
      channelCount: 0,
      totalTokensUsed: 0,
      lastActivity: new Date(),
      averageResponseTime: undefined,
      successRate: undefined,
    };
    
    // 默认元数据
    this.metadata = data.metadata || {
      tags: [],
      customData: {},
    };
    
    // 时间戳
    const now = new Date();
    this.createdAt = data.createdAt || now;
    this.updatedAt = data.updatedAt || now;
    this.lastActivityAt = data.lastActivityAt || now;
    
    // 设置过期时间（默认24小时）
    if (data.expiresAt) {
      this.expiresAt = data.expiresAt;
    } else if (this.type === 'guest') {
      // 访客会话1小时后过期
      this.expiresAt = new Date(now.getTime() + 60 * 60 * 1000);
    } else {
      // 普通用户会话24小时后过期
      this.expiresAt = new Date(now.getTime() + 24 * 60 * 60 * 1000);
    }
  }
  
  // 获取默认权限
  private getDefaultPermissions(type?: SessionType): SessionPermissions {
    const basePermissions: SessionPermissions = {
      canChat: true,
      canCreateAgents: false,
      canManageChannels: false,
      canAccessSettings: false,
      canInviteUsers: false,
      canExportData: false,
      canDeleteContent: false,
      maxAgents: 3,
      maxChannels: 10,
      maxMessagesPerDay: 100,
    };
    
    switch (type) {
      case 'admin':
        return {
          ...basePermissions,
          canCreateAgents: true,
          canManageChannels: true,
          canAccessSettings: true,
          canInviteUsers: true,
          canExportData: true,
          canDeleteContent: true,
          maxAgents: 100,
          maxChannels: 1000,
          maxMessagesPerDay: 10000,
        };
      
      case 'user':
        return {
          ...basePermissions,
          canCreateAgents: true,
          canManageChannels: true,
          canAccessSettings: true,
          maxAgents: 10,
          maxChannels: 50,
          maxMessagesPerDay: 1000,
        };
      
      case 'guest':
        return {
          ...basePermissions,
          canCreateAgents: false,
          canManageChannels: false,
          maxAgents: 0,
          maxChannels: 1,
          maxMessagesPerDay: 50,
        };
      
      default:
        return basePermissions;
    }
  }
  
  // 更新会话状态
  updateStatus(status: SessionStatus): this {
    this.status = status;
    this.updatedAt = new Date();
    return this;
  }
  
  // 更新活动时间
  updateActivity(): this {
    const now = new Date();
    this.lastActivityAt = now;
    this.stats.lastActivity = now;
    this.updatedAt = now;
    return this;
  }
  
  // 增加消息计数
  incrementMessageCount(): this {
    this.stats.messageCount += 1;
    this.updateActivity();
    return this;
  }
  
  // 增加令牌使用量
  addTokensUsed(tokens: number): this {
    this.stats.totalTokensUsed += tokens;
    this.updateActivity();
    return this;
  }
  
  // 设置关联代理
  setAgent(agentId: string): this {
    this.agentId = agentId;
    this.updatedAt = new Date();
    return this;
  }
  
  // 设置关联频道
  setChannel(channelId: string): this {
    this.channelId = channelId;
    this.updatedAt = new Date();
    return this;
  }
  
  // 更新权限
  updatePermissions(permissions: Partial<SessionPermissions>): this {
    this.permissions = { ...this.permissions, ...permissions };
    this.updatedAt = new Date();
    return this;
  }
  
  // 添加元数据
  addMetadata(metadata: Partial<SessionMetadata>): this {
    this.metadata = { ...this.metadata, ...metadata };
    this.updatedAt = new Date();
    return this;
  }
  
  // 延长会话过期时间
  extendExpiration(hours: number = 24): this {
    const now = new Date();
    this.expiresAt = new Date(now.getTime() + hours * 60 * 60 * 1000);
    this.updatedAt = now;
    return this;
  }
  
  // 验证会话数据
  validate(): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 验证用户ID
    if (!this.userId || this.userId.trim().length === 0) {
      errors.push('用户ID不能为空');
    }
    
    // 验证过期时间
    if (this.expiresAt && this.expiresAt < new Date()) {
      errors.push('会话已过期');
    }
    
    // 验证权限限制
    if (this.permissions.maxAgents !== undefined && this.stats.agentCount > this.permissions.maxAgents) {
      errors.push(`代理数量超过限制（最大${this.permissions.maxAgents}个）`);
    }
    
    if (this.permissions.maxChannels !== undefined && this.stats.channelCount > this.permissions.maxChannels) {
      errors.push(`频道数量超过限制（最大${this.permissions.maxChannels}个）`);
    }
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
  
  // 检查会话是否活跃
  isActive(): boolean {
    return this.status === 'active' && !this.isExpired();
  }
  
  // 检查会话是否过期
  isExpired(): boolean {
    if (!this.expiresAt) {
      return false;
    }
    return this.expiresAt < new Date();
  }
  
  // 检查会话是否空闲（30分钟无活动）
  isIdle(): boolean {
    const thirtyMinutesAgo = new Date(Date.now() - 30 * 60 * 1000);
    return this.lastActivityAt < thirtyMinutesAgo;
  }
  
  // 检查是否有特定权限
  hasPermission(permission: keyof SessionPermissions): boolean {
    return this.permissions[permission] === true;
  }
  
  // 检查是否达到限制
  isAtLimit(resource: 'agents' | 'channels' | 'messages'): boolean {
    switch (resource) {
      case 'agents':
        return this.permissions.maxAgents !== undefined && 
               this.stats.agentCount >= this.permissions.maxAgents;
      
      case 'channels':
        return this.permissions.maxChannels !== undefined && 
               this.stats.channelCount >= this.permissions.maxChannels;
      
      case 'messages':
        // 这里需要额外的逻辑来检查每日消息限制
        return false;
      
      default:
        return false;
    }
  }
  
  // 获取剩余时间（分钟）
  getTimeRemaining(): number {
    if (!this.expiresAt) {
      return Infinity;
    }
    
    const now = new Date();
    const diff = this.expiresAt.getTime() - now.getTime();
    return Math.max(0, Math.floor(diff / 60000)); // 转换为分钟
  }
  
  // 转换为纯对象（用于序列化）
  toJSON(): Record<string, any> {
    return {
      id: this.id,
      userId: this.userId,
      type: this.type,
      status: this.status,
      agentId: this.agentId,
      channelId: this.channelId,
      permissions: this.permissions,
      stats: {
        ...this.stats,
        lastActivity: this.stats.lastActivity.toISOString(),
      },
      metadata: this.metadata,
      createdAt: this.createdAt.toISOString(),
      updatedAt: this.updatedAt.toISOString(),
      expiresAt: this.expiresAt?.toISOString(),
      lastActivityAt: this.lastActivityAt.toISOString(),
    };
  }
  
  // 从纯对象创建实例（用于反序列化）
  static fromJSON(data: Record<string, any>): Session {
    return new Session({
      id: data.id,
      userId: data.userId,
      type: data.type,
      status: data.status,
      agentId: data.agentId,
      channelId: data.channelId,
      permissions: data.permissions,
      stats: {
        ...data.stats,
        lastActivity: data.stats?.lastActivity ? new Date(data.stats.lastActivity) : new Date(),
      },
      metadata: data.metadata,
      createdAt: data.createdAt ? new Date(data.createdAt) : new Date(),
      updatedAt: data.updatedAt ? new Date(data.updatedAt) : new Date(),
      expiresAt: data.expiresAt ? new Date(data.expiresAt) : undefined,
      lastActivityAt: data.lastActivityAt ? new Date(data.lastActivityAt) : new Date(),
    });
  }
  
  // 创建用户会话
  static createUserSession(userId: string, permissions?: Partial<SessionPermissions>): Session {
    return new Session({
      userId,
      type: 'user',
      permissions: permissions ? { ...this.prototype.getDefaultPermissions('user'), ...permissions } : undefined,
    });
  }
  
  // 创建访客会话
  static createGuestSession(): Session {
    const guestId = `guest_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    return new Session({
      userId: guestId,
      type: 'guest',
    });
  }
  
  // 创建管理员会话
  static createAdminSession(userId: string): Session {
    return new Session({
      userId,
      type: 'admin',
    });
  }
  
  // 获取会话类型标签
  getTypeLabel(): string {
    const labels: Record<SessionType, string> = {
      user: '用户',
      guest: '访客',
      admin: '管理员',
      system: '系统',
      api: 'API',
    };
    return labels[this.type];
  }
  
  // 获取状态标签
  getStatusLabel(): string {
    const labels: Record<SessionStatus, string> = {
      active: '活跃',
      inactive: '不活跃',
      closed: '已关闭',
      expired: '已过期',
      error: '错误',
    };
    return labels[this.status];
  }
  
  // 获取会话摘要
  getSummary(): string {
    return `会话 ${this.id.substring(0, 8)}... (${this.getTypeLabel()} - ${this.getStatusLabel()})`;
  }
}

// 会话过滤器类型
export interface SessionFilter {
  userId?: string;
  type?: SessionType | SessionType[];
  status?: SessionStatus | SessionStatus[];
  agentId?: string;
  channelId?: string;
  isActive?: boolean;
  isExpired?: boolean;
  isIdle?: boolean;
  startTime?: Date;
  endTime?: Date;
  search?: string;
}

// 会话排序选项
export type SessionSortOption = 'createdAt' | 'updatedAt' | 'lastActivityAt' | 'expiresAt';

// 会话分页结果
export interface SessionPaginationResult {
  sessions: Session[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  activeCount: number;
  expiredCount: number;
}

// 会话统计信息
export interface SessionStatistics {
  totalSessions: number;
  activeSessions: number;
  averageSessionDuration: number; // 分钟
  averageMessagesPerSession: number;
  averageTokensPerSession: number;
  mostActiveTime: string; // 最活跃时间段
  retentionRate: number; // 留存率
}

// 所有类型已经通过export语句导出
// 不需要额外的export type语句
