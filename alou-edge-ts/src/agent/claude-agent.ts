/**
 * Alou Edge TypeScript - Claude Agent SDK 集成
 */

import Anthropic from '@anthropic-ai/sdk';
import type { Env, ChatMessage, CustomAgentInfo, ToolCall } from '../types';
import { 
  getSystemPrompt, 
  getSystemPromptForCustomAgent, 
  addContextToPrompt,
  detectPromptMode,
  type PromptMode 
} from './prompts';

// 工具定义
const TOOLS: Anthropic.Tool[] = [
  {
    name: 'query_blockchain',
    description: '查询区块链数据，包括余额、交易、合约信息等',
    input_schema: {
      type: 'object' as const,
      properties: {
        action: {
          type: 'string',
          enum: ['eth_balance', 'erc20_balance', 'sol_balance', 'transaction', 'contract'],
          description: '查询类型',
        },
        address: {
          type: 'string',
          description: '钱包地址或合约地址',
        },
        chain: {
          type: 'string',
          description: '区块链网络（如 ethereum, polygon, base）',
        },
        token_address: {
          type: 'string',
          description: 'ERC20代币合约地址（可选）',
        },
      },
      required: ['action', 'address'],
    },
  },
  {
    name: 'build_transaction',
    description: '构建区块链交易',
    input_schema: {
      type: 'object' as const,
      properties: {
        from: {
          type: 'string',
          description: '发送方地址',
        },
        to: {
          type: 'string',
          description: '接收方地址',
        },
        value: {
          type: 'string',
          description: '转账金额（单位为原生代币）',
        },
        chain: {
          type: 'string',
          description: '区块链网络',
        },
        data: {
          type: 'string',
          description: '交易数据（可选，用于合约调用）',
        },
      },
      required: ['from', 'to', 'value', 'chain'],
    },
  },
  {
    name: 'broadcast_transaction',
    description: '广播已签名的交易到区块链网络',
    input_schema: {
      type: 'object' as const,
      properties: {
        signed_tx: {
          type: 'string',
          description: '已签名的交易数据',
        },
        chain: {
          type: 'string',
          description: '区块链网络',
        },
      },
      required: ['signed_tx', 'chain'],
    },
  },
];

// Claude Agent 客户端
export class ClaudeAgent {
  private client: Anthropic;
  private model: string = 'claude-sonnet-4-20250514';
  
  constructor(apiKey: string) {
    this.client = new Anthropic({ apiKey });
  }
  
  /**
   * 处理聊天消息
   */
  async chat(options: {
    message: string;
    history?: ChatMessage[];
    agentInfo?: CustomAgentInfo;
    walletAddress?: string;
    chain?: string;
  }): Promise<{
    response: string;
    tool_calls?: ToolCall[];
  }> {
    const { message, history = [], agentInfo, walletAddress, chain } = options;
    
    // 确定系统 Prompt
    let systemPrompt: string;
    if (agentInfo) {
      // 使用自定义智能体的 Prompt
      systemPrompt = getSystemPromptForCustomAgent(agentInfo);
    } else {
      // 根据消息内容检测 Prompt 模式
      const mode = detectPromptMode(message);
      systemPrompt = getSystemPrompt(mode);
    }
    
    // 添加上下文信息
    systemPrompt = addContextToPrompt(systemPrompt, walletAddress, chain);
    
    // 构建消息历史
    const messages: Anthropic.MessageParam[] = history.map(msg => ({
      role: msg.role,
      content: msg.content,
    }));
    
    // 添加当前消息
    messages.push({
      role: 'user',
      content: message,
    });
    
    // 调用 Claude API
    const response = await this.client.messages.create({
      model: this.model,
      max_tokens: 4096,
      system: systemPrompt,
      tools: TOOLS,
      messages,
    });
    
    // 处理响应
    const toolCalls: ToolCall[] = [];
    let textContent = '';
    
    for (const block of response.content) {
      if (block.type === 'text') {
        textContent += block.text;
      } else if (block.type === 'tool_use') {
        toolCalls.push({
          id: block.id,
          name: block.name,
          input: block.input as Record<string, unknown>,
        });
      }
    }
    
    // 如果有工具调用，处理工具结果并继续对话
    if (toolCalls.length > 0 && response.stop_reason === 'tool_use') {
      const toolResults = await this.executeTools(toolCalls);
      
      // 构建工具结果消息
      const toolResultMessage: Anthropic.MessageParam = {
        role: 'user',
        content: toolResults.map(result => ({
          type: 'tool_result' as const,
          tool_use_id: result.id,
          content: result.output || 'Tool executed successfully',
        })),
      };
      
      // 继续对话
      const followUp = await this.client.messages.create({
        model: this.model,
        max_tokens: 4096,
        system: systemPrompt,
        tools: TOOLS,
        messages: [...messages, { role: 'assistant', content: response.content }, toolResultMessage],
      });
      
      // 提取最终响应
      for (const block of followUp.content) {
        if (block.type === 'text') {
          textContent += block.text;
        }
      }
      
      // 更新工具调用结果
      for (const result of toolResults) {
        const call = toolCalls.find(c => c.id === result.id);
        if (call) {
          call.output = result.output;
        }
      }
    }
    
    return {
      response: textContent,
      tool_calls: toolCalls.length > 0 ? toolCalls : undefined,
    };
  }
  
