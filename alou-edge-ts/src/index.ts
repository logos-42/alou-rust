/**
 * Alou Edge TypeScript - Cloudflare Workers 入口
 * 使用 Claude Agent SDK 的 TypeScript 后端服务
 */

import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import type { 
  Env, 
  ChatRequest, 
  ChatResponse, 
  CreateSessionRequest, 
  CreateSessionResponse,
  CreateAgentRequest,
  SessionData,
  CustomAgentInfo,
} from './types';
import { createClaudeAgent, ClaudeAgent } from './agent/claude-agent';

// 创建 Hono 应用
const app = new Hono<{ Bindings: Env }>();

// 中间件
app.use('*', logger());
app.use('*', cors({
  origin: '*',
  allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowHeaders: ['Content-Type', 'Authorization'],
}));

// 健康检查
app.get('/health', (c) => {
  return c.json({ 
    status: 'ok', 
    service: 'alou-edge-ts',
    version: '0.1.0',
    timestamp: Date.now(),
  });
});

// 服务状态
app.get('/status', (c) => {
  return c.json({
    status: 'running',
    environment: c.env.ENVIRONMENT,
    features: {
      claude_agent_sdk: true,
      diap_integration: true,
      blockchain_tools: true,
    },
  });
});

// ============ 会话管理 ============

// 创建会话
app.post('/session', async (c) => {
  try {
    const body = await c.req.json<CreateSessionRequest>();
    const sessionId = crypto.randomUUID();
    const now = Date.now();
    
    const sessionData: SessionData = {
      id: sessionId,
      wallet_address: body.wallet_address,
      created_at: now,
      updated_at: now,
      messages: [],
    };
    
    // 存储到 KV
    await c.env.SESSIONS.put(sessionId, JSON.stringify(sessionData), {
      expirationTtl: 86400 * 7, // 7 天过期
    });
    
    const response: CreateSessionResponse = {
      success: true,
      session_id: sessionId,
      created_at: now,
    };
    
    return c.json(response);
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to create session',
    }, 500);
  }
});

// 获取会话
app.get('/session/:id', async (c) => {
  try {
    const sessionId = c.req.param('id');
    const sessionData = await c.env.SESSIONS.get(sessionId);
    
    if (!sessionData) {
      return c.json({ success: false, error: 'Session not found' }, 404);
    }
    
    return c.json({
      success: true,
      session: JSON.parse(sessionData),
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to get session',
    }, 500);
  }
});

// 删除会话
app.delete('/session/:id', async (c) => {
  try {
    const sessionId = c.req.param('id');
    await c.env.SESSIONS.delete(sessionId);
    
    return c.json({ success: true });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to delete session',
    }, 500);
  }
});

// ============ 智能体聊天 ============

// 发送消息
app.post('/agent/chat', async (c) => {
  try {
    const body = await c.req.json<ChatRequest>();
    const { session_id, message, wallet_address, chain } = body;
    
    // 获取会话数据
    const sessionRaw = await c.env.SESSIONS.get(session_id);
    let sessionData: SessionData;
    
    if (sessionRaw) {
      sessionData = JSON.parse(sessionRaw);
    } else {
      // 创建新会话
      sessionData = {
        id: session_id,
        wallet_address,
        chain,
        created_at: Date.now(),
        updated_at: Date.now(),
        messages: [],
      };
    }
    
    // 更新钱包地址和链
    if (wallet_address) sessionData.wallet_address = wallet_address;
    if (chain) sessionData.chain = chain;
    
    // 创建 Claude Agent
    const agent = createClaudeAgent(c.env);
    
    // 调用 Claude Agent
    const result = await agent.chat({
      message,
      history: sessionData.messages,
      agentInfo: sessionData.agent_info,
      walletAddress: sessionData.wallet_address,
      chain: sessionData.chain,
    });
    
    // 更新会话消息历史
    sessionData.messages.push({
      role: 'user',
      content: message,
      timestamp: Date.now(),
    });
    
    sessionData.messages.push({
      role: 'assistant',
      content: result.response,
      timestamp: Date.now(),
      tool_calls: result.tool_calls,
    });
    
    sessionData.updated_at = Date.now();
    
    // 保存会话
    await c.env.SESSIONS.put(session_id, JSON.stringify(sessionData), {
      expirationTtl: 86400 * 7,
    });
    
    const response: ChatResponse = {
      success: true,
      response: result.response,
      tool_calls: result.tool_calls,
    };
    
    return c.json(response);
  } catch (error) {
    console.error('Chat error:', error);
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Chat failed',
    }, 500);
  }
});

