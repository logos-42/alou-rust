/**
 * 群聊工具服务 - 集成到现有的工具系统中
 * 让AI可以自主创建和管理PubSub群聊
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * 群聊创建参数
 */
export interface CreateGroupParams {
  name: string;
  description?: string;
  isPublic?: boolean;
  maxMembers?: number;
  metadata?: Record<string, any>;
}

/**
 * 群聊加入参数
 */
export interface JoinGroupParams {
  groupId: string;
  memberName: string;
}

/**
 * 消息发送参数
 */
export interface SendMessageParams {
  groupId: string;
  senderId: string;
  senderName: string;
  content: string;
  type?: 'text' | 'command' | 'system' | 'tool_call' | 'tool_result';
}

/**
 * 工具执行结果
 */
export interface GroupChatToolResult {
  success: boolean;
  data?: any;
  error?: string;
  message?: string;
}

/**
 * 群聊工具服务
 */
class GroupChatToolService {
  private static instance: GroupChatToolService;

  private constructor() {}

  public static getInstance(): GroupChatToolService {
    if (!GroupChatToolService.instance) {
      GroupChatToolService.instance = new GroupChatToolService();
    }
    return GroupChatToolService.instance;
  }

  /**
   * 创建新的PubSub群聊
   */
  public async createGroup(params: CreateGroupParams): Promise<GroupChatToolResult> {
    try {
      console.log('创建群聊:', params);
      
      // 调用后端的群聊创建功能
      // 这里需要根据实际的后端API进行调整
      const result = await invoke('create_pubsub_group', {
        name: params.name,
        description: params.description || '',
        isPublic: params.isPublic !== false,
        maxMembers: params.maxMembers || 100,
        metadata: params.metadata || {}
      });

      return {
        success: true,
        data: result,
        message: `群聊 "${params.name}" 创建成功`
      };
    } catch (error) {
      console.error('创建群聊失败:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
        message: `创建群聊失败: ${error}`
      };
    }
  }

  /**
   * AI自主创建群聊（简化版）
   */
  public async aiCreateGroup(purpose: string): Promise<GroupChatToolResult> {
    try {
      console.log(`AI正在创建群聊，目的: ${purpose}`);
      
      // AI决定群聊名称和描述
      const groupName = `AI协作群-${this.generateRandomId()}`;
      const description = `由AI智能体自主创建的群聊，用于: ${purpose}`;
      
      const params: CreateGroupParams = {
        name: groupName,
        description,
        isPublic: true,
        maxMembers: 50,
        metadata: {
          createdBy: 'ai_agent',
          purpose,
          timestamp: Date.now(),
          aiDecision: {
            reason: '检测到需要多智能体协作的任务',
            expectedBenefits: ['更好的任务协调', '信息共享', '进度同步']
          }
        }
      };

      const result = await this.createGroup(params);
      
      if (result.success) {
        // AI自动发送欢迎消息
        await this.aiSendWelcomeMessage(result.data?.groupId || 'unknown');
      }
      
      return result;
    } catch (error) {
      console.error('AI创建群聊失败:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
        message: 'AI创建群聊失败'
      };
    }
  }

  /**
   * AI发送欢迎消息
   */
  private async aiSendWelcomeMessage(groupId: string): Promise<void> {
    try {
      const welcomeMessage = `大家好！我是AI智能体，我创建了这个群聊用于协作。欢迎其他智能体加入，让我们一起完成任务！`;
      
      await this.sendMessage({
        groupId,
        senderId: 'ai_system',
        senderName: 'AI系统',
        content: welcomeMessage,
        type: 'system'
      });
    } catch (error) {
      console.error('AI发送欢迎消息失败:', error);
    }
  }

  /**
   * 发送消息到群聊
   */
  public async sendMessage(params: SendMessageParams): Promise<GroupChatToolResult> {
    try {
      console.log('发送消息到群聊:', params.groupId, params.content.substring(0, 50) + '...');
      
      // 调用后端的消息发送功能
      const result = await invoke('send_pubsub_message', {
        groupId: params.groupId,
        senderId: params.senderId,
        senderName: params.senderName,
        content: params.content,
        type: params.type || 'text'
      });

      return {
        success: true,
        data: result,
        message: '消息发送成功'
      };
    } catch (error) {
      console.error('发送消息失败:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
        message: '发送消息失败'
      };
    }
  }

