/**
 * PubSub群聊工具服务
 * 让Alou可以自主创建和管理IPFS PubSub群聊
 * 无需钱包验证，直接调用工具创建
 */

import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';

/**
 * PubSub群聊配置
 */
export interface PubSubGroupConfig {
  /** 群聊名称 */
  name: string;
  /** 群聊描述 */
  description?: string;
  /** 主题名称 (自动生成如果未提供) */
  topic?: string;
  /** 是否公开 */
  isPublic?: boolean;
  /** 最大成员数 */
  maxMembers?: number;
  /** 元数据 */
  metadata?: Record<string, any>;
}

/**
 * 群聊成员
 */
export interface GroupMember {
  /** 成员ID */
  id: string;
  /** 成员名称 */
  name: string;
  /** 角色 */
  role: 'admin' | 'member' | 'observer';
  /** 加入时间 */
  joinedAt: number;
}

/**
 * PubSub群聊
 */
export interface PubSubGroup {
  /** 群聊ID */
  id: string;
  /** 群聊名称 */
  name: string;
  /** 群聊描述 */
  description?: string;
  /** 主题名称 */
  topic: string;
  /** 创建者ID */
  creatorId: string;
  /** 创建时间 */
  createdAt: number;
  /** 是否活跃 */
  isActive: boolean;
  /** 成员列表 */
  members: GroupMember[];
  /** 消息数量 */
  messageCount: number;
  /** 最后活动时间 */
  lastActivity: number;
  /** 元数据 */
  metadata: Record<string, any>;
}

/**
 * PubSub消息
 */
export interface PubSubMessage {
  /** 消息ID */
  id: string;
  /** 发送者ID */
  senderId: string;
  /** 发送者名称 */
  senderName: string;
  /** 消息内容 */
  content: string;
  /** 消息类型 */
  type: 'text' | 'command' | 'system' | 'tool_call' | 'tool_result';
  /** 时间戳 */
  timestamp: number;
  /** 元数据 */
  metadata?: Record<string, any>;
}

/**
 * 工具执行结果
 */
export interface ToolExecutionResult {
  success: boolean;
  data?: any;
  error?: string;
  executionTime?: number;
  group?: PubSubGroup;
  message?: string;
}

/**
 * PubSub群聊工具服务
 */
class PubSubToolService {
  private static instance: PubSubToolService;
  private groups: Map<string, PubSubGroup> = new Map();
  private messageListeners: Map<string, ((message: PubSubMessage) => void)[]> = new Map();
  private groupUpdateListeners: ((group: PubSubGroup) => void)[] = [];

  private constructor() {
    this.loadGroupsFromStorage();
  }

  /**
   * 获取单例实例
   */
  public static getInstance(): PubSubToolService {
    if (!PubSubToolService.instance) {
      PubSubToolService.instance = new PubSubToolService();
    }
    return PubSubToolService.instance;
  }

  /**
   * 从存储加载群聊
   */
  private async loadGroupsFromStorage(): Promise<void> {
    try {
      const stored = localStorage.getItem('alou_pubsub_groups');
      if (stored) {
        const groups = JSON.parse(stored);
        groups.forEach((group: PubSubGroup) => {
          this.groups.set(group.id, group);
        });
        console.log(`Loaded ${groups.length} PubSub groups from storage`);
      }
    } catch (error) {
      console.error('Failed to load PubSub groups from storage:', error);
    }
  }

  /**
   * 保存群聊到存储
   */
  private async saveGroupsToStorage(): Promise<void> {
    try {
      const groups = Array.from(this.groups.values());
      localStorage.setItem('alou_pubsub_groups', JSON.stringify(groups));
    } catch (error) {
      console.error('Failed to save PubSub groups to storage:', error);
    }
  }

