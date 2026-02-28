# Agent记忆系统使用指南

## 概述

Alou的记忆系统允许AI智能体跨会话保存和检索重要信息。所有记忆都存储在Markdown文档中，AI可以通过工具读取和编辑这些文档。

## 文档结构

每个agent都有自己的文档目录：
```
~/Library/Application Support/com.alou.desktop/agent-documents/{agent_id}/
├── MEMORY.md          # 长期记忆
├── SOUL.md            # 核心身份
├── IDENTITY.md        # 身份定义
├── CAPABILITIES.md    # 能力清单
├── CONSTRAINTS.md     # 约束限制
├── TOOLS.md           # 工具记录
└── AGENTS.md          # 协作记录
```

## 文档说明

### 1. MEMORY.md - 长期记忆
存储跨会话的重要信息：
- 用户偏好和习惯
- 项目信息和上下文
- 学到的知识和经验
- 重要对话记录

**示例内容：**
```markdown
# 长期记忆

## 用户偏好
- 喜欢简洁的代码风格
- 使用TypeScript作为主要语言
- 偏好函数式编程

## 项目信息
- 当前项目：Alou Desktop
- 技术栈：Tauri + React + TypeScript
- 主要功能：AI智能体平台

## 学到的知识
- 用户经常使用git_helper工具进行版本控制
- filesystem工具的edit操作比write更高效
```

### 2. SOUL.md - 核心身份
定义agent的核心特质：
- 角色定位
- 核心价值观
- 个性特点

### 3. IDENTITY.md - 身份定义
明确agent的身份：
- 名称和角色
- 专长领域
- 工作方式

### 4. CAPABILITIES.md - 能力清单
记录agent的能力：
- 核心能力
- 工具使用经验
- 学习能力

### 5. CONSTRAINTS.md - 约束限制
定义行为边界：
- 操作限制
- 行为准则
- 安全原则

### 6. TOOLS.md - 工具记录
记录工具使用经验：
- 常用工具
- 有效的工具组合
- 最佳实践

### 7. AGENTS.md - 协作记录
记录与其他agent的协作：
- 已知智能体
- 协作经验
- 协作模式

## AI如何使用记忆系统

### 方法1：使用agent_document工具

#### 读取记忆
```json
{
  "action": "read",
  "document_type": "memory"
}
```

#### 更新记忆
```json
{
  "action": "update",
  "document_type": "memory",
  "new_content": "# 长期记忆\n\n## 用户偏好\n- 喜欢简洁的代码\n...",
  "reason": "记录用户偏好"
}
```

### 方法2：使用filesystem工具

AI也可以直接使用filesystem工具读取和编辑.md文件：

#### 读取文档
```json
{
  "operation": "read",
  "path": "~/Library/Application Support/com.alou.desktop/agent-documents/{agent_id}/MEMORY.md"
}
```

#### 编辑文档
```json
{
  "operation": "edit",
  "path": "~/Library/Application Support/com.alou.desktop/agent-documents/{agent_id}/MEMORY.md",
  "old_text": "## 用户偏好\n（暂无记录）",
  "new_text": "## 用户偏好\n- 喜欢简洁的代码\n- 使用TypeScript"
}
```

## 最佳实践

### 1. 何时更新记忆

- ✅ 用户明确表达偏好时
- ✅ 完成重要任务后
- ✅ 学到新知识或技巧时
- ✅ 发现有效的工作流程时
- ❌ 不要记录临时信息
- ❌ 不要记录敏感信息

### 2. 记忆内容组织

- 使用清晰的Markdown结构
- 使用标题和列表组织信息
- 添加时间戳记录更新时间
- 保持内容简洁和相关

### 3. 更新频率

- 重要信息立即记录
- 定期整理和更新
- 删除过时信息

## 前端集成

### 初始化文档服务

在App.tsx中：
```typescript
import { useAgentDocuments } from '@/hooks/useAgentDocuments';

function App() {
  const { initializeDocuments } = useAgentDocuments();

  useEffect(() => {
    // 当创建新agent时初始化文档
    const agentId = 'agent_123';
    const agentInfo = { name: 'MyAgent', role_description: '...' };
    initializeDocuments(agentId, agentInfo);
  }, []);

  return <div>...</div>;
}
```

### 读取和更新文档

```typescript
import { useAgentDocuments } from '@/hooks/useAgentDocuments';

function AgentPanel() {
  const { getDocument, updateDocument } = useAgentDocuments();

  const loadMemory = async () => {
    const content = await getDocument('agent_123', 'memory');
    console.log('Memory:', content);
  };

  const saveMemory = async () => {
    const newContent = '# 长期记忆\n\n...';
    await updateDocument('agent_123', 'memory', newContent);
  };

  return <div>...</div>;
}
```

### 监听文档更新事件

```typescript
useEffect(() => {
  const handleDocumentUpdate = (event: CustomEvent) => {
    const { agentId, documentType, content, filePath } = event.detail;
    console.log(`文档已更新: ${documentType}`);
    // 更新UI或执行其他操作
  };

  window.addEventListener('agent-document-updated', handleDocumentUpdate as EventListener);

  return () => {
    window.removeEventListener('agent-document-updated', handleDocumentUpdate as EventListener);
  };
}, []);
```

## 系统提示词集成

系统提示词中已包含文档段落标记：

```
=== MEMORY ===
# 长期记忆
...

=== SOUL ===
# 核心身份
...
```

AI可以通过`agent_document`工具的`read`操作读取这些段落，通过`update`操作更新它们。

## 故障排除

### 问题1：文档不存在
**解决方案：** 调用`initializeAgentDocuments`初始化文档

### 问题2：无法读取文档
**解决方案：** 检查文件权限和路径是否正确

### 问题3：更新不生效
**解决方案：** 确保`agentDocumentService.initialize()`已被调用

## 未来改进

- [ ] 支持文档版本控制
- [ ] 支持文档搜索功能
- [ ] 支持文档导出和导入
- [ ] 支持文档加密
- [ ] 支持文档同步到云端
- [ ] 支持文档可视化编辑器

## 总结

记忆系统让AI智能体能够：
1. 跨会话保存重要信息
2. 学习和改进工作方式
3. 记住用户偏好和项目上下文
4. 与其他智能体共享知识

通过合理使用记忆系统，AI可以提供更个性化和智能的服务。
