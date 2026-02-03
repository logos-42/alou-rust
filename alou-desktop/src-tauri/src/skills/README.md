# Skills 系统

统一的技能定义、存储和执行系统，支持 AI 自动发现和使用技能。

## 架构

```
src/skills/
├── mod.rs           # 模块导出
├── manifest.rs      # 技能清单定义
├── storage.rs       # 技能存储管理
├── executor.rs      # 技能执行器接口
├── agent_skill.rs   # Agent 技能执行器
├── builtin.rs       # 内置技能执行器
├── prompt.rs        # Prompt 模板执行器
├── toolchain.rs     # 工具链执行器
└── script.rs        # 脚本执行器
```

## 技能类型

1. **Builtin** - 内置 Rust 实现
2. **PromptTemplate** - Prompt 模板
3. **ToolChain** - 工具链组合
4. **AgentSkill** - AI Agent 技能
5. **Script** - 脚本代码

## AI 使用接口

### 1. 搜索技能

```json
{
  "action": "search_skills",
  "query": "text summarization",
  "category": "text_processing"
}
```

### 2. 执行技能

```json
{
  "action": "execute_skill",
  "skill_id": "skill_text_summarizer",
  "inputs": {
    "text": "需要摘要的长文本...",
    "max_length": 200
  }
}
```

### 3. 创建 Agent Skill

```json
{
  "action": "create_agent_skill",
  "display_name": "Code Reviewer",
  "description": "Reviews code for issues",
  "persona": "You are a code review expert...",
  "capabilities": ["find bugs", "suggest improvements"],
  "constraints": ["be concise", "focus on security"]
}
```

## 内置技能

- `skill_text_summarizer` - 文本摘要
- `skill_code_formatter` - 代码格式化
- `skill_data_validator` - 数据验证
- `skill_file_analyzer` - 文件分析

## AI 决策集成

AI 现在可以在决策时使用技能：

```
决策选项：
1. COMPLETED - 任务完成
2. RETRY:<原因> - 重试
3. SKILL:<skill_id>:<JSON输入> - 调用技能
4. RESEARCH:<查询> - 调研
5. CONTINUE - 继续
```

示例：
```
SKILL:skill_text_summarizer:{"text":"长文本内容","max_length":300}
```
