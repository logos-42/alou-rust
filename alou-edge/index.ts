// TypeScript Worker入口文件 - AI Agent 实现

// Cloudflare Workers types
interface Env {
  AI_API_KEY?: string;
  CLAUDE_API_KEY?: string;
  AI_PROVIDER?: string;
  AI_MODEL?: string;
  ENVIRONMENT?: string;
  JWT_SECRET?: string;
  ETH_RPC_URL?: string;
  SOL_RPC_URL?: string;
  SESSIONS?: any; // KVNamespace
  CACHE?: any; // KVNamespace
  NONCES?: any; // KVNamespace
}

// CORS headers for all responses
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};

export default {
  async fetch(request: Request, env: Env, ctx: any): Promise<Response> {
    try {
      // 处理CORS预检请求
      if (request.method === 'OPTIONS') {
        return new Response(null, {
          status: 200,
          headers: {
            ...corsHeaders,
            'Access-Control-Max-Age': '86400',
          }
        });
      }

      console.log(`[${new Date().toISOString()}] ${request.method} ${new URL(request.url).pathname}`);

      const url = new URL(request.url);
      
      // 健康检查
      if (url.pathname === '/api/health') {
        return new Response(JSON.stringify({
          status: 'ok',
          message: 'alou-edge worker is running (TypeScript fallback)',
          timestamp: new Date().toISOString(),
          environment: env.ENVIRONMENT || 'development',
          mode: 'typescript-fallback'
        }), {
          headers: { 
            'Content-Type': 'application/json',
            ...corsHeaders
          }
        });
      }
      
      // 状态检查
      if (url.pathname === '/api/status') {
        return new Response(JSON.stringify({
          status: 'running',
          version: '1.0.0',
          mode: 'typescript-fallback',
          features: ['ai-agent', 'blockchain-tools', 'payment-processing']
        }), {
          headers: { 
            'Content-Type': 'application/json',
            ...corsHeaders
          }
        });
      }
      
      // 聊天API端点 - 调用真实AI
      if (url.pathname === '/api/agent/chat' && request.method === 'POST') {
        try {
          const body = await request.json();
          const { session_id, message } = body;
          
          // 调用AI API (DeepSeek)
          const aiApiKey = env.AI_API_KEY || env.CLAUDE_API_KEY;
          const aiProvider = env.AI_PROVIDER || 'deepseek';
          const aiModel = env.AI_MODEL || 'deepseek-chat';
          
          if (!aiApiKey) {
            throw new Error('AI_API_KEY not configured');
          }

          console.log(`→ Calling ${aiProvider} API with model ${aiModel}`);
          
          // 构建AI请求
          const aiEndpoint = aiProvider === 'deepseek' 
            ? 'https://api.deepseek.com/v1/chat/completions'
            : aiProvider === 'qwen'
            ? 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions'
            : 'https://api.openai.com/v1/chat/completions';

          const aiResponse = await fetch(aiEndpoint, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${aiApiKey}`,
            },
            body: JSON.stringify({
              model: aiModel,
              messages: [
                {
                  role: 'system',
                  content: '你是Alou智能助手，一个专业的Web3和区块链AI助手。你可以帮助用户查询钱包余额、构建交易、解答区块链问题。请用友好、专业的方式回答用户问题。'
                },
                {
                  role: 'user',
                  content: message
                }
              ],
              temperature: 0.7,
              max_tokens: 2000,
            }),
          });

          if (!aiResponse.ok) {
            const errorText = await aiResponse.text();
            console.error('AI API error:', errorText);
            throw new Error(`AI API returned ${aiResponse.status}: ${errorText}`);
          }

          const aiData = await aiResponse.json();
          const content = aiData.choices?.[0]?.message?.content || '抱歉，我现在无法回答。';
          
          console.log('✓ AI response received');
          
          return new Response(JSON.stringify({
            content,
            session_id: session_id || `session_${Date.now()}`,
            timestamp: Date.now(),
            source: 'ai-powered',
            provider: aiProvider,
            model: aiModel
          }), {
            headers: { 
              'Content-Type': 'application/json',
              ...corsHeaders
            }
          });
        } catch (error) {
          console.error('Chat error:', error);
          return new Response(JSON.stringify({
            content: '抱歉，处理您的请求时出现了错误。请稍后重试。',
            error: error instanceof Error ? error.message : 'Unknown error',
            session_id: 'error',
            timestamp: Date.now(),
          }), {
            status: 500,
            headers: { 
              'Content-Type': 'application/json',
              ...corsHeaders
            }
          });
        }
      }
      
      // 会话管理端点
      if (url.pathname === '/api/session' && request.method === 'POST') {
        const sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        return new Response(JSON.stringify({
          session_id: sessionId,
          created_at: new Date().toISOString()
        }), {
          headers: { 
            'Content-Type': 'application/json',
            ...corsHeaders
          }
        });
      }
      
      // 默认响应
      return new Response(JSON.stringify({
        message: 'Welcome to alou-edge API',
        mode: 'typescript-fallback',
        endpoints: [
          '/api/health',
          '/api/status',
          '/api/agent/chat',
          '/api/session'
        ]
      }), {
        headers: { 
          'Content-Type': 'application/json',
          ...corsHeaders
        }
      });
      
    } catch (error) {
      console.error('Worker error:', error);
      return new Response(JSON.stringify({
        status: 'error',
        message: 'Internal server error',
        error: error instanceof Error ? error.message : 'Unknown error'
      }), {
        status: 500,
        headers: { 
          'Content-Type': 'application/json',
          ...corsHeaders
        }
      });
    }
  }
};