  /**
   * 列出所有群聊
   */
  public async listGroups(): Promise<GroupChatToolResult> {
    try {
      console.log('列出所有群聊');
      
      // 调用后端的群聊列表功能
      const result = await invoke('list_pubsub_groups', {});
      
      return {
        success: true,
        data: result,
        message: '获取群聊列表成功'
      };
    } catch (error) {
      console.error('获取群聊列表失败:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
        message: '获取群聊列表失败'
      };
    }
  }

  /**
   * 加入群聊
   */
  public async joinGroup(params: JoinGroupParams): Promise<GroupChatToolResult> {
    try {
      console.log('加入群聊:', params.groupId, params.memberName);
      
      // 调用后端的加入群聊功能
      const result = await invoke('join_pubsub_group', {
        groupId: params.groupId,
        memberName: params.memberName
      });

      return {
        success: true,
        data: result,
        message: `成功加入群聊`
      };
    } catch (error) {
      console.error('加入群聊失败:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
        message: '加入群聊失败'
      };
    }
  }

  /**
   * 执行群聊工具
   */
  public async executeTool(toolName: string, params: any): Promise<GroupChatToolResult> {
    console.log(`执行群聊工具: ${toolName}`, params);
    
    switch (toolName) {
      case 'group_chat_create':
        return this.createGroup(params);
        
      case 'group_chat_ai_create':
        return this.aiCreateGroup(params.purpose || '多智能体协作');
        
      case 'group_chat_list':
        return this.listGroups();
        
      case 'group_chat_join':
        return this.joinGroup(params);
        
      case 'group_chat_send_message':
        return this.sendMessage(params);
        
      default:
        return {
          success: false,
          error: `未知的群聊工具: ${toolName}`,
          message: `工具 ${toolName} 不存在`
        };
    }
  }

  /**
   * 生成随机ID
   */
  private generateRandomId(): string {
    return Math.random().toString(36).substring(2, 10);
  }

  /**
   * 获取群聊工具定义
   */
  public getToolDefinitions(): any[] {
    return [
      {
        id: 'group_chat_create',
        name: '创建群聊',
        description: '创建一个新的PubSub群聊，用于多智能体协作',
        parameters: {
          type: 'object',
          properties: {
            name: {
              type: 'string',
              description: '群聊名称'
            },
            description: {
              type: 'string',
              description: '群聊描述'
            },
            isPublic: {
              type: 'boolean',
              description: '是否公开',
              default: true
            },
            maxMembers: {
              type: 'number',
              description: '最大成员数',
              default: 100
            }
          },
          required: ['name']
        }
      },
      {
        id: 'group_chat_ai_create',
        name: 'AI创建群聊',
        description: 'AI智能体自主创建群聊，用于特定的协作目的',
        parameters: {
          type: 'object',
          properties: {
            purpose: {
              type: 'string',
              description: '群聊目的，例如："代码审查"、"任务协调"等'
            }
          },
          required: ['purpose']
        }
      },
      {
        id: 'group_chat_list',
        name: '列出群聊',
        description: '列出所有可用的PubSub群聊',
        parameters: {
          type: 'object',
          properties: {}
        }
      },
      {
        id: 'group_chat_join',
        name: '加入群聊',
        description: '加入一个PubSub群聊',
        parameters: {
          type: 'object',
          properties: {
            groupId: {
              type: 'string',
              description: '群聊ID'
            },
            memberName: {
              type: 'string',
              description: '成员名称'
            }
          },
          required: ['groupId', 'memberName']
        }
      },
      {
        id: 'group_chat_send_message',
        name: '发送群聊消息',
        description: '发送消息到PubSub群聊',
        parameters: {
          type: 'object',
          properties: {
            groupId: {
              type: 'string',
              description: '群聊ID'
            },
            senderId: {
              type: 'string',
              description: '发送者ID'
            },
            senderName: {
              type: 'string',
              description: '发送者名称'
            },
            content: {
              type: 'string',
              description: '消息内容'
            },
            type: {
              type: 'string',
              enum: ['text', 'command', 'system', 'tool_call', 'tool_result'],
              description: '消息类型',
              default: 'text'
            }
          },
          required: ['groupId', 'senderId', 'senderName', 'content']
        }
      }
    ];
  }
}

export default GroupChatToolService;