  /**
   * 生成群聊ID
   */
  private generateGroupId(): string {
    return `group_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 生成主题名称
   */
  private generateTopic(groupName: string): string {
    const sanitizedName = groupName
      .toLowerCase()
      .replace(/[^a-z0-9]/g, '_')
      .substring(0, 20);
    return `/alou/pubsub/${sanitizedName}_${Date.now()}`;
  }

  /**
   * 创建新的PubSub群聊
   */
  public async createGroup(config: PubSubGroupConfig): Promise<ToolExecutionResult> {
    const startTime = Date.now();
    
    try {
      console.log('Creating PubSub group with config:', config);
      
      // 生成群聊ID和主题
      const groupId = this.generateGroupId();
      const topic = config.topic || this.generateTopic(config.name);
      
      // 创建群聊对象
      const group: PubSubGroup = {
        id: groupId,
        name: config.name,
        description: config.description,
        topic,
        creatorId: 'system', // 系统创建，无需钱包
        createdAt: Date.now(),
        isActive: true,
        members: [
          {
            id: 'system',
            name: 'System',
            role: 'admin',
            joinedAt: Date.now()
          }
        ],
        messageCount: 0,
        lastActivity: Date.now(),
        metadata: {
          ...config.metadata,
          isPublic: config.isPublic ?? true,
          maxMembers: config.maxMembers ?? 100,
          createdByTool: true
        }
      };

      // 保存群聊
      this.groups.set(groupId, group);
      await this.saveGroupsToStorage();

      // 通知监听器
      this.notifyGroupUpdate(group);

      const executionTime = Date.now() - startTime;
      
      console.log(`PubSub group created successfully: ${groupId} (${executionTime}ms)`);
      
      return {
        success: true,
        data: { groupId, topic },
        group,
        executionTime,
        message: `群聊 "${config.name}" 创建成功！主题: ${topic}`
      };
      
    } catch (error) {
      const executionTime = Date.now() - startTime;
      console.error('Failed to create PubSub group:', error);
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        executionTime,
        message: `创建群聊失败: ${error}`
      };
    }
  }

  /**
   * 获取所有群聊
   */
  public async listGroups(): Promise<ToolExecutionResult> {
    const startTime = Date.now();
    
    try {
      const groups = Array.from(this.groups.values());
      const executionTime = Date.now() - startTime;
      
      return {
        success: true,
        data: { groups },
        executionTime,
        message: `找到 ${groups.length} 个群聊`
      };
    } catch (error) {
      const executionTime = Date.now() - startTime;
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        executionTime,
        message: '获取群聊列表失败'
      };
    }
  }

  /**
   * 加入群聊
   */
  public async joinGroup(groupId: string, memberName: string): Promise<ToolExecutionResult> {
    const startTime = Date.now();
    
    try {
      const group = this.groups.get(groupId);
      if (!group) {
        throw new Error(`群聊 ${groupId} 不存在`);
      }

      // 检查是否已满
      if (group.members.length >= (group.metadata.maxMembers || 100)) {
        throw new Error('群聊已满');
      }

      // 添加成员
      const memberId = `member_${Date.now()}_${Math.random().toString(36).substr(2, 6)}`;
      const newMember: GroupMember = {
        id: memberId,
        name: memberName,
        role: 'member',
        joinedAt: Date.now()
      };

      group.members.push(newMember);
      group.lastActivity = Date.now();
      
      this.groups.set(groupId, group);
      await this.saveGroupsToStorage();
      this.notifyGroupUpdate(group);

      const executionTime = Date.now() - startTime;
      
      return {
        success: true,
        data: { groupId, memberId },
        executionTime,
        message: `成功加入群聊 "${group.name}"`
      };
      
    } catch (error) {
      const executionTime = Date.now() - startTime;
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        executionTime,
        message: `加入群聊失败: ${error}`
      };
    }
  }

  /**
   * 发送消息到群聊
   */
  public async sendMessage(
    groupId: string, 
    senderId: string, 
    senderName: string, 
    content: string,
    type: PubSubMessage['type'] = 'text'
  ): Promise<ToolExecutionResult> {
    const startTime = Date.now();
    
    try {
      const group = this.groups.get(groupId);
      if (!group) {
        throw new Error(`群聊 ${groupId} 不存在`);
      }

      // 创建消息
      const message: PubSubMessage = {
        id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 6)}`,
        senderId,
        senderName,
        content,
        type,
        timestamp: Date.now()
      };

