// TypeScript Worker入口文件
export default {
  async fetch(request: Request, env: any, ctx: ExecutionContext): Promise<Response> {
    try {
      // 处理CORS预检请求
      if (request.method === 'OPTIONS') {
        return new Response(null, {
          status: 200,
          headers: {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, Authorization',
            'Access-Control-Max-Age': '86400',
          }
        });
      }

      // 简单的健康检查端点
      const url = new URL(request.url);
      
      // CORS headers for all responses
      const corsHeaders = {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type, Authorization',
      };
      
      if (url.pathname === '/api/health') {
        return new Response(JSON.stringify({
          status: 'ok',
          message: 'alou-edge worker is running',
          timestamp: new Date().toISOString(),
          environment: env.ENVIRONMENT || 'development'
        }), {
          headers: { 
            'Content-Type': 'application/json',
            ...corsHeaders
          }
        });
      }
      
      if (url.pathname === '/api/status') {
        return new Response(JSON.stringify({
          status: 'running',
          version: '1.0.0',
          features: ['ai-agent', 'blockchain-tools', 'payment-processing']
        }), {
          headers: { 
            'Content-Type': 'application/json',
            ...corsHeaders
          }
        });
      }
      
      // 聊天API端点
      if (url.pathname === '/api/agent/chat' && request.method === 'POST') {
        try {
          const body = await request.json();
          const { session_id, message, wallet_address } = body;
          
          // 简单的AI响应逻辑
          let response = '';
          if (message.toLowerCase().includes('hello') || message.toLowerCase().includes('你好')) {
            response = '你好！我是Alou智能助手，很高兴为您服务！我可以帮助您：\n\n• 查询钱包余额\n• 发送代币交易\n• 查看交易历史\n• 解答区块链相关问题\n\n请告诉我您需要什么帮助？';
          } else if (message.toLowerCase().includes('balance') || message.toLowerCase().includes('余额')) {
            response = '要查询钱包余额，请提供您的钱包地址。我可以帮您查询ETH和ERC-20代币的余额。\n\n请发送您的钱包地址，格式如：0x...';
          } else if (message.toLowerCase().includes('send') || message.toLowerCase().includes('发送')) {
            response = '要发送代币，我需要以下信息：\n\n• 发送方钱包地址\n• 接收方钱包地址\n• 代币类型（ETH或ERC-20代币地址）\n• 发送数量\n\n请提供这些信息，我会帮您构建交易。';
          } else if (message.toLowerCase().includes('transaction') || message.toLowerCase().includes('交易')) {
            response = '我可以帮您：\n\n• 构建交易数据\n• 广播交易到区块链\n• 查询交易状态\n• 查看交易历史\n\n请告诉我您想要进行哪种操作？';
          } else {
            response = `我收到了您的消息："${message}"\n\n作为Alou智能助手，我可以帮助您：\n\n• 💰 查询钱包余额\n• 📤 发送代币交易\n• 📊 查看交易历史\n• 🔍 解答区块链问题\n\n请告诉我您需要什么帮助？`;
          }
          
          return new Response(JSON.stringify({
            content: response,
            session_id: session_id || `session_${Date.now()}`,
            timestamp: Date.now(),
            source: 'alou-edge-ts'
          }), {
            headers: { 
              'Content-Type': 'application/json',
              ...corsHeaders
            }
          });
        } catch (error) {
          return new Response(JSON.stringify({
            content: '抱歉，处理您的请求时出现了错误。请稍后重试。',
            session_id: 'error',
            timestamp: Date.now(),
            source: 'error'
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
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type, Authorization',
        }
      });
    }
  }
};
