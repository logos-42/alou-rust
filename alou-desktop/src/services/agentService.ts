/**
 * Agent Service - 与 Cloudflare Workers (alou-edge) 通信
 * 重构后：使用组合模式，将职责分离到不同的服务
 */
import apiClient from './api';
import agentResolverService from './agentResolverService';
import ipfsContentService from './ipfsContentService';
import asyncTaskService from './asyncTaskService';
import promptService from './promptService';
import { parseDidDocumentToAgent } from './didDocumentParser';

// 本地类型定义
interface AgentInfo {
  id: string;
  name: string;
  description?: string;
  mode?: string;
  did?: string;
  ipns?: string;
  capabilities?: string[];
  display_name?: string;
  role_description?: string;
  avatar?: string;
  avatar_url?: string;
  agent_type?: string;
  mcp_ports?: any;
  mcp_config?: any;
  status?: string;
}

interface ServiceResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp: string;
}

type ApiResponse<T> = ServiceResponse<T>;

interface SendMessageOptions {
  sessionId?: string;
  context?: Record<string, any>;
  priority?: 'low' | 'normal' | 'high';
  mode?: string;
  chain?: string;
  contextEvents?: any[];
  eventSummary?: string;
  useAsync?: boolean;
  timeout?: number;
  agentName?: string;
  roleDescription?: string;
  customInstructions?: string;
  customPrompt?: string;
  stream?: boolean;
}

interface Message {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: string;
  metadata?: Record<string, any>;
}

// @ts-ignore - Vite 环境变量
const DEFAULT_IPFS_API = (import.meta as any).env?.VITE_IPFS_API_URL || 'http://127.0.0.1:5001';
// @ts-ignore - Vite 环境变量
const DEFAULT_IPFS_GATEWAY = (import.meta as any).env?.VITE_IPFS_GATEWAY_URL || 'http://127.0.0.1:8080';

// 辅助函数：生成系统提示（使用统一Prompt服务）
const getSystemPromptForChat = async (agentInfo: AgentInfo, context: Record<string, any> = {}): Promise<string> => {
  console.log('[getSystemPromptForChat] 使用统一Prompt服务:', { agentInfo, context });

  try {
    // 获取可用工具列表
    const { default: toolService } = await import('./toolService');
    const tools = await toolService.getToolList();

    // 转换为 promptService 期望的格式
    const formattedTools = tools.map(tool => {
      // 优先使用工具服务提供的信息，否则使用注册表中的默认信息
      const toolId = tool.id || tool.name;
      return {
        name: toolId,
        description: tool.description || tool.desc || `${tool.name || toolId} - ${tool.category || '通用'} 工具`
      };
    }).filter(tool => tool.name); // 确保工具名称不为空

    // 使用PromptService生成动态Prompt
    const prompt = promptService.generateCustomAgentPrompt(agentInfo, context, {}, formattedTools);

    console.log('[getSystemPromptForChat] 生成Prompt成功，长度:', prompt.length);
    return prompt;
  } catch (error) {
    console.error('[getSystemPromptForChat] 生成Prompt失败:', error);

    // 降级处理：使用基础Prompt
    const mode = agentInfo?.mode || 'agent';
    const basePrompt = promptService.getBasePrompt(mode as any);
    return basePrompt || '你是一个AI助手，请帮助用户解决问题。';
  }
};

