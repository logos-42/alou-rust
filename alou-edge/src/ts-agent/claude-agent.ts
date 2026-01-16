/**
 * TypeScript Agent - 使用 Claude Agent SDK 风格的接口
 * 参考: https://platform.claude.com/docs/en/agent-sdk/typescript
 * 
 * 注意：在 Cloudflare Workers 环境中，Claude Agent SDK 无法直接使用（需要进程执行）
 * 因此我们使用方案 B: 自定义适配器，实现 Agent SDK 风格的接口
 * 底层使用 DeepSeek API（OpenAI 兼容格式）作为模型后端
 * 
 * 这提供了与 Claude Agent SDK 兼容的接口风格，但使用 DeepSeek 作为后端
 */

// 注意：Claude Agent SDK 在 Workers 环境中不可用（需要进程执行）
// import { query } from '@anthropic-ai/claude-agent-sdk';
import type { CustomAgentInfo, ChatMessage, ToolCall } from './types';
import { 
  getSystemPrompt, 
  getSystemPromptForCustomAgent, 
  addContextToPrompt,
  detectPromptMode,
} from './prompts';

const DEEPSEEK_API_URL = 'https://api.deepseek.com/v1/chat/completions';

/**
 * 工具定义（DeepSeek/OpenAI 格式）
 */
export interface ToolDefinition {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

/**
 * Agent SDK 风格的查询接口
 */
export interface AgentQueryOptions {
  prompt: string;
  systemPrompt?: string;
  history?: ChatMessage[];
  agentInfo?: CustomAgentInfo;
  walletAddress?: string;
  chain?: string;
  model?: string;
  maxTokens?: number;
  temperature?: number;
  tools?: ToolDefinition[];
  toolExecutor?: (toolCall: ToolCall) => Promise<{ result: any; error?: string }>;
}

/**
 * Agent SDK 风格的响应接口
 */
export interface AgentQueryResult {
  response: string;
  tool_calls?: ToolCall[];
  usage?: {
    input_tokens: number;
    output_tokens: number;
  };
  task_id?: string; // 异步任务ID
}

/**
 * Claude Agent SDK 风格的客户端
 * 
 * 实现说明：
 * - 使用方案 B: 自定义适配器（DeepSeek API 直接调用）
 * - 提供与 Claude Agent SDK 兼容的接口风格（query() 方法）
 * - 底层使用 DeepSeek API（OpenAI 兼容格式）作为模型后端
 * 
 * 为什么不用真正的 Claude Agent SDK？
 * 1. Claude Agent SDK 需要进程执行（spawn process），Cloudflare Workers 不支持
 * 2. Claude Agent SDK 主要用于代码执行和工具调用，我们只需要简单聊天
 * 3. DeepSeek API 兼容 OpenAI 格式，可以直接调用，更简单可靠
 */
export class ClaudeAgent {
  private apiKey: string;
  private defaultModel: string = 'deepseek-chat';
  // 移除工具调用次数限制（无限调用）
  // private maxToolIterations: number = 10;
  
  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  /**
   * 创建异步任务（使用Rust WASM后端）
   */
  private async createAsyncTask(compatRequest: any, env: any, ctx: any, aiTasksNamespace?: any): Promise<string> {
    console.log('[Claude SDK] Creating async task via Rust WASM backend');

    try {
      // 构建内部请求URL
      const baseUrl = `http://dummy`; // Durable Object使用相对URL

      // 创建任务初始化请求
      const requestBody = JSON.stringify(compatRequest);
      const headers = new Headers();
      headers.set('Content-Type', 'application/json');

      // 注意：这里我们需要使用Rust WASM的Durable Object
      // 由于在TypeScript环境中，我们通过fetch调用Rust WASM后端
      // 但是由于路由问题，我们直接创建Durable Object实例

      // 获取Durable Object命名空间（优先使用传入的命名空间）
      const namespace = aiTasksNamespace || env.AI_TASKS;
      if (!namespace) {
        throw new Error('AI_TASKS Durable Object namespace not found. Make sure Durable Objects are properly configured.');
      }

      // 生成任务ID
      const taskId = `task_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      console.log(`[Claude SDK] Generated task ID: ${taskId}`);

      // 获取Durable Object存根
      const id = namespace.idFromName(taskId);
      const stub = id.getStub();

      // 调用init-and-start端点
      const response = await stub.fetch('http://dummy/init-and-start', {
        method: 'POST',
        headers: headers,
        body: requestBody,
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Failed to create async task: ${response.status} ${errorText}`);
      }

      const result = await response.json();
      console.log(`[Claude SDK] Async task created successfully: ${taskId}`);

