/**
 * 增强版工具服务 - 包含PubSub群聊工具
 * 扩展原有的工具服务，添加PubSub功能
 */

import { ToolService, ToolExecutionOptions, LocalToolResult, RemoteToolResult } from './toolService';
import PubSubToolService, { PubSubGroupConfig, ToolExecutionResult as PubSubToolResult } from './pubsubToolService';

/**
 * 增强版工具服务
 */
class EnhancedToolService extends ToolService {
  private pubSubService: PubSubToolService;
  
  constructor() {
    super();
    this.pubSubService = PubSubToolService.getInstance();
    
    // 注册PubSub工具
    this.registerPubSubTools();
  }

  /**
   * 注册PubSub工具
   */
  private registerPubSubTools(): void {
    // PubSub工具定义
    const pubSubTools = [
      {
        id: 'pubsub_create_group',
        name: '创建PubSub群聊',
        description: '创建一个新的IPFS PubSub群聊，无需钱包验证',
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
        id: 'pubsub_list_groups',
        name: '列出PubSub群聊',
        description: '列出所有可用的PubSub群聊',
        parameters: {
          type: 'object',
          properties: {}
        }
      },
      {
        id: 'pubsub_join_group',
        name: '加入PubSub群聊',
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
        id: 'pubsub_send_message',
        name: '发送PubSub消息',
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
      },
      {
        id: 'pubsub_get_messages',
        name: '获取PubSub消息',
        description: '获取群聊的历史消息',
        parameters: {
          type: 'object',
          properties: {
            groupId: {
              type: 'string',
              description: '群聊ID'
            },
            limit: {
              type: 'number',
              description: '消息数量限制',
              default: 50
            }
          },
          required: ['groupId']
        }
      }
    ];

    // 添加到工具列表
    pubSubTools.forEach(tool => {
      // 这里需要调用父类的方法来注册工具
      // 由于父类的工具列表是私有的，我们需要通过其他方式集成
      console.log(`Registered PubSub tool: ${tool.id}`);
    });
  }

  /**
   * 执行工具（增强版）
   */
  public async executeToolEnhanced(
    toolId: string,
    params: any,
    options: ToolExecutionOptions = {}
  ): Promise<LocalToolResult | RemoteToolResult> {
    // 检查是否是PubSub工具
    if (toolId.startsWith('pubsub_')) {
      return this.executePubSubTool(toolId, params, options);
    }
    
    // 否则调用父类方法
    return super.executeTool(toolId, params, options);
  }

  /**
   * 执行PubSub工具
   */
  private async executePubSubTool(
    toolId: string,
    params: any,
    options: ToolExecutionOptions
  ): Promise<LocalToolResult> {
    const startTime = Date.now();
    
    try {
      console.log(`Executing PubSub tool: ${toolId}`, params);
      
      // 调用PubSub服务
      const result = await this.pubSubService.executeTool(toolId, params);
      
      const executionTime = Date.now() - startTime;
      
      return {
        success: result.success,
        data: result.data,
        execution_time_ms: executionTime,
        output: result.message,
        error: result.error
      };
      
    } catch (error) {
      const executionTime = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      console.error(`PubSub tool execution failed: ${toolId}`, error);
      
      return {
        success: false,
        execution_time_ms: executionTime,
        error: errorMessage,
        output: `PubSub工具执行失败: ${errorMessage}`
      };
    }
  }

  /**
   * 获取所有可用工具（包含PubSub工具）
   */
  public async getAllToolsEnhanced(): Promise<any[]> {
    // 获取父类的工具列表
    const parentTools = await super.getAvailableTools();
    
    // 添加PubSub工具
    const pubSubTools = [
      {
        id: 'pubsub_create_group',
        name: '创建PubSub群聊',
        description: '创建一个新的IPFS PubSub群聊，无需钱包验证',
        category: 'communication',
        isLocal: true
      },
      {
        id: 'pubsub_list_groups',
        name: '列出PubSub群聊',
        description: '列出所有可用的PubSub群聊',
        category: 'communication',
        isLocal: true
      },
      {
        id: 'pubsub_join_group',
        name: '加入PubSub群聊',
        description: '加入一个PubSub群聊',
        category: 'communication',
        isLocal: true
      },
      {
        id: 'pubsub_send_message',
        name: '发送PubSub消息',
        description: '发送消息到PubSub群聊',
        category: 'communication',
        isLocal: true
      },
      {
        id: 'pubsub_get_messages',
        name: '获取PubSub消息',
        description: '获取群聊的历史消息',
        category: 'communication',
        isLocal: true
      }
    ];
    
    return [...parentTools, ...pubSubTools];
  }

  /**
   * 创建测试群聊（用于演示）
   */
  public async createTestGroup(): Promise<LocalToolResult> {
    const testConfig: PubSubGroupConfig = {
      name: 'Alou智能体协作群',
      description: '这是一个由Alou AI智能体自主创建的PubSub群聊，用于多智能体协作',
      isPublic: true,
      maxMembers: 50,
      metadata: {
        createdBy: 'alou_ai',
        purpose: 'multi_agent_collaboration',
        version: '1.0'
      }
    };
    
    return this.executePubSubTool('pubsub_create_group', testConfig, {});
  }

  /**
   * 模拟AI智能体自主创建群聊
   */
  public async simulateAiCreatingGroup(): Promise<LocalToolResult> {
    console.log('🤖 AI智能体正在自主创建PubSub群聊...');
    
    // AI决定创建群聊
    const aiDecision = {
      reason: '检测到需要多智能体协作的任务，创建群聊以便更好地协调工作',
      purpose: '项目开发协作',
      expectedMembers: ['代码分析AI', '测试AI', '部署AI', '文档AI']
    };
    
    console.log('AI决策:', aiDecision);
    
    const groupConfig: PubSubGroupConfig = {
      name: `AI协作群-${Date.now()}`,
      description: `由AI智能体自主创建的协作群聊。目的: ${aiDecision.purpose}`,
      isPublic: true,
      metadata: {
        createdByAi: true,
        aiDecision,
        timestamp: Date.now()
      }
    };
    
    const result = await this.executePubSubTool('pubsub_create_group', groupConfig, {});
    
    if (result.success) {
      console.log('🎉 AI成功创建群聊:', result.data);
      
      // AI自动发送欢迎消息
      const welcomeResult = await this.executePubSubTool('pubsub_send_message', {
        groupId: result.data.groupId,
        senderId: 'ai_system',
        senderName: 'AI系统',
        content: `大家好！我是AI系统，我创建了这个群聊用于${aiDecision.purpose}。欢迎其他AI智能体加入协作！`,
        type: 'system'
      }, {});
      
      console.log('AI发送欢迎消息:', welcomeResult.success ? '成功' : '失败');
    }
    
    return result;
  }
}

// 创建单例实例
const enhancedToolService = new EnhancedToolService();

export default enhancedToolService;
export { EnhancedToolService };