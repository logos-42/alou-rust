/**
 * Worker 入口文件 - 混合 Rust WASM 和 TypeScript SDK
 * 
 * 根据 agent_type 自动选择使用 Rust WASM 或 TypeScript SDK
 */

import RustWorker from '../build/index.js';
import { ClaudeAgent } from './ts-agent/claude-agent';
import type { CustomAgentInfo, ChatMessage } from './ts-agent/types';

// WASM初始化状态
let wasmInitialized = false;
let wasmInitPromise: Promise<void> | null = null;

// 确保WASM正确初始化
async function ensureWasmInitialized(): Promise<void> {
  if (wasmInitialized) return;
  
  if (!wasmInitPromise) {
    wasmInitPromise = (async () => {
      try {
        console.log('[Worker] 初始化WASM模块...');
        
        // #region agent log - 测试假设C：Worker环境切换问题
        try {
          fetch('http://127.0.0.1:7242/ingest/730fa833-2da6-4d3a-bcad-37f577a26c2f', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              location: 'worker.ts:ensureWasmInitialized',
              message: '开始WASM初始化',
              data: { wasmInitialized, hasWasmInitPromise: !!wasmInitPromise },
              timestamp: Date.now(),
              sessionId: 'debug-session',
              runId: 'run1',
              hypothesisId: 'C'
            })
          }).catch(() => {});
        } catch (e) {}
        // #endregion
        
        // 检查是否有初始化函数
        if (typeof (RustWorker as any).ensureWasmInitialized === 'function') {
          await (RustWorker as any).ensureWasmInitialized();
        }
        
        // 尝试创建实例来验证初始化
        const testInstance = new (RustWorker as any)({}, {});
        if (testInstance && typeof testInstance.fetch === 'function') {
          console.log('[Worker] WASM初始化成功');
          wasmInitialized = true;
          
          // #region agent log - 测试假设C：Worker环境切换问题
          try {
            fetch('http://127.0.0.1:7242/ingest/730fa833-2da6-4d3a-bcad-37f577a26c2f', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                location: 'worker.ts:ensureWasmInitialized',
                message: 'WASM初始化成功',
                data: { wasmInitialized: true },
                timestamp: Date.now(),
                sessionId: 'debug-session',
                runId: 'run1',
                hypothesisId: 'C'
              })
            }).catch(() => {});
          } catch (e) {}
          // #endregion
        } else {
          throw new Error('WASM实例创建失败');
        }
      } catch (error) {
        console.error('[Worker] WASM初始化失败:', error);
        
        // #region agent log - 测试假设C：Worker环境切换问题
        try {
          fetch('http://127.0.0.1:7242/ingest/730fa833-2da6-4d3a-bcad-37f577a26c2f', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              location: 'worker.ts:ensureWasmInitialized',
              message: 'WASM初始化失败',
              data: { error: error instanceof Error ? error.message : String(error) },
              timestamp: Date.now(),
              sessionId: 'debug-session',
              runId: 'run1',
              hypothesisId: 'C'
            })
          }).catch(() => {});
        } catch (e) {}
        // #endregion
        
        throw error;
      }
    })();
  }
  
  return wasmInitPromise;
}

// 创建 Rust Worker 实例（用于调用 Rust WASM）
// WorkerEntrypoint 类需要 (ctx, env) 作为构造函数参数
const createRustInstance = (env: any, ctx: any) => {
  // Rust WASM 导出的是 WorkerEntrypoint 类
  // 构造函数需要 (ctx, env) 参数
  const instance = new (RustWorker as any)(ctx, env) as { fetch: (request: Request) => Promise<Response> };
  return instance;
};

