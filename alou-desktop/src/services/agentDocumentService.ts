/**
 * Agent文档服务
 * 处理agent的文档更新和持久化到 .md 文件
 */

import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import useAgentStore from '@/stores/agentStore';

export interface DocumentUpdatePayload {
  document_type: string;
  new_content: string;
  reason: string;
  agent_id?: string;  // Rust 事件会携带正确的 agent_id
  persisted_to_file?: boolean;  // 是否已成功保存到文件
}

export interface AgentDocuments {
  memory?: string;
  soul?: string;
  identity?: string;
  capabilities?: string;
  constraints?: string;
  tools?: string;
  agents?: string;
  ipfs?: string;
  user?: string;
  project?: string;
  key?: string;
}

// 为了兼容性，导出Document类型
export interface Document {
  content: string;
  cid?: string;
  metadata?: Record<string, any>;
}

// 文档类型枚举
export enum DocumentTypes {
  MEMORY = 'memory',
  SOUL = 'soul',
  IDENTITY = 'identity',
  CAPABILITIES = 'capabilities',
  CONSTRAINTS = 'constraints',
  TOOLS = 'tools',
  AGENTS = 'agents',
  IPFS = 'ipfs',
  USER = 'user',
  PROJECT = 'project',
  KEY = 'key'
}

class AgentDocumentService {
  private listeners: Array<() => void> = [];
  private agentDocsPath = '.alou'; // 相对于AppData目录

  /**
   * 初始化文档服务，监听来自Rust后端的文档更新事件
   */
  async initialize() {
    console.log('[AgentDocumentService] 初始化文档服务');

    // 确保文档目录存在
    await this.ensureDocumentsDirectory();

    // 监听document:updated事件
    const unlisten = await listen<DocumentUpdatePayload>('document:updated', (event) => {
      console.log('[AgentDocumentService] 收到文档更新事件:', event.payload);
      this.handleDocumentUpdate(event.payload);
    });

    this.listeners.push(unlisten);
  }

  /**
   * 确保文档目录存在
   */
  private async ensureDocumentsDirectory() {
    try {
      const dirExists = await exists(this.agentDocsPath, { baseDir: undefined });
      if (!dirExists) {
        await mkdir(this.agentDocsPath, { baseDir: undefined, recursive: true });
        console.log('[AgentDocumentService] 创建文档目录:', this.agentDocsPath);
      }
    } catch (error) {
      console.error('[AgentDocumentService] 创建文档目录失败:', error);
    }
  }

  /**
   * 获取agent的文档目录路径
   */
  private getAgentDocPath(agentId: string): string {
    return `${this.agentDocsPath}/${agentId}`;
  }

  /**
   * 获取文档文件路径
   */
  private getDocumentPath(agentId: string, documentType: string): string {
    return `${this.getAgentDocPath(agentId)}/${documentType.toUpperCase()}.md`;
  }

  /**
   * 处理文档更新
   * 注意：Rust 后端已经直接写入文件，这里只需要更新前端状态
   */
  private async handleDocumentUpdate(payload: DocumentUpdatePayload) {
    const { document_type, new_content, reason, agent_id } = payload;

    console.log(`[AgentDocumentService] 更新文档: ${document_type}, 原因: ${reason}, agent_id: ${agent_id}`);

    // 优先使用事件中携带的正确 agent_id
    const currentAgentId = agent_id || this.getCurrentAgentId();
    if (!currentAgentId) {
      console.warn('[AgentDocumentService] 未找到 agent ID');
      return;
    }

    // 注意：Rust 后端已经写入文件，这里只需要更新前端状态（如果需要）
    // 触发自定义事件，通知其他组件
    window.dispatchEvent(new CustomEvent('agent-document-updated', {
      detail: { 
        agentId: currentAgentId, 
        documentType: document_type, 
        content: new_content,
        filePath: this.getDocumentPath(currentAgentId, document_type)
      }
    }));
  }

