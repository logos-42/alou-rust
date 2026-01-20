/**
 * Worker 入口文件 - 混合 Rust WASM 和 TypeScript SDK
 * 
 * 根据 agent_type 自动选择使用 Rust WASM 或 TypeScript SDK
 */

import RustWorker from '../build/index.js';

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
// 注意：build/index.js 中 AITaskDO 被导出为 It 的别名
// 我们需要确保正确导入
import { AITaskDO as RustAITaskDO } from '../build/index.js';
export { RustAITaskDO as AITaskDO };

// 工具执行相关函数
function isComplexTool(toolId: string): boolean {
  // 定义需要异步执行的复杂工具
  const complexTools = [
    'search', // 搜索可能需要长时间处理
    'filesystem_copy', // 大文件复制
    'network_scan', // 网络扫描
    'code_analysis', // 代码分析
  ];

  return complexTools.includes(toolId) || toolId.includes('complex') || toolId.includes('heavy');
}

async function createToolExecutionTask(
  toolId: string,
  args: any,
  env: any,
  sessionId?: string
): Promise<string> {
  // 生成任务ID
  const taskId = `tool_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

  // 创建异步任务数据
  const taskData = {
    id: taskId,
    type: 'tool_execution',
    tool_id: toolId,
    args,
    session_id: sessionId,
    status: 'queued',
    created_at: Date.now(),
    updated_at: Date.now(),
  };

  // 存储到 KV 或 Durable Object
  const taskKey = `tool_task:${taskId}`;
  await env.SESSIONS.put(taskKey, JSON.stringify(taskData), {
    expirationTtl: 86400, // 24小时过期
  });

  // 这里可以触发实际的工具执行逻辑
  // 为了简化，我们直接模拟异步执行
  // 在实际实现中，应该调用相应的工具执行逻辑

  // 模拟异步执行（实际应该调用真正的工具）
  setTimeout(async () => {
    try {
      const result = await executeToolDirectly(toolId, args, env, sessionId);

      // 更新任务状态
      const completedTask = {
        ...taskData,
        status: 'completed',
        result,
        updated_at: Date.now(),
        execution_time_ms: 1000, // 模拟执行时间
      };

      await env.SESSIONS.put(taskKey, JSON.stringify(completedTask), {
        expirationTtl: 86400,
      });

    } catch (error: any) {
      // 更新任务状态为失败
      const failedTask = {
        ...taskData,
        status: 'failed',
        error: error.message,
        updated_at: Date.now(),
      };

      await env.SESSIONS.put(taskKey, JSON.stringify(failedTask), {
        expirationTtl: 86400,
      });
    }
  }, 100); // 短暂延迟模拟异步处理

  return taskId;
}

async function executeToolDirectly(
  toolId: string,
  args: any,
  env: any,
  sessionId?: string
): Promise<any> {
  console.log(`[Worker] 直接执行工具: ${toolId}`, args);

  // 这里实现实际的工具执行逻辑
  // 目前提供简单的模拟实现

  switch (toolId) {
    case 'filesystem':
      return await executeFilesystemTool(args, env);

    case 'search':
      return await executeSearchTool(args, env);

    case 'bash':
      return await executeBashTool(args, env);

    case 'plan':
      return await executePlanTool(args, env);

    case 'skills':
      return await executeSkillsTool(args, env);

    default:
      // 对于未知工具，尝试通过 Rust WASM 后端执行
      return await executeViaRustBackend(toolId, args, env, sessionId);
  }
}

async function executeFilesystemTool(args: any, env: any): Promise<any> {
  const { operation, path } = args;

  // 简单的文件系统工具模拟
  // 实际应该调用真正的文件系统操作
  switch (operation) {
    case 'read':
      return {
        success: true,
        data: { content: `Mock content from ${path}` },
        output: `Successfully read file: ${path}`,
      };

    case 'write':
      return {
        success: true,
        data: { written: true },
        output: `Successfully wrote to file: ${path}`,
      };

    case 'list':
      return {
        success: true,
        data: {
          entries: [
            { name: 'file1.txt', type: 'file' },
            { name: 'dir1', type: 'directory' },
          ]
        },
        output: `Listed contents of: ${path}`,
      };

    default:
      return {
        success: false,
        error: `Unknown filesystem operation: ${operation}`,
      };
  }
}

async function executeSearchTool(args: any, env: any): Promise<any> {
  const { type, pattern, path } = args;

  // 简单的搜索工具模拟
  return {
    success: true,
    data: {
      pattern,
      matches: [
        { file: `${path}/example.txt`, line: 1, content: `Found ${pattern}` }
      ],
      total_matches: 1,
    },
    output: `Search completed for pattern: ${pattern}`,
  };
}

async function executeBashTool(args: any, env: any): Promise<any> {
  const { command } = args;

  // 简单的 Bash 工具模拟
  // 注意：在 Cloudflare Workers 中不能实际执行 shell 命令
  // 这里应该返回适当的错误或通过其他方式处理
  return {
    success: false,
    error: 'Shell command execution not supported in Cloudflare Workers environment',
    data: { command },
  };
}

async function executePlanTool(args: any, env: any): Promise<any> {
  const { action } = args;

  // 简单的计划工具模拟
  switch (action) {
    case 'create_plan':
      return {
        success: true,
        data: {
          plan: {
            id: `plan_${Date.now()}`,
            name: args.name,
            steps: args.steps || [],
          }
        },
        output: `Created plan: ${args.name}`,
      };

    default:
      return {
        success: false,
        error: `Unknown plan action: ${action}`,
      };
  }
}

async function executeSkillsTool(args: any, env: any): Promise<any> {
  const { action } = args;

  // 简单的技能工具模拟
  switch (action) {
    case 'list_skills':
      return {
        success: true,
        data: {
          skills: [
            { id: 'text_summarizer', name: 'Text Summarizer' },
            { id: 'code_formatter', name: 'Code Formatter' },
          ]
        },
        output: 'Retrieved skills list',
      };

    default:
      return {
        success: false,
        error: `Unknown skills action: ${action}`,
      };
  }
}

async function executeViaRustBackend(
  toolId: string,
  args: any,
  env: any,
  sessionId?: string
): Promise<any> {
  // 尝试通过 Rust WASM 后端执行未知工具
  console.log(`[Worker] 尝试通过 Rust 后端执行工具: ${toolId}`);

  try {
    // 这里可以构建请求调用 Rust WASM 后端的工具执行接口
    // 暂时返回模拟结果
    return {
      success: true,
      data: { message: `Tool ${toolId} executed via Rust backend` },
      output: `Executed tool via Rust backend: ${toolId}`,
    };
  } catch (error: any) {
    return {
      success: false,
      error: `Failed to execute via Rust backend: ${error.message}`,
    };
  }
}

async function getToolTaskStatus(taskId: string, env: any): Promise<any> {
  const taskKey = `tool_task:${taskId}`;
  const taskDataStr = await env.SESSIONS.get(taskKey);

  if (!taskDataStr) {
    return { error: 'Task not found', status: 'not_found' };
  }

  const taskData = JSON.parse(taskDataStr);
  return {
    task_id: taskId,
    status: taskData.status,
    tool_id: taskData.tool_id,
    result: taskData.result,
    error: taskData.error,
    created_at: taskData.created_at,
    updated_at: taskData.updated_at,
    execution_time_ms: taskData.execution_time_ms,
  };
}

async function getToolList(env: any): Promise<any[]> {
  // 返回支持的工具列表
  return [
    {
      id: 'filesystem',
      name: 'File System',
      category: 'FileSystem',
      description: '文件系统操作工具',
      executionMode: 'remote',
    },
    {
      id: 'search',
      name: 'Search',
      category: 'Search',
      description: '搜索工具',
      executionMode: 'remote',
    },
    {
      id: 'bash',
      name: 'Bash Shell',
      category: 'Terminal',
      description: '终端命令执行',
      executionMode: 'remote',
    },
    {
      id: 'plan',
      name: 'Task Planning',
      category: 'Planning',
      description: '任务规划工具',
      executionMode: 'remote',
    },
    {
      id: 'skills',
      name: 'Skills',
      category: 'Skills',
      description: '技能管理系统',
      executionMode: 'remote',
    },
  ];
}

async function getToolExecutionHistory(limit: number, env: any): Promise<any[]> {
  // 这里应该从存储中获取执行历史
  // 暂时返回空数组
  return [];
}

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
        
        // 所有请求都直接使用 Rust WASM 后端处理
        console.log('[Worker] 所有请求都使用 Rust WASM 后端处理');

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
    
    // 工具执行路由
    if (url.pathname === '/api/tools/execute' && request.method === 'POST') {
      try {
        const body = await request.json() as {
          tool_id: string;
          args: any;
          session_id?: string;
          async?: boolean;
        };

        console.log(`[Worker] 执行工具: ${body.tool_id}`, { async: body.async });

        // 检查是否应该异步执行
        if (body.async || isComplexTool(body.tool_id)) {
          // 创建异步任务
          const taskId = await createToolExecutionTask(body.tool_id, body.args, env, body.session_id);
          const response = new Response(JSON.stringify({
            task_id: taskId,
            status: 'queued',
            message: 'Tool execution started asynchronously'
          }), {
            headers: { 'Content-Type': 'application/json' }
          });
          return addCorsHeaders(response);
        }

        // 同步执行
        const result = await executeToolDirectly(body.tool_id, body.args, env, body.session_id);
        const response = new Response(JSON.stringify(result), {
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);

      } catch (error: any) {
        console.error('[Worker] 工具执行失败:', error);
        const response = new Response(JSON.stringify({
          success: false,
          error: error.message || 'Tool execution failed'
        }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);
      }
    }

    // 工具任务状态查询
    if (url.pathname.startsWith('/api/tools/status/') && request.method === 'GET') {
      try {
        const taskId = url.pathname.split('/').pop();
        if (!taskId) {
          const response = new Response(JSON.stringify({ error: 'Invalid task ID' }), {
            status: 400,
            headers: { 'Content-Type': 'application/json' }
          });
          return addCorsHeaders(response);
        }

        const status = await getToolTaskStatus(taskId, env);
        const response = new Response(JSON.stringify(status), {
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);

      } catch (error: any) {
        console.error('[Worker] 获取任务状态失败:', error);
        const response = new Response(JSON.stringify({
          error: error.message || 'Failed to get task status'
        }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);
      }
    }

    // 工具列表查询
    if (url.pathname === '/api/tools/list' && request.method === 'GET') {
      try {
        const tools = await getToolList(env);
        const response = new Response(JSON.stringify({ tools }), {
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);

      } catch (error: any) {
        console.error('[Worker] 获取工具列表失败:', error);
        const response = new Response(JSON.stringify({
          error: error.message || 'Failed to get tool list'
        }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);
      }
    }

    // 工具执行历史查询
    if (url.pathname === '/api/tools/history' && request.method === 'GET') {
      try {
        const urlParams = new URLSearchParams(url.search);
        const limit = parseInt(urlParams.get('limit') || '50');
        const history = await getToolExecutionHistory(limit, env);

        const response = new Response(JSON.stringify({ history }), {
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);

      } catch (error: any) {
        console.error('[Worker] 获取执行历史失败:', error);
        const response = new Response(JSON.stringify({
          error: error.message || 'Failed to get execution history'
        }), {
          status: 500,
          headers: { 'Content-Type': 'application/json' }
        });
        return addCorsHeaders(response);
      }
    }

    // 其他请求直接使用 Rust WASM 处理
    return await rustFetch(request, env, ctx);
  }
};