/**
 * Channel 实体类
 * 表示聊天频道/对话通道
 */

// 简单的ID生成函数
function generateId(): string {
  return 'channel_' + Date.now().toString(36) + Math.random().toString(36).substr(2, 9);
}

// 频道状态类型
export type ChannelStatus = 'online' | 'offline' | 'busy' | 'error' | 'maintenance';

// 频道模式类型
export type ChannelMode = 'single' | 'group' | 'broadcast' | 'private';

// 频道元数据
export interface ChannelMeta {
  mode?: ChannelMode;
  isPinned?: boolean;
  isArchived?: boolean;
  isMuted?: boolean;
  lastReadMessageId?: string;
  unreadCount?: number;
  customSettings?: Record<string, any>;
  participants?: string[]; // 参与者ID列表
  tags?: string[];
}

// 频道统计信息
export interface ChannelStats {
  messageCount: number;
  participantCount: number;
  lastActivity: Date;
  averageResponseTime?: number; // 平均响应时间(ms)
  successRate?: number; // 成功率(0-1)
}

// 频道实体类
export class Channel {
  // 核心属性
  readonly id: string;
  name: string;
  agentId: string;
  status: ChannelStatus;
  statusLabel: string;
  icon: string;
  color: string;
  
  // 扩展属性
  description?: string;
  meta: ChannelMeta;
  stats: ChannelStats;
  
  // 时间戳
  readonly createdAt: Date;
  updatedAt: number; // 使用时间戳便于排序
  
  // 构造函数
  constructor(data: Partial<Channel> & { name: string; agentId: string }) {
    this.id = data.id || generateId();
    this.name = data.name;
    this.agentId = data.agentId;
    this.status = data.status || 'online';
    this.statusLabel = data.statusLabel || this.getDefaultStatusLabel(data.status);
    this.icon = data.icon || this.getDefaultIcon();
    this.color = data.color || this.getDefaultColor();
    this.description = data.description;
    
    // 默认元数据
    this.meta = data.meta || {
      mode: 'single',
      isPinned: false,
      isArchived: false,
      isMuted: false,
      unreadCount: 0,
      participants: [],
      tags: [],
    };
    
    // 默认统计信息
    this.stats = data.stats || {
      messageCount: 0,
      participantCount: 1,
      lastActivity: new Date(),
      averageResponseTime: undefined,
      successRate: undefined,
    };
    
    // 时间戳
    const now = new Date();
    this.createdAt = data.createdAt || now;
    this.updatedAt = data.updatedAt || now.getTime();
  }
  
  // 获取默认状态标签
  private getDefaultStatusLabel(status?: ChannelStatus): string {
    const labels: Record<ChannelStatus, string> = {
      online: '在线',
      offline: '离线',
      busy: '忙碌',
      error: '错误',
      maintenance: '维护中',
    };
    return status ? labels[status] : labels.online;
  }
  
  // 获取默认图标
  private getDefaultIcon(): string {
    return '💬'; // 默认聊天图标
  }
  
  // 获取默认颜色
  private getDefaultColor(): string {
    const colors = ['#4CAF50', '#2196F3', '#9C27B0', '#FF9800', '#F44336'];
    const index = Math.floor(Math.random() * colors.length);
    return colors[index];
  }
  
  // 更新频道信息
  update(updates: Partial<Omit<Channel, 'id' | 'createdAt'>>): this {
    if (updates.name !== undefined) this.name = updates.name;
    if (updates.agentId !== undefined) this.agentId = updates.agentId;
    if (updates.status !== undefined) {
      this.status = updates.status;
      this.statusLabel = updates.statusLabel || this.getDefaultStatusLabel(updates.status);
    }
    if (updates.statusLabel !== undefined) this.statusLabel = updates.statusLabel;
    if (updates.icon !== undefined) this.icon = updates.icon;
    if (updates.color !== undefined) this.color = updates.color;
    if (updates.description !== undefined) this.description = updates.description;
    if (updates.meta !== undefined) this.meta = { ...this.meta, ...updates.meta };
    if (updates.stats !== undefined) this.stats = { ...this.stats, ...updates.stats };
    
    this.updatedAt = Date.now();
    return this;
  }
  
  // 更新活动时间
  updateActivity(): this {
    this.stats.lastActivity = new Date();
    this.updatedAt = Date.now();
    return this;
  }
  
  // 增加消息计数
  incrementMessageCount(): this {
    this.stats.messageCount += 1;
    this.updateActivity();
    return this;
  }
  
