import { useCallback } from 'react';
import agentService from '@/services/agentService';
import type { AgentInfo } from '@shared/types/services';

interface UseAgentCreationParams {
  appendMessage: (message: string | Message) => void;
}

interface AgentCreationResult {
  name: string;
  roleDescription: string;
  description?: string;
  avatar?: string;
  isDefault?: boolean;
}

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  timestamp: number;
  metadata?: Record<string, any>;
}

/**
 * Hook for managing agent creation logic
 */
export const useAgentCreation = ({
  appendMessage,
}: UseAgentCreationParams) => {

  // 直接调用AI解析指令
  const parseAgentCreationCommandWithAIDirect = useCallback(async (text: string): Promise<AgentCreationResult> => {
    console.log('[useAgentMessages] 直接调用AI解析指令:', text);

    try {
      // 移除创建命令关键词
      const createKeywords = ['创建智能体', '新建智能体', 'create agent', 'new agent', '/create', '/new'];
      let cleanedText = text.toLowerCase().trim();
      for (const keyword of createKeywords) {
        cleanedText = cleanedText.replace(keyword.toLowerCase(), '').trim();
      }

      // 如果指令为空，使用默认值
      if (!cleanedText) {
        return {
          name: `智能体_${Date.now().toString().slice(-6)}`,
          roleDescription: '这是一个自动创建的智能体，可以帮助您处理各种任务。',
          isDefault: true
        };
      }

      // 构建AI请求
      const prompt = `请根据以下描述创建一个智能体配置。只返回JSON格式的结果，不要其他文字。

描述: ${cleanedText}

返回格式:
{
  "name": "智能体名称",
  "roleDescription": "智能体的角色描述",
  "description": "智能体的详细描述（可选）",
  "avatar": "头像URL（可选）"
}`;

      const response = await fetch('/api/agent/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: prompt,
          mode: 'agent',
          temperature: 0.3,
        }),
      });

      if (!response.ok) {
        throw new Error(`AI解析失败: ${response.status}`);
      }

      const result = await response.json();
      const aiContent = result.content || result.message;

      // 尝试解析JSON
      try {
        const parsed = JSON.parse(aiContent);
        return {
          name: parsed.name || `智能体_${Date.now().toString().slice(-6)}`,
          roleDescription: parsed.roleDescription || '这是一个自动创建的智能体',
          description: parsed.description,
          avatar: parsed.avatar,
        };
      } catch (parseError) {
        console.warn('[useAgentCreation] AI返回的不是有效JSON，使用默认配置');
        return {
          name: `智能体_${Date.now().toString().slice(-6)}`,
          roleDescription: aiContent.slice(0, 200) || '这是一个自动创建的智能体',
          isDefault: true
        };
      }
    } catch (error) {
      console.error('[useAgentCreation] 解析创建指令失败:', error);
      
      // 返回默认配置
      return {
        name: `智能体_${Date.now().toString().slice(-6)}`,
        roleDescription: '这是一个自动创建的智能体，可以帮助您处理各种任务。',
        isDefault: true
      };
    }
  }, []);

  // 创建智能体
  const createAgent = useCallback(async (agentConfig: AgentCreationResult): Promise<AgentInfo | null> => {
    try {
      console.log('[useAgentCreation] 开始创建智能体:', agentConfig);

      // 构建智能体数据
      const agentData: Partial<AgentInfo> = {
        name: agentConfig.name,
        display_name: agentConfig.name,
        description: agentConfig.description || agentConfig.roleDescription,
        role_description: agentConfig.roleDescription,
        avatar: agentConfig.avatar,
        avatar_url: agentConfig.avatar,
        mode: 'agent',
        status: 'active',
        agent_type: 'custom',
      };

      // 调用服务创建智能体
      const result = await agentService.createAgent(agentData);

      if (result.success && result.data) {
        const newAgent = result.data;
        console.log('[useAgentCreation] 智能体创建成功:', newAgent);

        // 添加成功消息
        const successMessage: Message = {
          id: `msg_${Date.now()}`,
          role: 'assistant',
          content: `✅ 智能体 "${agentConfig.name}" 创建成功！\n\n描述: ${agentConfig.roleDescription}`,
          timestamp: Date.now(),
        };
        appendMessage(successMessage);

        return newAgent;
      } else {
        throw new Error(result.error || '创建智能体失败');
      }
    } catch (error) {
      console.error('[useAgentCreation] 创建智能体失败:', error);
      
      // 添加错误消息
      const errorMessage: Message = {
        id: `msg_${Date.now()}`,
        role: 'assistant',
        content: `❌ 创建智能体失败: ${(error as Error).message}`,
        timestamp: Date.now(),
      };
      appendMessage(errorMessage);

      return null;
    }
  }, [appendMessage]);

  // 处理创建指令
  const handleCreateCommand = useCallback(async (text: string): Promise<void> => {
    try {
      // 解析创建指令
      const agentConfig = await parseAgentCreationCommandWithAIDirect(text);
      
      // 创建智能体
      await createAgent(agentConfig);
    } catch (error) {
      console.error('[useAgentCreation] 处理创建指令失败:', error);
      
      // 添加错误消息
      const errorMessage: Message = {
        id: `msg_${Date.now()}`,
        role: 'assistant',
        content: `❌ 处理创建指令失败: ${(error as Error).message}`,
        timestamp: Date.now(),
      };
      appendMessage(errorMessage);
    }
  }, [parseAgentCreationCommandWithAIDirect, createAgent, appendMessage]);

  // 检查是否为创建指令
  const isCreateCommand = useCallback((text: string): boolean => {
    const createKeywords = [
      '创建智能体', '新建智能体', 'create agent', 'new agent', 
      '/create', '/new', '/agent', '创建', '新建'
    ];
    
    const lowerText = text.toLowerCase().trim();
    return createKeywords.some(keyword => lowerText.includes(keyword.toLowerCase()));
  }, []);

  return {
    parseAgentCreationCommandWithAIDirect,
    createAgent,
    handleCreateCommand,
    isCreateCommand,
  };
};