      // 更新群聊状态
      group.messageCount += 1;
      group.lastActivity = Date.now();
      this.groups.set(groupId, group);
      await this.saveGroupsToStorage();

      // 通知消息监听器
      this.notifyMessageListeners(groupId, message);

      const executionTime = Date.now() - startTime;
      
      return {
        success: true,
        data: { messageId: message.id },
        executionTime,
        message: '消息发送成功'
      };
      
    } catch (error) {
      const executionTime = Date.now() - startTime;
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        executionTime,
        message: `发送消息失败: ${error}`
      };
    }
  }

  /**
   * 获取群聊消息
   */
  public async getGroupMessages(groupId: string, limit: number = 50): Promise<ToolExecutionResult> {
    const startTime = Date.now();
    
    try {
      // 这里应该从实际存储中获取消息
      // 目前返回模拟数据
      const messages: PubSubMessage[] = [
        {
          id: 'welcome_msg',
          senderId: 'system',
          senderName: 'System',
          content: `欢迎来到群聊！这是一个基于IPFS PubSub的去中心化聊天室。`,
          type: 'system',
          timestamp: Date.now() - 1000
        }
      ];

      const executionTime = Date.now() - startTime;
      
      return {
        success: true,
        data: { messages },
        executionTime,
        message: `获取到 ${messages.length} 条消息`
      };
      
    } catch (error) {
      const executionTime = Date.now() - startTime;
      
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        executionTime,
        message: `获取消息失败: ${error}`
      };
    }
  }

  /**
   * 添加消息监听器
   */
  public addMessageListener(groupId: string, callback: (message: PubSubMessage) => void): void {
    if (!this.messageListeners.has(groupId)) {
      this.messageListeners.set(groupId, []);
    }
    this.messageListeners.get(groupId)!.push(callback);
  }

  /**
   * 移除消息监听器
   */
  public removeMessageListener(groupId: string, callback: (message: PubSubMessage) => void): void {
    const listeners = this.messageListeners.get(groupId);
    if (listeners) {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  /**
   * 添加群聊更新监听器
   */
  public addGroupUpdateListener(callback: (group: PubSubGroup) => void): void {
    this.groupUpdateListeners.push(callback);
  }

  /**
   * 移除群聊更新监听器
   */
  public removeGroupUpdateListener(callback: (group: PubSubGroup) => void): void {
    const index = this.groupUpdateListeners.indexOf(callback);
    if (index > -1) {
      this.groupUpdateListeners.splice(index, 1);
    }
  }

  /**
   * 通知消息监听器
   */
  private notifyMessageListeners(groupId: string, message: PubSubMessage): void {
    const listeners = this.messageListeners.get(groupId);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(message);
        } catch (error) {
          console.error('Error in message listener:', error);
        }
      });
    }
  }

  /**
   * 通知群聊更新监听器
   */
  private notifyGroupUpdate(group: PubSubGroup): void {
    this.groupUpdateListeners.forEach(callback => {
      try {
        callback(group);
      } catch (error) {
        console.error('Error in group update listener:', error);
      }
    });
  }

  /**
   * 工具调用接口 - 供AI智能体调用
   */
  public async executeTool(toolName: string, params: any): Promise<ToolExecutionResult> {
    console.log(`Executing PubSub tool: ${toolName}`, params);
    
    switch (toolName) {
      case 'pubsub_create_group':
        return this.createGroup(params);
        
      case 'pubsub_list_groups':
        return this.listGroups();
        
      case 'pubsub_join_group':
        return this.joinGroup(params.groupId, params.memberName);
        
      case 'pubsub_send_message':
        return this.sendMessage(
          params.groupId,
          params.senderId,
          params.senderName,
          params.content,
          params.type
        );
        
      case 'pubsub_get_messages':
        return this.getGroupMessages(params.groupId, params.limit);
        
      default:
        return {
          success: false,
          error: `未知的工具: ${toolName}`,
          message: `工具 ${toolName} 不存在`
        };
    }
  }
}

export default PubSubToolService;