  /**
   * 获取当前agent ID
   */
  private getCurrentAgentId(): string | null {
    // 优先从 agentStore 获取当前智能体 ID
    try {
      const state = useAgentStore.getState();
      const agents = state.agents;
      if (agents && agents.length > 0) {
        // 返回第一个智能体的 ID（通常是最近使用的）
        const latestAgent = agents[0];
        if (latestAgent && latestAgent.id) {
          console.log('[AgentDocumentService] 从 agentStore 获取当前智能体 ID:', latestAgent.id);
          return latestAgent.id;
        }
      }
    } catch (error) {
      console.warn('[AgentDocumentService] 从 agentStore 获取智能体 ID 失败:', error);
    }

    // 备选方案：从 localStorage 获取当前选中的 agent
    const selectedAgent = localStorage.getItem('selectedAgent');
    if (selectedAgent) {
      try {
        const agent = JSON.parse(selectedAgent);
        return agent.id;
      } catch (error) {
        console.error('[AgentDocumentService] 解析 selectedAgent 失败:', error);
      }
    }
    
    // 备选方案：从活跃频道 ID 获取智能体 ID
    try {
      const activeChannelId = localStorage.getItem('activeChannelId');
      if (activeChannelId) {
        console.log('[AgentDocumentService] 使用活跃频道 ID 作为智能体 ID:', activeChannelId);
        return activeChannelId;
      }
    } catch (error) {
      console.warn('[AgentDocumentService] 获取活跃频道 ID 失败:', error);
    }
    
    return null;
  }

  /**
   * 读取agent的文档
   */
  async getAgentDocument(agentId: string, documentType: string): Promise<string | null> {
    try {
      const filePath = this.getDocumentPath(agentId, documentType);
      const fileExists = await exists(filePath, { baseDir: undefined });
      
      if (!fileExists) {
        console.log(`[AgentDocumentService] 文档不存在: ${filePath}`);
        return null;
      }

      const content = await readTextFile(filePath, { baseDir: undefined });
      return content;
    } catch (error) {
      console.error('[AgentDocumentService] 读取文档失败:', error);
      return null;
    }
  }

  /**
   * 读取agent的所有文档
   */
  async getAgentDocuments(agentId: string): Promise<AgentDocuments> {
    const documentTypes = ['memory', 'soul', 'identity', 'capabilities', 'constraints', 'tools', 'agents', 'ipfs', 'user', 'project', 'key'];
    const documents: AgentDocuments = {};

    for (const type of documentTypes) {
      const content = await this.getAgentDocument(agentId, type);
      if (content) {
        documents[type as keyof AgentDocuments] = content;
      }
    }

    return documents;
  }

  /**
   * 手动更新agent文档
   */
  async updateAgentDocument(agentId: string, documentType: string, content: string): Promise<boolean> {
    try {
      // 确保agent文档目录存在
      const agentDocPath = this.getAgentDocPath(agentId);
      const dirExists = await exists(agentDocPath, { baseDir: undefined });
      if (!dirExists) {
        await mkdir(agentDocPath, { baseDir: undefined, recursive: true });
      }

      // 写入文档文件
      const filePath = this.getDocumentPath(agentId, documentType);
      await writeTextFile(filePath, content, { baseDir: undefined });

      console.log(`[AgentDocumentService] 手动更新文档成功: ${filePath}`);
      return true;
    } catch (error) {
      console.error('[AgentDocumentService] 手动更新文档失败:', error);
      return false;
    }
  }

  /**
   * 初始化agent的文档（如果不存在）
   */
  async initializeAgentDocuments(agentId: string, agentInfo?: any): Promise<void> {
    const documentTypes = ['memory', 'soul', 'identity', 'capabilities', 'constraints', 'tools', 'agents', 'ipfs', 'user', 'project', 'key'];

    for (const type of documentTypes) {
      const filePath = this.getDocumentPath(agentId, type);
      const fileExists = await exists(filePath, { baseDir: undefined });

      if (!fileExists) {
        const initialContent = this.getDocumentInitialContent(type, agentInfo);
        await this.updateAgentDocument(agentId, type, initialContent);
        console.log(`[AgentDocumentService] 初始化文档: ${type}`);
      }
    }
  }