  // 更新未读计数
  updateUnreadCount(count: number): this {
    this.meta.unreadCount = count;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 增加未读计数
  incrementUnreadCount(): this {
    this.meta.unreadCount = (this.meta.unreadCount || 0) + 1;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 重置未读计数
  resetUnreadCount(): this {
    this.meta.unreadCount = 0;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 切换置顶状态
  togglePin(): this {
    this.meta.isPinned = !this.meta.isPinned;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 切换静音状态
  toggleMute(): this {
    this.meta.isMuted = !this.meta.isMuted;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 归档频道
  archive(): this {
    this.meta.isArchived = true;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 取消归档
  unarchive(): this {
    this.meta.isArchived = false;
    this.updatedAt = Date.now();
    return this;
  }
  
  // 添加参与者
  addParticipant(participantId: string): this {
    if (!this.meta.participants) {
      this.meta.participants = [];
    }
    
    if (!this.meta.participants.includes(participantId)) {
      this.meta.participants.push(participantId);
      this.stats.participantCount = this.meta.participants.length;
      this.updatedAt = Date.now();
    }
    
    return this;
  }
  
  // 移除参与者
  removeParticipant(participantId: string): this {
    if (this.meta.participants) {
      const index = this.meta.participants.indexOf(participantId);
      if (index > -1) {
        this.meta.participants.splice(index, 1);
        this.stats.participantCount = this.meta.participants.length;
        this.updatedAt = Date.now();
      }
    }
    
    return this;
  }
  
  // 添加标签
  addTag(tag: string): this {
    if (!this.meta.tags) {
      this.meta.tags = [];
    }
    
    if (!this.meta.tags.includes(tag)) {
      this.meta.tags.push(tag);
      this.updatedAt = Date.now();
    }
    
    return this;
  }
  
  // 移除标签
  removeTag(tag: string): this {
    if (this.meta.tags) {
      const index = this.meta.tags.indexOf(tag);
      if (index > -1) {
        this.meta.tags.splice(index, 1);
        this.updatedAt = Date.now();
      }
    }
    
    return this;
  }
  
  // 验证频道数据
  validate(): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 验证名称
    if (!this.name || this.name.trim().length === 0) {
      errors.push('频道名称不能为空');
    }
    
    if (this.name.length > 50) {
      errors.push('频道名称不能超过50个字符');
    }
    
    // 验证代理ID
    if (!this.agentId || this.agentId.trim().length === 0) {
      errors.push('代理ID不能为空');
    }
    
    // 验证描述
    if (this.description && this.description.length > 500) {
      errors.push('频道描述不能超过500个字符');
    }
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
  
  // 检查频道是否活跃
  isActive(): boolean {
    const oneHourAgo = Date.now() - 60 * 60 * 1000;
    return this.stats.lastActivity.getTime() > oneHourAgo;
  }
  
  // 检查是否有未读消息
  hasUnreadMessages(): boolean {
    return (this.meta.unreadCount || 0) > 0;
  }
  
  // 转换为纯对象（用于序列化）
  toJSON(): Record<string, any> {
    return {
      id: this.id,
      name: this.name,
      agentId: this.agentId,
      status: this.status,
      statusLabel: this.statusLabel,
      icon: this.icon,
      color: this.color,
      description: this.description,
      meta: this.meta,
      stats: {
        ...this.stats,
        lastActivity: this.stats.lastActivity.toISOString(),
      },
      createdAt: this.createdAt.toISOString(),
      updatedAt: this.updatedAt,
    };
  }
  
  // 从纯对象创建实例（用于反序列化）
  static fromJSON(data: Record<string, any>): Channel {
    return new Channel({
      id: data.id,
      name: data.name,
      agentId: data.agentId,
      status: data.status,
      statusLabel: data.statusLabel,
      icon: data.icon,
      color: data.color,
      description: data.description,
      meta: data.meta,
      stats: {
        ...data.stats,
        lastActivity: data.stats?.lastActivity ? new Date(data.stats.lastActivity) : new Date(),
      },
      createdAt: data.createdAt ? new Date(data.createdAt) : new Date(),
      updatedAt: data.updatedAt || Date.now(),
    });
  }
  
  // 创建示例频道
  static createExample(agentId: string, name: string = '示例频道'): Channel {
    return new Channel({
      name,
      agentId,
      description: '这是一个示例频道',
      icon: '🌟',
      color: '#FF9800',
      meta: {
        mode: 'single',
        isPinned: true,
        tags: ['example', 'demo'],
        participants: ['user-123'],
      },
      stats: {
        messageCount: 42,
        participantCount: 1,
        lastActivity: new Date(),
        averageResponseTime: 1500,
        successRate: 0.95,
      },
    });
  }
  
  // 根据状态获取CSS类名
  getStatusClass(): string {
    const classes: Record<ChannelStatus, string> = {
      online: 'status-online',
      offline: 'status-offline',
      busy: 'status-busy',
      error: 'status-error',
      maintenance: 'status-maintenance',
    };
    return classes[this.status];
  }
  
  // 获取显示时间（相对时间）
  getDisplayTime(): string {
    const now = Date.now();
    const diff = now - this.updatedAt;
    
    if (diff < 60000) { // 1分钟内
      return '刚刚';
    } else if (diff < 3600000) { // 1小时内
      const minutes = Math.floor(diff / 60000);
      return `${minutes}分钟前`;
    } else if (diff < 86400000) { // 1天内
      const hours = Math.floor(diff / 3600000);
      return `${hours}小时前`;
    } else {
      const days = Math.floor(diff / 86400000);
      return `${days}天前`;
    }
  }
}

// 频道过滤器类型
export interface ChannelFilter {
  status?: ChannelStatus | ChannelStatus[];
  agentId?: string;
  mode?: ChannelMode;
  isPinned?: boolean;
  isArchived?: boolean;
  isMuted?: boolean;
  hasUnread?: boolean;
  tags?: string[];
  search?: string;
  minMessageCount?: number;
  maxMessageCount?: number;
}

// 频道排序选项
export type ChannelSortOption = 'name' | 'updatedAt' | 'createdAt' | 'messageCount' | 'lastActivity';

// 频道分页结果
export interface ChannelPaginationResult {
  channels: Channel[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

// 所有类型已经通过export语句导出
// 不需要额外的export type语句
