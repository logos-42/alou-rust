/**
 * Message 实体类
 * 表示聊天消息
 */

// 简单的ID生成函数
function generateId(): string {
  return 'msg_' + Date.now().toString(36) + Math.random().toString(36).substr(2, 9);
}

// 消息类型
export type MessageType = 'user' | 'assistant' | 'system' | 'tool' | 'error' | 'info';

// 消息状态
export type MessageStatus = 'sending' | 'sent' | 'delivered' | 'read' | 'failed' | 'cancelled';

// 消息内容类型
export type ContentType = 'text' | 'image' | 'file' | 'audio' | 'video' | 'markdown' | 'code';

// 消息内容
export interface MessageContent {
  type: ContentType;
  text?: string;
  url?: string;
  fileName?: string;
  fileSize?: number;
  mimeType?: string;
  metadata?: Record<string, any>;
}

// 消息元数据
export interface MessageMetadata {
  tokens?: number;
  model?: string;
  temperature?: number;
  reasoning?: string;
  citations?: Array<{
    source: string;
    text: string;
    page?: number;
  }>;
  toolCalls?: Array<{
    id: string;
    name: string;
    arguments: Record<string, any>;
    result?: any;
  }>;
  parentMessageId?: string;
  threadId?: string;
  customData?: Record<string, any>;
}

// 消息实体类
export class Message {
  // 核心属性
  readonly id: string;
  readonly channelId: string;
  type: MessageType;
  content: MessageContent;
  status: MessageStatus;
  
  // 发送者信息
  senderId: string;
  senderName?: string;
  senderAvatar?: string;
  
  // 元数据
  metadata: MessageMetadata;
  
  // 时间戳
  readonly timestamp: number;
  readonly createdAt: Date;
  updatedAt: Date;
  
  // 构造函数
  constructor(data: Partial<Message> & { 
    channelId: string; 
    type: MessageType; 
    content: MessageContent;
    senderId: string;
  }) {
    this.id = data.id || generateId();
    this.channelId = data.channelId;
    this.type = data.type;
    this.content = data.content;
    this.status = data.status || 'sent';
    this.senderId = data.senderId;
    this.senderName = data.senderName;
    this.senderAvatar = data.senderAvatar;
    
    // 默认元数据
    this.metadata = data.metadata || {};
    
    // 时间戳
    const now = new Date();
    this.timestamp = data.timestamp || now.getTime();
    this.createdAt = data.createdAt || now;
    this.updatedAt = data.updatedAt || now;
  }
  
  // 更新消息状态
  updateStatus(status: MessageStatus): this {
    this.status = status;
    this.updatedAt = new Date();
    return this;
  }
  
  // 更新消息内容
  updateContent(content: MessageContent): this {
    this.content = content;
    this.updatedAt = new Date();
    return this;
  }
  
  // 添加元数据
  addMetadata(metadata: Partial<MessageMetadata>): this {
    this.metadata = { ...this.metadata, ...metadata };
    this.updatedAt = new Date();
    return this;
  }
  
  // 添加工具调用结果
  addToolCallResult(toolCallId: string, result: any): this {
    if (!this.metadata.toolCalls) {
      this.metadata.toolCalls = [];
    }
    
    const toolCall = this.metadata.toolCalls.find(tc => tc.id === toolCallId);
    if (toolCall) {
      toolCall.result = result;
      this.updatedAt = new Date();
    }
    
    return this;
  }
  
  // 验证消息数据
  validate(): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 验证频道ID
    if (!this.channelId || this.channelId.trim().length === 0) {
      errors.push('频道ID不能为空');
    }
    
    // 验证发送者ID
    if (!this.senderId || this.senderId.trim().length === 0) {
      errors.push('发送者ID不能为空');
    }
    
    // 验证内容
    if (!this.content) {
      errors.push('消息内容不能为空');
    } else {
      // 验证文本内容
      if (this.content.type === 'text' || this.content.type === 'markdown' || this.content.type === 'code') {
        if (!this.content.text || this.content.text.trim().length === 0) {
          errors.push('文本内容不能为空');
        }
        
        if (this.content.text && this.content.text.length > 10000) {
          errors.push('文本内容不能超过10000个字符');
        }
      }
      
      // 验证文件内容
      if (this.content.type === 'file' || this.content.type === 'image' || this.content.type === 'audio' || this.content.type === 'video') {
        if (!this.content.url) {
          errors.push('文件URL不能为空');
        }
        
        if (!this.content.fileName) {
          errors.push('文件名不能为空');
        }
      }
    }
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
  
  // 检查消息是否来自用户
  isFromUser(): boolean {
    return this.type === 'user';
  }
  
  // 检查消息是否来自助手
  isFromAssistant(): boolean {
    return this.type === 'assistant';
  }
  
  // 检查消息是否包含工具调用
  hasToolCalls(): boolean {
    return !!(this.metadata.toolCalls && this.metadata.toolCalls.length > 0);
  }
  
  // 检查消息是否成功发送
  isSent(): boolean {
    return this.status === 'sent' || this.status === 'delivered' || this.status === 'read';
  }
  
  // 检查消息是否失败
  isFailed(): boolean {
    return this.status === 'failed';
  }
  
  // 获取消息预览（截断文本）
  getPreview(maxLength: number = 100): string {
    if (this.content.type === 'text' || this.content.type === 'markdown' || this.content.type === 'code') {
      const text = this.content.text || '';
      if (text.length <= maxLength) {
        return text;
      }
      return text.substring(0, maxLength) + '...';
    }
    
    // 文件类型预览
    const typeLabels: Record<ContentType, string> = {
      text: '文本',
      image: '图片',
      file: '文件',
      audio: '音频',
      video: '视频',
      markdown: 'Markdown',
      code: '代码',
    };
    
    return `[${typeLabels[this.content.type]}] ${this.content.fileName || '未命名文件'}`;
  }
  
