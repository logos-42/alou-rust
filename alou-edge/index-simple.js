// 简化的Worker入口文件，直接使用WASM
import wasmModule from './target/wasm32-unknown-unknown/release/alou_edge.wasm';

export default {
  async fetch(request, env, ctx) {
    try {
      // 直接使用WASM模块
      const instance = await WebAssembly.instantiate(wasmModule, {
        env: {
          // 提供必要的环境变量
          ...env,
        }
      });
      
      // 调用WASM中的fetch函数
      if (instance.exports.fetch) {
        return await instance.exports.fetch(request, env, ctx);
      }
      
      // 如果没有fetch函数，返回基本信息
      return new Response(JSON.stringify({
        status: 'ok',
        message: 'alou-edge worker is running',
        timestamp: new Date().toISOString()
      }), {
        headers: { 'Content-Type': 'application/json' }
      });
      
    } catch (error) {
      console.error('WASM execution error:', error);
      return new Response(JSON.stringify({
        status: 'error',
        message: 'WASM execution failed',
        error: error.message
      }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' }
      });
    }
  }
};