// ============ 智能体管理 ============

// 创建/更新智能体
app.post('/agent/create-claude', async (c) => {
  try {
    const body = await c.req.json<CreateAgentRequest>();
    const { session_id, name, role_description, avatar_cid, mcp_config_cid, diap_identity } = body;
    
    // 获取会话
    const sessionRaw = await c.env.SESSIONS.get(session_id);
    let sessionData: SessionData;
    
    if (sessionRaw) {
      sessionData = JSON.parse(sessionRaw);
    } else {
      sessionData = {
        id: session_id,
        created_at: Date.now(),
        updated_at: Date.now(),
        messages: [],
      };
    }
    
    // 创建智能体信息
    const agentInfo: CustomAgentInfo = {
      name,
      role_description: role_description || '',
      avatar_cid,
      mcp_config_cid,
      did: diap_identity?.did,
      ipns: diap_identity?.ipns,
      cid: diap_identity?.cid,
    };
    
    sessionData.agent_info = agentInfo;
    sessionData.updated_at = Date.now();
    
    // 保存会话
    await c.env.SESSIONS.put(session_id, JSON.stringify(sessionData), {
      expirationTtl: 86400 * 7,
    });
    
    return c.json({
      success: true,
      agent: agentInfo,
      session_id,
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to create agent',
    }, 500);
  }
});

// 解析智能体
app.post('/agent/resolve', async (c) => {
  try {
    const body = await c.req.json<{ target: string; session_id?: string }>();
    const { target } = body;
    
    // TODO: 实现通过 IPFS/IPNS 解析智能体
    // 这里返回模拟数据
    
    return c.json({
      success: true,
      agent: {
        name: 'Resolved Agent',
        did: `did:alou:${target}`,
        cid: target,
      },
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to resolve agent',
    }, 500);
  }
});

// 搜索智能体
app.post('/agent/search', async (c) => {
  try {
    const body = await c.req.json<{ query: string }>();
    
    // TODO: 实现智能体搜索
    
    return c.json({
      success: true,
      results: [],
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Search failed',
    }, 500);
  }
});

// ============ 区块链操作 ============

// 查询余额
app.post('/blockchain/balance', async (c) => {
  try {
    const body = await c.req.json<{
      address: string;
      chain: string;
      token_address?: string;
    }>();
    
    // TODO: 实现实际的余额查询
    
    return c.json({
      success: true,
      balance: '1.5',
      symbol: 'ETH',
      chain: body.chain,
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Balance query failed',
    }, 500);
  }
});

// 构建交易
app.post('/blockchain/transaction/build', async (c) => {
  try {
    const body = await c.req.json<{
      from: string;
      to: string;
      value: string;
      chain: string;
    }>();
    
    // TODO: 实现实际的交易构建
    
    return c.json({
      success: true,
      unsigned_tx: '0x...',
      gas_estimate: '21000',
      gas_price: '30000000000',
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Transaction build failed',
    }, 500);
  }
});

// 广播交易
app.post('/blockchain/transaction/broadcast', async (c) => {
  try {
    const body = await c.req.json<{
      signed_tx: string;
      chain: string;
    }>();
    
    // TODO: 实现实际的交易广播
    
    return c.json({
      success: true,
      tx_hash: '0x' + Math.random().toString(16).slice(2),
      status: 'pending',
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Broadcast failed',
    }, 500);
  }
});

// 获取交易状态
app.get('/blockchain/transaction/:hash', async (c) => {
  try {
    const hash = c.req.param('hash');
    const chain = c.req.query('chain') || 'ethereum';
    
    // TODO: 实现实际的交易状态查询
    
    return c.json({
      success: true,
      tx_hash: hash,
      chain,
      status: 'confirmed',
      confirmations: 12,
    });
  } catch (error) {
    return c.json({
      success: false,
      error: error instanceof Error ? error.message : 'Status query failed',
    }, 500);
  }
});

// 导出应用
export default app;