  // 获取显示时间（格式化）
  getDisplayTime(format: 'relative' | 'absolute' = 'relative'): string {
    if (format === 'absolute') {
      return new Date(this.timestamp).toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
      });
    }
    
    // 相对时间
    const now = Date.now();
    const diff = now - this.timestamp;
    
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
  
  // 转换为纯对象（用于序列化）
  toJSON(): Record<string, any> {
    return {
      id: this.id,
      channelId: this.channelId,
      type: this.type,
      content: this.content,
      status: this.status,
      senderId: this.senderId,
      senderName: this.senderName,
      senderAvatar: this.senderAvatar,
      metadata: this.metadata,
      timestamp: this.timestamp,
      createdAt: this.createdAt.toISOString(),
      updatedAt: this.updatedAt.toISOString(),
    };
  }
  
  // 从纯对象创建实例（用于反序列化）
  static fromJSON(data: Record<string, any>): Message {
    return new Message({
      id: data.id,
      channelId: data.channelId,
      type: data.type,
      content: data.content,
      status: data.status,
      senderId: data.senderId,
      senderName: data.senderName,
      senderAvatar: data.senderAvatar,
      metadata: data.metadata,
      timestamp: data.timestamp,
      createdAt: data.createdAt ? new Date(data.createdAt) : new Date(),
      updatedAt: data.updatedAt ? new Date(data.updatedAt) : new Date(),
    });
  }
  
  // 创建文本消息
  static createTextMessage(
    channelId: string,
    senderId: string,
    text: string,
    type: MessageType = 'user'
  ): Message {
    return new Message({
      channelId,
      type,
      senderId,
      content: {
        type: 'text',
        text,
      },
    });
  }
  
  // 创建文件消息
  static createFileMessage(
    channelId: string,
    senderId: string,
    fileUrl: string,
    fileName: string,
    fileSize: number,
    mimeType: string
  ): Message {
    const type = this.getContentTypeFromMime(mimeType);
    
    return new Message({
      channelId,
      type: 'user',
      senderId,
      content: {
        type,
        url: fileUrl,
        fileName,
        fileSize,
        mimeType,
      },
    });
  }
  
  // 创建助手消息
  static createAssistantMessage(
    channelId: string,
    text: string,
    metadata?: MessageMetadata
  ): Message {
    return new Message({
      channelId,
      type: 'assistant',
      senderId: 'assistant',
      content: {
        type: 'text',
        text,
      },
      metadata,
    });
  }
  
  // 创建系统消息
  static createSystemMessage(
    channelId: string,
    text: string
  ): Message {
    return new Message({
      channelId,
      type: 'system',
      senderId: 'system',
      content: {
        type: 'text',
        text,
      },
    });
  }
  
  // 根据MIME类型获取内容类型
  private static getContentTypeFromMime(mimeType: string): ContentType {
    if (mimeType.startsWith('image/')) {
      return 'image';
    } else if (mimeType.startsWith('audio/')) {
      return 'audio';
    } else if (mimeType.startsWith('video/')) {
      return 'video';
    } else if (mimeType.includes('markdown') || mimeType.includes('md')) {
      return 'markdown';
    } else if (mimeType.includes('javascript') || mimeType.includes('typescript') || 
               mimeType.includes('python') || mimeType.includes('java') ||
               mimeType.includes('json') || mimeType.includes('xml')) {
      return 'code';
    } else {
      return 'file';
    }
  }
  
  // 获取消息类型标签
  getTypeLabel(): string {
    const labels: Record<MessageType, string> = {
      user: '用户',
      assistant: '助手',
      system: '系统',
      tool: '工具',
      error: '错误',
      info: '信息',
    };
    return labels[this.type];
  }
  
  // 获取状态标签
  getStatusLabel(): string {
    const labels: Record<MessageStatus, string> = {
      sending: '发送中',
      sent: '已发送',
      delivered: '已送达',
      read: '已读',
      failed: '发送失败',
      cancelled: '已取消',
    };
    return labels[this.status];
  }
  
  // 获取内容类型标签
  getContentTypeLabel(): string {
    const labels: Record<ContentType, string> = {
      text: '文本',
      image: '图片',
      file: '文件',
      audio: '音频',
      video: '视频',
      markdown: 'Markdown',
      code: '代码',
    };
    return labels[this.content.type];
  }
}

// 消息过滤器类型
export interface MessageFilter {
  channelId?: string;
  type?: MessageType | MessageType[];
  senderId?: string;
  status?: MessageStatus | MessageStatus[];
  contentType?: ContentType | ContentType[];
  startTime?: number;
  endTime?: number;
  search?: string;
  hasToolCalls?: boolean;
  parentMessageId?: string;
  threadId?: string;
}

// 消息排序选项
export type MessageSortOption = 'timestamp' | 'createdAt' | 'updatedAt';

// 消息分页结果
export interface MessagePaginationResult {
  messages: Message[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasMore: boolean;
}

// 消息批量操作结果
export interface MessageBatchResult {
  success: number;
  failed: number;
  errors: Array<{
    messageId: string;
    error: string;
  }>;
}

// 导出类型
export type {
  MessageType,
  MessageStatus,
  ContentType,
  MessageContent,
  MessageMetadata,
  MessageFilter,
  MessageSortOption,
  MessagePaginationResult,
  MessageBatchResult,
};
