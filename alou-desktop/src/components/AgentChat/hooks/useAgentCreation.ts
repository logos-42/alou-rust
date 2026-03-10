import { useCallback } from 'react';
import agentService from '@/services/agentService';
import asyncDiapCreationService from '@/services/asyncDiapCreationService';
// import agentDocumentService from '@/services/agentDocumentService';
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
  customPrompt?: string | null;
  documents?: Record<string, string> | null;
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

  // 直接调用 AI 解析指令
  const parseAgentCreationCommandWithAIDirect = useCallback(async (text: string): Promise<AgentCreationResult> => {
    console.log('[useAgentMessages] 直接调用 AI 解析指令:', text);

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

      // 构建 AI 请求
      const prompt = `请根据以下描述创建一个智能体配置。只返回 JSON 格式的结果，不要其他文字。

描述：${cleanedText}

返回格式:
{
  "name": "智能体名称",
  "roleDescription": "智能体的角色描述",
  "description": "智能体的详细描述（可选）",
  "avatar": "头像 URL（可选）"
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
        throw new Error(`AI 解析失败：${response.status}`);
      }

      const result = await response.json();
      const aiContent = result.content || result.message;

      // 尝试解析 JSON
      let parsedName: string;
      let parsedRoleDescription: string;
      let parsedDescription: string | undefined;
      let parsedAvatar: string | undefined;

      try {
        const parsed = JSON.parse(aiContent);
        parsedName = parsed.name || `智能体_${Date.now().toString().slice(-6)}`;
        parsedRoleDescription = parsed.roleDescription || '这是一个自动创建的智能体';
        parsedDescription = parsed.description;
        parsedAvatar = parsed.avatar;
      } catch (parseError) {
        console.warn('[useAgentCreation] AI 返回的不是有效 JSON，使用默认配置');
        parsedName = `智能体_${Date.now().toString().slice(-6)}`;
        parsedRoleDescription = aiContent.slice(0, 200) || '这是一个自动创建的智能体';
      }

      // 生成文档集，构建 customPrompt
      const customPrompt: string | null = null;
      const documentsMap: Record<string, string> | null = null;
      try {
        console.log('[useAgentCreation] 开始为自动创建的智能体生成文档集...');
        // TODO: 实现文档生成逻辑
        // const userPrompt = `创建一个名为"${parsedName}"的智能体，角色描述：${parsedRoleDescription}`;
        // const agentDocuments = await agentDocumentService.generateFullDocumentSet(userPrompt, {
        //   name: parsedName,
        // });
        // customPrompt = agentDocumentService.buildSystemPromptFromDocuments(agentDocuments);
        // documentsMap = agentDocumentService.extractDocumentMap(agentDocuments);
        console.log('[useAgentCreation] 文档集生成跳过（待实现）');
      } catch (docErr) {
        console.warn('[useAgentCreation] 文档集生成失败，将使用无文档模式:', (docErr as Error).message);
      }

      return {
        name: parsedName,
        roleDescription: parsedRoleDescription,
        description: parsedDescription,
        avatar: parsedAvatar,
        customPrompt,
        documents: documentsMap,
      };
    } catch (error) {
      console.error('[useAgentCreation] 解析创建指令失败:', error);

      // 返回默认配置
      return {
        name: `智能体_${Date.now().toString().slice(-6)}`,
        roleDescription: '这是一个自动创建的智能体，可以帮助您处理各种任务。',
        isDefault: true,
        customPrompt: null,
      };
    }
  }, []);

  // 创建智能体
  const createAgent = useCallback(async (agentConfig: AgentCreationResult): Promise<AgentInfo | null> => {
    try {
      console.log('[useAgentCreation] 开始创建智能体:', agentConfig);

      // 构建智能体数据
      const agentData: Partial<AgentInfo> & { customPrompt?: string | null; documents?: Record<string, string> | null } = {
        name: agentConfig.name,
        display_name: agentConfig.name,
        description: agentConfig.description || agentConfig.roleDescription,
        role_description: agentConfig.roleDescription,
        avatar: agentConfig.avatar,
        avatar_url: agentConfig.avatar,
        mode: 'agent',
        status: 'active',
        agent_type: 'custom',
        // 传递 AI 生成的文档系统提示词
        customPrompt: agentConfig.customPrompt ?? null,
        // 传递单独文档 map
        documents: agentConfig.documents ?? null,
      };

      console.log('[useAgentCreation] 构建智能体数据，hasCustomPrompt:', !!agentConfig.customPrompt, 'promptLen:', agentConfig.customPrompt?.length ?? 0);

      // 调用服务创建智能体
      const result = await agentService.createAgent(agentData);

      if (result.success && result.data) {
        const newAgent = result.data;
        console.log('[useAgentCreation] 智能体创建成功:', newAgent);

        // 确保 mode 属性类型正确
        const typedAgent: AgentInfo = {
          ...newAgent,
          mode: (newAgent.mode as 'agent' | 'alou') || 'agent',
        };

        // 异步启动 DIAP 身份创建（后台执行，进度显示在右上角 DIAP 身份面板）
        const sessionId = typedAgent.id || typedAgent.sessionId
        if (sessionId) {
          console.log('[useAgentCreation] 启动异步 DIAP 身份创建:', sessionId)
          asyncDiapCreationService.startDiapCreation(
            sessionId,
            {
              name: agentConfig.name,
              roleDescription: agentConfig.roleDescription,
            },
            { maxRetries: 3 }
          )
        }

        // 添加成功消息
        const successMessage: Message = {
          id: `msg_${Date.now()}`,
          role: 'assistant',
          content: `✅ 智能体 "${agentConfig.name}" 创建成功！\n\n描述：${agentConfig.roleDescription}`,
          timestamp: Date.now(),
        };
        appendMessage(successMessage);

        return typedAgent;
      } else {
        throw new Error(result.error || '创建智能体失败');
      }
    } catch (error) {
      console.error('[useAgentCreation] 创建智能体失败:', error);

      // 添加错误消息
      const errorMessage: Message = {
        id: `msg_${Date.now()}`,
        role: 'assistant',
        content: `❌ 创建智能体失败：${(error as Error).message}`,
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
        content: `❌ 处理创建指令失败：${(error as Error).message}`,
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
