/**
 * TypeScript Agent - Claude Agent SDK 集成
 * 使用官方 @anthropic-ai/sdk
 */

import Anthropic from '@anthropic-ai/sdk';
import type { CustomAgentInfo, ChatMessage, ToolCall } from './types';
import { 
  getSystemPrompt, 
  getSystemPromptForCustomAgent, 
  addContextToPrompt,
  detectPromptMode,
} from './prompts';

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
    
    // 调用 Claude API（不使用工具，工具由 Rust 后端处理）
    const response = await this.client.messages.create({
      model: this.model,
      max_tokens: 4096,
      system: systemPrompt,
      messages,
    });
    
    // 处理响应
    let textContent = '';
    
    for (const block of response.content) {
      if (block.type === 'text') {
        textContent += block.text;
      }
    }
    
    return {
      response: textContent,
      tool_calls: undefined, // 工具调用由 Rust 后端处理
    };
  }
}

/**
 * 创建 Claude Agent 实例
 */
export function createClaudeAgent(apiKey: string): ClaudeAgent {
  if (!apiKey) {
    throw new Error('ANTHROPIC_API_KEY is required');
  }
  return new ClaudeAgent(apiKey);
}