  /**
   * 获取文档的初始内容
   */
  private getDocumentInitialContent(documentType: string, agentInfo?: any): string {
    const templates: Record<string, string> = {
      memory: `# 长期记忆

这里存储跨会话的重要信息。

## 用户偏好
（暂无记录）

## 项目信息
（暂无记录）

## 学到的知识
（暂无记录）

## 重要对话
（暂无记录）

---
最后更新: ${new Date().toISOString()}`,

      soul: `# 核心身份
**智能体 ID**: ${agentInfo?.id || 'unknown'}
**名称**: ${agentInfo?.name || '智能体'}

${agentInfo?.name || '智能体'}的核心特质和价值观。

## 角色定位
${agentInfo?.role_description || '专业的AI助手'}

## 核心价值观
- 准确性：提供准确可靠的信息
- 效率：快速完成任务
- 安全性：注重操作安全
- 学习性：从每次交互中学习和改进

## 个性特点
- 友好且专业
- 注重细节
- 善于沟通

---
最后更新: ${new Date().toISOString()}`,

      identity: `# 身份定义

**智能体 ID**: ${agentInfo?.id || 'unknown'}

## 名称
${agentInfo?.name || '智能体'}

## 角色
${agentInfo?.role_description || '专业的AI助手'}

## 专长领域
（根据实际使用情况更新）

## 工作方式
- 理解用户需求
- 选择合适工具
- 执行任务
- 反馈结果

---
最后更新: ${new Date().toISOString()}`,

      capabilities: `# 能力清单

## 核心能力
- 文件操作：读取、写入、编辑、搜索文件
- 终端命令：执行系统命令
- 网络操作：搜索信息、获取网页内容
- 任务规划：制定和管理任务计划
- 代码理解：分析和修改代码

## 工具使用
- 熟练使用所有可用工具
- 能够组合多个工具完成复杂任务
- 理解工具的限制和最佳实践

## 学习能力
- 从用户反馈中学习
- 记录成功的解决方案
- 避免重复错误

---
最后更新: ${new Date().toISOString()}`,

      constraints: `# 约束和限制

## 操作限制
- 不执行危险命令
- 不访问敏感文件
- 不进行未经授权的网络操作

## 行为准则
- 始终征求用户确认重要操作
- 清晰解释操作步骤
- 提供操作结果反馈

## 安全原则
- 保护用户数据安全
- 遵守系统安全策略
- 及时报告异常情况

---
最后更新: ${new Date().toISOString()}`,

      tools: `# 工具使用记录

## 常用工具
（根据实际使用情况更新）

## 工具组合
（记录有效的工具组合方案）

## 最佳实践
（记录工具使用的最佳实践）

---
最后更新: ${new Date().toISOString()}`,

      agents: `# 协作智能体

## 已知智能体
（暂无记录）

## 协作经验
（暂无记录）

## 协作模式
（暂无记录）

---
最后更新: ${new Date().toISOString()}`,

      ipfs: `# IPFS 对话历史索引

这里记录了存储在 IPFS 上的重要对话会话。

## 最近会话
（暂无记录）

## 统计信息
- 总会话数：0
- 总消息数：0

---
最后更新: ${new Date().toISOString()}`,

      user: `# 用户喜好与偏好

这里记录了用户的个人偏好、习惯和工作方式。

## 用户偏好
（暂无记录）

## 沟通风格
（暂无记录）

## 技术栈偏好
（暂无记录）

---
最后更新: ${new Date().toISOString()}`,

      project: `# 项目与工作报告

这里记录了参与的项目、工作报告和重要成果。

## 当前项目
（暂无记录）

## 已完成项目
（暂无记录）

## 工作报告
（暂无记录）

---
最后更新: ${new Date().toISOString()}`,

      key: `# 关键密钥路径

这里记录了重要密钥和凭证的存储路径（不存储实际密钥）。

## 钱包密钥
（暂无记录）

## API 密钥
（暂无记录）

## 安全提醒
- ⚠️ 本文件只记录路径，不存储实际密钥

---
最后更新: ${new Date().toISOString()}`
    };

    return templates[documentType] || `# ${documentType.toUpperCase()}\n\n（暂无内容）\n\n---\n最后更新: ${new Date().toISOString()}`;
  }

  /**
   * 获取文档文件的绝对路径（用于AI读取）
   */
  async getDocumentAbsolutePath(agentId: string, documentType: string): Promise<string | null> {
    try {
      // 通过Tauri API获取AppData目录的绝对路径
      const appDataPath = await invoke<string>('plugin:path|resolve', {
        directory: undefined
      });
      
      const relativePath = this.getDocumentPath(agentId, documentType);
      return `${appDataPath}/${relativePath}`;
    } catch (error) {
      console.error('[AgentDocumentService] 获取绝对路径失败:', error);
      return null;
    }
  }

  /**
   * 清理监听器
   */
  cleanup() {
    this.listeners.forEach(unlisten => unlisten());
    this.listeners = [];
  }
}

// 导出单例
export const agentDocumentService = new AgentDocumentService();
export default agentDocumentService;