  /**
   * 执行工具调用
   */
  private async executeTools(toolCalls: ToolCall[]): Promise<{ id: string; output: string }[]> {
    const results: { id: string; output: string }[] = [];
    
    for (const call of toolCalls) {
      try {
        const output = await this.executeTool(call.name, call.input);
        results.push({ id: call.id, output });
      } catch (error) {
        results.push({ 
          id: call.id, 
          output: `Error executing tool: ${error instanceof Error ? error.message : 'Unknown error'}` 
        });
      }
    }
    
    return results;
  }
  
  /**
   * 执行单个工具
   */
  private async executeTool(name: string, input: Record<string, unknown>): Promise<string> {
    switch (name) {
      case 'query_blockchain':
        return this.queryBlockchain(input);
      case 'build_transaction':
        return this.buildTransaction(input);
      case 'broadcast_transaction':
        return this.broadcastTransaction(input);
      default:
        return `Unknown tool: ${name}`;
    }
  }
  
  /**
   * 查询区块链数据
   */
  private async queryBlockchain(input: Record<string, unknown>): Promise<string> {
    const { action, address, chain = 'ethereum' } = input as {
      action: string;
      address: string;
      chain?: string;
    };
    
    // TODO: 实现实际的区块链查询逻辑
    // 这里返回模拟数据
    switch (action) {
      case 'eth_balance':
        return JSON.stringify({
          address,
          chain,
          balance: '1.5',
          unit: 'ETH',
          timestamp: Date.now(),
        });
      case 'erc20_balance':
        return JSON.stringify({
          address,
          chain,
          balance: '1000',
          symbol: 'USDC',
          decimals: 6,
          timestamp: Date.now(),
        });
      default:
        return JSON.stringify({ error: `Unknown action: ${action}` });
    }
  }
  
  /**
   * 构建交易
   */
  private async buildTransaction(input: Record<string, unknown>): Promise<string> {
    const { from, to, value, chain } = input as {
      from: string;
      to: string;
      value: string;
      chain: string;
    };
    
    // TODO: 实现实际的交易构建逻辑
    return JSON.stringify({
      from,
      to,
      value,
      chain,
      gas_estimate: '21000',
      gas_price: '30000000000',
      nonce: 0,
      unsigned_tx: '0x...',
      message: '交易已构建，请使用钱包签名',
    });
  }
  
  /**
   * 广播交易
   */
  private async broadcastTransaction(input: Record<string, unknown>): Promise<string> {
    const { signed_tx, chain } = input as {
      signed_tx: string;
      chain: string;
    };
    
    // TODO: 实现实际的交易广播逻辑
    return JSON.stringify({
      chain,
      tx_hash: '0x' + Math.random().toString(16).slice(2),
      status: 'pending',
      message: '交易已广播，等待确认',
    });
  }
}

/**
 * 创建 Claude Agent 实例
 */
export function createClaudeAgent(env: Env): ClaudeAgent {
  if (!env.ANTHROPIC_API_KEY) {
    throw new Error('ANTHROPIC_API_KEY is required');
  }
  return new ClaudeAgent(env.ANTHROPIC_API_KEY);
}

