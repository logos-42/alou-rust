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
  corsHeaders.set('Access-Control-Allow-Headers', 'Content-Type, Authorization, signal');
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
          'Access-Control-Allow-Headers': 'Content-Type, Authorization, signal',
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
    
    // 工具执行路由 - 直接转发给 Rust WASM 后端处理
    if (url.pathname === '/api/tools/execute' && request.method === 'POST') {
      console.log('[Worker] 工具执行请求转发给 Rust WASM 后端');
      return await rustFetch(request, env, ctx);
    }

    // 工具任务状态查询 - 直接转发给 Rust WASM 后端处理
    if (url.pathname.startsWith('/api/tools/status/') && request.method === 'GET') {
      console.log('[Worker] 工具任务状态查询请求转发给 Rust WASM 后端');
      return await rustFetch(request, env, ctx);
    }

    // 工具列表查询 - 直接转发给 Rust WASM 后端处理
    if (url.pathname === '/api/tools/list' && request.method === 'GET') {
      console.log('[Worker] 工具列表查询请求转发给 Rust WASM 后端');
      return await rustFetch(request, env, ctx);
    }

    // 工具执行历史查询 - 直接转发给 Rust WASM 后端处理
    if (url.pathname === '/api/tools/history' && request.method === 'GET') {
      console.log('[Worker] 工具执行历史查询请求转发给 Rust WASM 后端');
      return await rustFetch(request, env, ctx);
    }

    // 其他请求直接使用 Rust WASM 处理
    return await rustFetch(request, env, ctx);
  }
};