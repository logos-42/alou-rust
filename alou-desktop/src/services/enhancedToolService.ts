/**
 * 增强版工具服务 - 集成群聊工具
 * 扩展原有的工具服务，添加群聊创建和管理功能
 */

import { ToolService, ToolExecutionOptions, LocalToolResult, RemoteToolResult } from './toolService';
import GroupChatToolService, { GroupChatToolResult } from './groupChatToolService';

/**
 * 增强版工具服务
 */
class EnhancedToolService extends ToolService {
  private groupChatService: GroupChatToolService;
  
  constructor() {
    super();
    this.groupChatService = GroupChatToolService.getInstance();
  }

  /**
   * 执行工具（增强版）
   */
  public async executeToolEnhanced(
    toolId: string,
    params: any,
    options: ToolExecutionOptions = {}
  ): Promise<LocalToolResult | RemoteToolResult> {
    // 检查是否是群聊工具
    if (toolId.startsWith('group_chat_')) {
      return this.executeGroupChatTool(toolId, params, options);
    }
    
    // 否则调用父类方法
    return super.executeTool(toolId, params, options);
  }

  /**
   * 执行群聊工具
   */
  private async executeGroupChatTool(
    toolId: string,
    params: any,
    options: ToolExecutionOptions
  ): Promise<LocalToolResult> {
    const startTime = Date.now();
    
    try {
      console.log(`执行群聊工具: ${toolId}`, params);
      
      // 调用群聊服务
      const result = await this.groupChatService.executeTool(toolId, params);
      
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
      
      console.error(`群聊工具执行失败: ${toolId}`, error);
      
      return {
        success: false,
        execution_time_ms: executionTime,
        error: errorMessage,
        output: `群聊工具执行失败: ${errorMessage}`
      };
    }
  }

  /**
   * 获取所有可用工具（包含群聊工具）
   */
  public async getAvailableToolsEnhanced(): Promise<any[]> {
    // 获取父类的工具列表
    const parentTools = await super.getAvailableTools();
    
    // 添加群聊工具
    const groupChatTools = this.groupChatService.getToolDefinitions();
    
    return [...parentTools, ...groupChatTools];
  }

  /**
   * AI自主创建群聊演示
   */
  public async demoAiCreateGroup(): Promise<LocalToolResult> {
    console.log('🤖 演示：AI自主创建群聊');
    
    // 模拟AI决策过程
    const aiReasoning = `
作为AI智能体，我检测到以下情况：
1. 当前有多个相关任务需要协调
2. 涉及多个技能领域（代码、测试、部署）
3. 需要实时沟通和进度同步

决策：创建群聊以改善协作效率
`;
    
    console.log('AI推理:', aiReasoning);
    
    const purpose = '多智能体项目开发协作';
    
    const result = await this.executeGroupChatTool('group_chat_ai_create', { purpose }, {});
    
    if (result.success) {
      console.log('🎉 AI成功创建群聊！');
      console.log('群聊数据:', result.data);
      
      // AI可以进一步操作，比如邀请其他智能体
      console.log('🤖 AI正在考虑邀请其他智能体加入...');
    }
    
    return result;
  }

  /**
   * 测试群聊工具
   */
  public async testGroupChatTools(): Promise<LocalToolResult[]> {
    const results: LocalToolResult[] = [];
    
    console.log('🧪 测试群聊工具...');
    
    // 测试1：创建群聊
    const createResult = await this.executeGroupChatTool('group_chat_create', {
      name: '测试群聊',
      description: '用于测试群聊工具功能',
      isPublic: true,
      maxMembers: 10
    }, {});
    
    results.push(createResult);
    
    if (createResult.success && createResult.data?.groupId) {
      const groupId = createResult.data.groupId;
      
      // 测试2：发送消息
      const messageResult = await this.executeGroupChatTool('group_chat_send_message', {
        groupId,
        senderId: 'test_user',
        senderName: '测试用户',
        content: '这是一条测试消息，来自群聊工具测试'
      }, {});
      
      results.push(messageResult);
      
      // 测试3：列出群聊
      const listResult = await this.executeGroupChatTool('group_chat_list', {}, {});
      results.push(listResult);
    }
    
    console.log(`测试完成，共执行 ${results.length} 个操作`);
    
    return results;
  }
}

// 创建单例实例
const enhancedToolService = new EnhancedToolService();

export default enhancedToolService;
export { EnhancedToolService };