// 调用 Rust WASM 的 fetch 方法（带初始化检查）
const rustFetch = async (request: Request, env: any, ctx: any): Promise<Response> => {
  try {
    // 确保WASM已初始化
    await ensureWasmInitialized();
    
    const instance = createRustInstance(env, ctx);
    return instance.fetch(request);
  } catch (error) {
    console.error('[Worker] Rust WASM调用失败:', error);
    return new Response(JSON.stringify({
      error: 'WASM初始化失败',
      message: error instanceof Error ? error.message : String(error)
    }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
};

// 添加 CORS 头到响应
const addCorsHeaders = (response: Response, headers: Record<string, string> = {}): Response => {
  const corsHeaders = new Headers(response.headers);
  corsHeaders.set('Access-Control-Allow-Origin', '*');
  corsHeaders.set('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  corsHeaders.set('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  corsHeaders.set('Access-Control-Max-Age', '86400');
  
  // 合并额外的 headers
  Object.entries(headers).forEach(([key, value]) => {
    corsHeaders.set(key, value);
  });
  
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: corsHeaders,
  });
};

// 导出Durable Objects
export { AITaskDO } from '../build/index.js';

// Worker 入口
export default {
  async fetch(request: Request, env: any, ctx: any): Promise<Response> {
    const url = new URL(request.url);
    
    // 处理 OPTIONS 预检请求
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type, Authorization',
          'Access-Control-Max-Age': '86400',
        },
      });
    }
    
    // 如果是 /api/agent/chat，检查是否需要使用 TypeScript SDK
    if (url.pathname === '/api/agent/chat' && request.method === 'POST') {
      try {
        // 克隆请求以读取 body
        const body = await request.clone().json() as {
          session_id: string;
          message: string;
          wallet_address?: string;
          chain?: string;
          context_events?: any[];
          event_summary?: string;
        };
        
        // 从 KV 获取 session（包含 agent metadata）
        // Rust 后端使用格式: `session:${session_id}`
        const sessionKey = `session:${body.session_id}`;
        const sessionStr = await env.SESSIONS.get(sessionKey);
        
        if (sessionStr) {
          const session = JSON.parse(sessionStr);
          const metadata = session.agent_metadata;
          
          if (metadata) {
            const agentType = metadata.agent_type;
            
            // 如果是 claude_agent_sdk，使用 TypeScript SDK（Agent SDK 框架 + DeepSeek 模型后端）
            // 参考: https://platform.claude.com/docs/en/agent-sdk/typescript
            if (agentType === 'claude_agent_sdk') {
              console.log('[Worker] Agent 模式：使用 TypeScript SDK (DeepSeek API 作为模型后端)');
              
              // 获取 DeepSeek API Key（与 Rust WASM 后端使用相同的 API Key）
              const apiKey = env.AI_API_KEY;
              if (!apiKey) {
                console.error('[Worker] DeepSeek API Key 未配置，请设置 AI_API_KEY');
                return new Response(JSON.stringify({ 
                  error: 'AI_API_KEY (DeepSeek API Key) not configured for Agent mode' 
                }), {
                  status: 500,
                  headers: { 'Content-Type': 'application/json' }
                });
              }
              
              // 获取会话历史（已从上面获取）
              let history: ChatMessage[] = [];
              if (session && session.messages) {
                history = session.messages.map((msg: any) => ({
                  role: msg.role === 'assistant' ? 'assistant' : 'user',
                  content: msg.content,
                  timestamp: msg.timestamp,
                }));
              }
              
              // 构建 agentInfo
              const agentInfo: CustomAgentInfo = {
                name: metadata.display_name || metadata.name || '智能体',
                role_description: metadata.role_description || '',
                custom_instructions: metadata.customPrompt || metadata.custom_prompt,
                did: metadata.did,
                ipns: metadata.ipns,
                cid: metadata.cid,
                avatar_cid: metadata.avatar_cid,
                mcp_config_cid: metadata.mcp_config_cid,
              };
              
              // 创建工具执行器（通过调用 Rust WASM 后端执行工具）
              const toolExecutor = async (toolCall: any) => {
                console.log(`[Worker] 执行工具: ${toolCall.name}`, toolCall.input);
                
                // 通过调用 Rust WASM 后端的工具执行接口来执行工具
                try {
                  // 构建内部请求 URL（使用原始请求的 URL 作为基础）
                  const baseUrl = new URL(request.url);
                  const toolExecuteUrl = new URL('/api/mcp/execute-tool', baseUrl.origin);
                  
                  const toolExecuteRequest = new Request(toolExecuteUrl.toString(), {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                      tool_name: toolCall.name,
                      args: toolCall.input,
                      wallet_address: body.wallet_address,
                      chain: body.chain,
                    }),
                  });
                  
                  // 调用 Rust WASM 后端执行工具
                  const toolResponse = await rustFetch(toolExecuteRequest, env, ctx);
                  const toolResult = await toolResponse.json() as { result: any; error?: string };
                  
                  if (toolResult.error) {
                    throw new Error(toolResult.error);
                  }
                  
                  return { result: toolResult.result };
                } catch (error: any) {
                  console.error(`[Worker] 工具执行失败: ${toolCall.name}`, error);
                  return {
                    result: { error: error.message || 'Tool execution failed' },
                    error: error.message,
                  };
                }
              };
              
              // 获取可用工具列表（从 Rust WASM 后端）
              let tools: any[] = [];
              try {
                // 构建内部请求 URL
                const baseUrl = new URL(request.url);
                const toolsUrl = new URL('/api/mcp/tools', baseUrl.origin);
                
                const toolsRequest = new Request(toolsUrl.toString(), {
                  method: 'GET',
                });
                
                const toolsResponse = await rustFetch(toolsRequest, env, ctx);
                const toolsData = await toolsResponse.json() as { tools: Array<{ name: string; description: string; input_schema: any }> };
                
                // 转换为 DeepSeek/OpenAI 格式
                tools = toolsData.tools.map(tool => ({
                  type: 'function',
                  function: {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                  },
                }));
                
                console.log(`[Worker] 获取到 ${tools.length} 个可用工具`);
              } catch (error: any) {
                console.warn(`[Worker] 获取工具列表失败:`, error);
                // 继续使用空数组，不影响对话
              }
              
              // 使用 TypeScript SDK 处理（支持工具调用）
              const agent = new ClaudeAgent(apiKey);
              const result = await agent.chat({
                message: body.message,
                history,
                agentInfo,
                walletAddress: body.wallet_address,
                chain: body.chain,
                tools: tools,
                toolExecutor: toolExecutor,
              });
              
              // 保存消息到会话（与 Rust 后端格式一致）
              if (session) {
                session.messages = session.messages || [];
                session.messages.push({
                  role: 'user',
                  content: body.message,
                  timestamp: Math.floor(Date.now() / 1000), // Rust 使用秒级时间戳
                });
                session.messages.push({
                  role: 'assistant',
                  content: result.response,
                  timestamp: Math.floor(Date.now() / 1000),
                  tool_call_id: null,
                });
                session.updated_at = Math.floor(Date.now() / 1000);
                
                await env.SESSIONS.put(sessionKey, JSON.stringify(session), {
                  expirationTtl: 86400 * 7, // 7 days
                });
              }
              
              // 返回响应（格式与 Rust 后端一致）
              const response = new Response(JSON.stringify({
                content: result.response,
                session_id: body.session_id,
                tool_calls: result.tool_calls || [],
              }), {
                headers: { 'Content-Type': 'application/json' }
              });
              return addCorsHeaders(response, {});
            }
          }
        }
        
        // 如果不是 claude_agent_sdk，使用 Rust WASM 处理（默认使用 DeepSeek API）
        console.log('[Worker] Alou 模式：使用 Rust WASM 后端（DeepSeek API）');
        return await rustFetch(request, env, ctx);
        
      } catch (error: any) {
        // 如果处理过程中出错，记录详细错误信息并传递给 Rust 后端处理
        console.error('[Worker] TypeScript 层处理请求时出错:', error);
        console.error('[Worker] 错误堆栈:', error.stack);
        // 继续传递给 Rust 后端，让 Rust 后端处理错误
        // 这样可以确保错误信息正确传递，而不是在这里吞掉
        return await rustFetch(request, env, ctx);
      }
    }
    
    // 其他请求直接使用 Rust WASM 处理
    return await rustFetch(request, env, ctx);
  }
};