class AgentService {
  /**
   * 发送消息给智能体
   */
  async sendMessage(
    agentInfo: AgentInfo,
    message: string,
    options: SendMessageOptions = {}
  ): Promise<ApiResponse<any>> {
    try {
      const systemPrompt = await getSystemPromptForChat(agentInfo, {
        ...options,
        message,
        timestamp: Date.now(),
      });

      const payload = {
        message,
        agent_name: agentInfo.name || agentInfo.display_name,
        role_description: agentInfo.role_description,
        system_prompt: systemPrompt,
        mode: options.mode || 'agent',
        chain: options.chain,
        context_events: options.contextEvents,
        event_summary: options.eventSummary,
        use_async: options.useAsync,
        timeout: options.timeout,
        agent_name_override: options.agentName,
        role_description_override: options.roleDescription,
        custom_instructions: options.customInstructions,
        custom_prompt: options.customPrompt,
        stream: options.stream || false,
      };

      console.log('[AgentService] 发送消息:', { 
        agent: agentInfo.name, 
        messageLength: message.length,
        mode: payload.mode,
        stream: payload.stream,
      });

      const response = await apiClient.post('/agent/chat', payload);
      
      console.log('[AgentService] 消息发送成功:', response.success);
      return response;
    } catch (error) {
      console.error('[AgentService] 发送消息失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 创建智能体
   */
  async createAgent(agentData: Partial<AgentInfo>): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 创建智能体:', agentData.name);

      const payload = {
        name: agentData.name,
        display_name: agentData.display_name || agentData.name,
        description: agentData.description,
        role_description: agentData.role_description,
        avatar: agentData.avatar,
        avatar_url: agentData.avatar_url,
        mode: agentData.mode || 'agent',
        agent_type: agentData.agent_type || 'custom',
        mcp_ports: agentData.mcp_ports,
        mcp_config: agentData.mcp_config,
      };

      const response = await apiClient.post('/agent/create', payload);
      
      if (response.success && response.data) {
        console.log('[AgentService] 智能体创建成功:', response.data.id);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 创建智能体失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 更新智能体信息
   */
  async updateAgent(agentId: string, agentData: Partial<AgentInfo>): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 更新智能体:', agentId);

      const payload = {
        name: agentData.name,
        display_name: agentData.display_name,
        description: agentData.description,
        role_description: agentData.role_description,
        avatar: agentData.avatar,
        avatar_url: agentData.avatar_url,
        status: agentData.status,
        mcp_ports: agentData.mcp_ports,
        mcp_config: agentData.mcp_config,
      };

      const response = await apiClient.put(`/agent/${agentId}`, payload);
      
      if (response.success && response.data) {
        console.log('[AgentService] 智能体更新成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 更新智能体失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 删除智能体
   */
  async deleteAgent(agentId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AgentService] 删除智能体:', agentId);

      const response = await apiClient.delete(`/agent/${agentId}`);
      
      if (response.success) {
        console.log('[AgentService] 智能体删除成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 删除智能体失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取智能体列表
   */
  async getAgents(options: {
    page?: number;
    limit?: number;
    mode?: string;
    status?: string;
  } = {}): Promise<ApiResponse<AgentInfo[]>> {
    try {
      const params = new URLSearchParams();
      if (options.page) params.append('page', String(options.page));
      if (options.limit) params.append('limit', String(options.limit));
      if (options.mode) params.append('mode', options.mode);
      if (options.status) params.append('status', options.status);

      const response = await apiClient.get(`/agent/list?${params.toString()}`);
      
      console.log('[AgentService] 获取智能体列表成功:', response.data?.length);
      return response;
    } catch (error) {
      console.error('[AgentService] 获取智能体列表失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 根据ID获取智能体信息
   */
  async getAgentById(agentId: string): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 获取智能体信息:', agentId);

      const response = await apiClient.get(`/agent/${agentId}`);
      
      if (response.success && response.data) {
        console.log('[AgentService] 智能体信息获取成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 获取智能体信息失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 通过DID解析智能体
   */
  async resolveAgentByDid(did: string): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 通过DID解析智能体:', did);

      // 使用agentResolverService解析
      const resolverResult = await (agentResolverService as any).resolveByDid?.(did);
      
      if (resolverResult.success && resolverResult.data) {
        console.log('[AgentService] DID解析成功:', did);
        return {
          success: true,
          data: resolverResult.data,
          timestamp: new Date().toISOString(),
        };
      }

      // 降级处理：尝试直接解析DID文档
      const didDocument = await this.fetchDidDocument(did);
      if (didDocument) {
        const agent = parseDidDocumentToAgent(didDocument);
        return {
          success: true,
          data: agent,
          timestamp: new Date().toISOString(),
        };
      }

      throw new Error('无法解析DID');
    } catch (error) {
      console.error('[AgentService] DID解析失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 通过IPNS解析智能体
   */
  async resolveAgentByIpns(ipns: string): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 通过IPNS解析智能体:', ipns);

      // 使用agentResolverService解析
      const resolverResult = await (agentResolverService as any).resolveByIpns?.(ipns);
      
      if (resolverResult.success && resolverResult.data) {
        console.log('[AgentService] IPNS解析成功:', ipns);
        return {
          success: true,
          data: resolverResult.data,
          timestamp: new Date().toISOString(),
        };
      }

      throw new Error('无法解析IPNS');
    } catch (error) {
      console.error('[AgentService] IPNS解析失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取智能体的DID文档
   */
  async fetchDidDocument(did: string): Promise<any> {
    try {
      const response = await apiClient.get(`/agent/did/${did}`);
      return response.success ? response.data : null;
    } catch (error) {
      console.error('[AgentService] 获取DID文档失败:', error);
      return null;
    }
  }

  /**
   * 获取智能体状态
   */
  async getAgentStatus(agentId: string): Promise<ApiResponse<any>> {
    try {
      console.log('[AgentService] 获取智能体状态:', agentId);

      const response = await apiClient.get(`/agent/${agentId}/status`);
      
      return response;
    } catch (error) {
      console.error('[AgentService] 获取智能体状态失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 启动智能体
   */
  async startAgent(agentId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AgentService] 启动智能体:', agentId);

      const response = await apiClient.post(`/agent/${agentId}/start`);
      
      if (response.success) {
        console.log('[AgentService] 智能体启动成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 启动智能体失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 停止智能体
   */
  async stopAgent(agentId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AgentService] 停止智能体:', agentId);

      const response = await apiClient.post(`/agent/${agentId}/stop`);
      
      if (response.success) {
        console.log('[AgentService] 智能体停止成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 停止智能体失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 获取智能体消息历史
   */
  async getAgentMessages(
    agentId: string, 
    options: {
      page?: number;
      limit?: number;
      before?: string;
      after?: string;
    } = {}
  ): Promise<ApiResponse<Message[]>> {
    try {
      console.log('[AgentService] 获取智能体消息历史:', agentId);

      const params = new URLSearchParams();
      if (options.page) params.append('page', String(options.page));
      if (options.limit) params.append('limit', String(options.limit));
      if (options.before) params.append('before', options.before);
      if (options.after) params.append('after', options.after);

      const response = await apiClient.get(`/agent/${agentId}/messages?${params.toString()}`);
      
      console.log('[AgentService] 消息历史获取成功:', response.data?.length);
      return response;
    } catch (error) {
      console.error('[AgentService] 获取消息历史失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 清除智能体消息历史
   */
  async clearAgentMessages(agentId: string): Promise<ApiResponse<void>> {
    try {
      console.log('[AgentService] 清除智能体消息历史:', agentId);

      const response = await apiClient.delete(`/agent/${agentId}/messages`);
      
      if (response.success) {
        console.log('[AgentService] 消息历史清除成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 清除消息历史失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 导出智能体配置
   */
  async exportAgent(agentId: string): Promise<ApiResponse<any>> {
    try {
      console.log('[AgentService] 导出智能体配置:', agentId);

      const response = await apiClient.get(`/agent/${agentId}/export`);
      
      if (response.success) {
        console.log('[AgentService] 智能体配置导出成功:', agentId);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 导出智能体配置失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }

  /**
   * 导入智能体配置
   */
  async importAgent(configData: any): Promise<ApiResponse<AgentInfo>> {
    try {
      console.log('[AgentService] 导入智能体配置');

      const response = await apiClient.post('/agent/import', configData);
      
      if (response.success && response.data) {
        console.log('[AgentService] 智能体配置导入成功:', response.data.id);
      }
      
      return response;
    } catch (error) {
      console.error('[AgentService] 导入智能体配置失败:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date().toISOString(),
      };
    }
  }
}

// 创建单例实例
const agentService = new AgentService();

export default agentService;
export { AgentService };
export type { AgentInfo, SendMessageOptions, Message };