      return taskId;
    } catch (error: any) {
      console.error('[Claude SDK] Failed to create async task:', error);
      throw new Error(`Async task creation failed: ${error.message}`);
    }
  }
  
  /**
   * 方案 B: 自定义适配器（DeepSeek API 直接调用）
   *
   * 在 Cloudflare Workers 环境中，这是唯一可行的方案：
   * 1. Claude Agent SDK 需要进程执行，Workers 不支持
   * 2. 我们只需要简单的聊天功能，不需要代码执行
   * 3. DeepSeek API 兼容 OpenAI 格式，可以直接调用
   *
   * 这个实现提供了与 Claude Agent SDK 兼容的接口风格，并支持工具调用
   */
  async query(options: AgentQueryOptions, env?: any, ctx?: any, aiTasksNamespace?: any): Promise<AgentQueryResult> {
    const {
      prompt,
      systemPrompt: customSystemPrompt,
      history = [],
      agentInfo,
      walletAddress,
      chain,
      model = this.defaultModel,
      maxTokens = 8192, // DeepSeek API 限制最大 8192 tokens
      temperature = 0.7,
      tools = [],
      toolExecutor,
    } = options;

    // 🚀 检查是否应该使用异步处理（强制异步）
    console.log('🚀🚀🚀 CLAUDE SDK ASYNC CHECK START 🚀🚀🚀');
    console.log('[Claude SDK] Checking if should use async processing...');
    console.log(`[Claude SDK] env exists: ${!!env}, ctx exists: ${!!ctx}`);
    console.log(`[Claude SDK] env.AI_TASKS exists: ${!!(env && env.AI_TASKS)}`);
    console.log(`[Claude SDK] aiTasksNamespace exists: ${!!aiTasksNamespace}`);
    console.log(`[Claude SDK] FORCED ASYNC MODE ENABLED`);
    const shouldUseAsync = true; // 强制使用异步处理

    if (shouldUseAsync && (env || aiTasksNamespace)) {
      console.log('✅✅✅ USING ASYNC PROCESSING - CREATING BACKGROUND TASK ✅✅✅');
      console.log('[Claude SDK] Using async processing - creating background task');

      // 构建兼容性请求
      const compatRequest = {
        prompt,
        system_prompt: customSystemPrompt,
        history: history.map(h => ({
          role: h.role,
          content: h.content,
          timestamp: h.timestamp,
        })),
        agent_info: agentInfo,
        tools: tools.map(t => ({
          name: t.function.name,
          description: t.function.description,
          parameters: t.function.parameters,
        })),
        model,
        max_tokens: maxTokens,
        temperature,
        task_type: 'async', // 强制异步
      };

      // 创建异步任务（使用Rust WASM后端）
      const taskId = await this.createAsyncTask(compatRequest, env, ctx, aiTasksNamespace);

      return {
        response: `异步任务已创建，任务ID: ${taskId}`,
        tool_calls: undefined,
        usage: undefined,
        task_id: taskId, // 返回任务ID
      };
    }

    console.log('❌❌❌ FALLBACK TO SYNC PROCESSING ❌❌❌');
    console.log('[Claude SDK] Using sync processing');

    // 确定系统 Prompt
    let systemPrompt: string;
    if (customSystemPrompt) {
      systemPrompt = customSystemPrompt;
    } else if (agentInfo) {
      systemPrompt = getSystemPromptForCustomAgent(agentInfo);
    } else {
      const mode = detectPromptMode(prompt);
      systemPrompt = getSystemPrompt(mode);
    }

    systemPrompt = addContextToPrompt(systemPrompt, walletAddress, chain);
    
    // 构建消息历史（DeepSeek 使用 OpenAI 兼容格式）
    let messages: Array<{
      role: string;
      content: string;
      tool_call_id?: string;
      tool_calls?: Array<{
        id: string;
        type: string;
        function: {
          name: string;
          arguments: string;
        };
      }>;
    }> = [];
    
    if (systemPrompt) {
      messages.push({
        role: 'system',
        content: systemPrompt,
      });
    }
    
    // 转换历史消息（包括工具调用）
    for (const msg of history) {
      const message: any = {
        role: msg.role,
        content: msg.content,
      };
      
      if (msg.role === 'tool') {
        // 工具结果消息
        messages.push({
          role: 'tool',
          content: msg.content,
          tool_call_id: msg.tool_calls?.[0]?.id || '',
        });
      } else {
        // 用户或助手消息
        if (msg.tool_calls && msg.tool_calls.length > 0) {
          // 助手消息包含工具调用
          message.tool_calls = msg.tool_calls.map(tc => ({
            id: tc.id,
            type: 'function',
            function: {
              name: tc.name,
              arguments: JSON.stringify(tc.input),
            },
          }));
        }
        messages.push(message);
      }
    }
    
    // 工具调用循环（无限调用，直到没有工具调用或达到最大上下文）
    let iterations = 0;
    let finalContent = '';
    let toolCallResults: ToolCall[] = [];
    let totalUsage: { input_tokens: number; output_tokens: number } | undefined;
    const MAX_ITERATIONS_SAFETY = 1000; // 安全限制，防止无限循环（实际不会达到）
    
    while (iterations < MAX_ITERATIONS_SAFETY) {
      iterations++;
      
      // 添加用户消息（仅在第一次迭代）
      if (iterations === 1) {
        messages.push({
          role: 'user',
          content: prompt,
        });
      }
      
      // 准备请求体
      const requestBody: any = {
        model: model,
        messages: messages,
        max_tokens: maxTokens,
        temperature: temperature,
      };
      
      // 如果有工具定义，添加到请求中
      if (tools.length > 0) {
        requestBody.tools = tools;
        requestBody.tool_choice = 'auto'; // 让模型决定是否使用工具
      }
      
      // 调用 DeepSeek API
      const response = await fetch(DEEPSEEK_API_URL, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${this.apiKey}`,
        },
        body: JSON.stringify(requestBody),
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`DeepSeek API error: ${response.status} ${errorText}`);
      }
      
      const data = await response.json() as {
        choices: Array<{
          message: {
            role: string;
            content: string | null;
            tool_calls?: Array<{
              id: string;
              type: string;
              function: {
                name: string;
                arguments: string;
              };
            }>;
          };
          finish_reason: string;
        }>;
        usage?: {
          prompt_tokens: number;
          completion_tokens: number;
          total_tokens: number;
        };
      };
      
      const choice = data.choices[0];
      if (!choice) {
        throw new Error('No choices in response');
      }
      
      // 累计使用统计
      if (data.usage) {
        if (totalUsage) {
          totalUsage.input_tokens += data.usage.prompt_tokens;
          totalUsage.output_tokens += data.usage.completion_tokens;
        } else {
          totalUsage = {
            input_tokens: data.usage.prompt_tokens,
            output_tokens: data.usage.completion_tokens,
          };
        }
      }
      
      const assistantMessage = choice.message;
      const toolCalls = assistantMessage.tool_calls || [];
      
      // 如果有工具调用，执行工具
      if (toolCalls.length > 0 && toolExecutor) {
        // 添加助手消息（包含工具调用）
        messages.push({
          role: 'assistant',
          content: assistantMessage.content || '',
          tool_calls: toolCalls,
        });
        
        // 执行所有工具调用
        for (const toolCall of toolCalls) {
          try {
            const toolCallObj: ToolCall = {
              id: toolCall.id,
              name: toolCall.function.name,
              input: JSON.parse(toolCall.function.arguments),
            };
            
            const result = await toolExecutor(toolCallObj);
            
            // 添加工具结果到消息历史
            messages.push({
              role: 'tool',
              content: JSON.stringify(result.result),
              tool_call_id: toolCall.id,
            });
            
            // 记录工具调用结果
            toolCallResults.push({
              ...toolCallObj,
              output: JSON.stringify(result.result),
            });
          } catch (error: any) {
            // 工具执行失败，添加错误结果
            messages.push({
              role: 'tool',
              content: JSON.stringify({ error: error.message }),
              tool_call_id: toolCall.id,
            });
            
            toolCallResults.push({
              id: toolCall.id,
              name: toolCall.function.name,
              input: JSON.parse(toolCall.function.arguments),
              output: JSON.stringify({ error: error.message }),
            });
          }
        }
        
        // 继续循环，等待模型的下一步响应
        continue;
      } else {
        // 没有工具调用，返回最终响应
        finalContent = assistantMessage.content || '';
        break;
      }
    }
    
    if (iterations >= MAX_ITERATIONS_SAFETY) {
      throw new Error('Safety limit: Maximum tool iterations exceeded (this should never happen)');
    }
    
    return {
      response: finalContent,
      tool_calls: toolCallResults.length > 0 ? toolCallResults : undefined,
      usage: totalUsage,
    };
  }
  
  /**
   * 兼容旧接口的 chat() 方法
   */
  async chat(options: {
    message: string;
    history?: ChatMessage[];
    agentInfo?: CustomAgentInfo;
    walletAddress?: string;
    chain?: string;
    tools?: ToolDefinition[];
    toolExecutor?: (toolCall: ToolCall) => Promise<{ result: any; error?: string }>;
  }, env?: any, ctx?: any, aiTasksNamespace?: any): Promise<{
    response: string;
    tool_calls?: ToolCall[];
    task_id?: string;
  }> {
    const result = await this.query({
      prompt: options.message,
      history: options.history,
      agentInfo: options.agentInfo,
      walletAddress: options.walletAddress,
      chain: options.chain,
      tools: options.tools,
      toolExecutor: options.toolExecutor,
    }, env, ctx, aiTasksNamespace);

    return {
      response: result.response,
      tool_calls: result.tool_calls,
      task_id: result.task_id,
    };
  }
}

/**
 * 创建 Agent SDK 风格的客户端实例
 */
export function createClaudeAgent(apiKey: string): ClaudeAgent {
  if (!apiKey) {
    throw new Error('AI_API_KEY (DeepSeek API Key) is required');
  }
  return new ClaudeAgent(apiKey);
}