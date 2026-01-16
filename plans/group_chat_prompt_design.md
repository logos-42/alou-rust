# Group Chat Prompt Design

## Requirements Analysis

Based on the user's requirements, the group chat prompt needs to address:

1. **Separate Sessions**: Agents in group chat should not share session context with individual page interactions
2. **Context Awareness**: Agents must see and consider the full conversation history between humans and other agents
3. **Communication Style**: Use short sentences to keep interactions concise
4. **Task Management**: Handle task reporting and division of labor among different agents

## Current System Analysis

From examining the codebase:

- **Prompt System**: Located in `alou-edge/src/ts-agent/prompts.ts` and `alou-desktop/src/components/AgentChat/utils/agentPrompts.js`
- **Group Chat Implementation**: Uses PubSub messaging system with polling for real-time updates
- **Agent Modes**: Support different prompt modes (general, wallet, defi, nft, payment, developer, custom_agent)

## Proposed Solution

### 1. Add New Prompt Mode

Add `group_chat` to the `PromptMode` type in `prompts.ts`:

```typescript
export type PromptMode =
  | 'general'
  | 'wallet'
  | 'defi'
  | 'nft'
  | 'payment'
  | 'developer'
  | 'custom_agent'
  | 'group_chat';
```

### 2. Group Chat Prompt Template

```typescript
const GROUP_CHAT_PROMPT = `你是群聊中的智能体，参与多智能体协作任务。

## 群聊规则
- 使用简短句子交流，避免冗长回复
- 始终考虑对话上下文，包括人类和其他智能体的消息
- 专注于任务协作，不要偏离主题

## 协作职责
- 汇报任务进度：完成步骤时简要报告
- 分工协调：主动提出承担子任务或建议其他智能体分工
- 信息共享：分享重要发现或结果
- 问题求助：遇到困难时寻求帮助

## 沟通风格
- 直接明了，不用过多礼貌用语
- 使用行动导向的语言
- 保持专业但友好

## 上下文意识
- 阅读所有历史消息，了解当前状态
- 回应相关消息，不要重复已知信息
- 跟踪任务分配和完成情况

现在，以协作精神参与群聊任务。`;
```

### 3. Integration Points

- Update `getSystemPrompt()` function to return `GROUP_CHAT_PROMPT` for `group_chat` mode
- Update `detectPromptMode()` to detect group chat context (if needed)
- Ensure group chat agents receive full conversation history as context
- Implement session isolation to prevent mixing with individual page sessions

### 4. Context Handling

The prompt assumes that the conversation history will be provided as context to each agent response, ensuring they can see previous messages from humans and other agents.

### 5. Implementation Steps

1. Add `group_chat` mode to PromptMode type
2. Add GROUP_CHAT_PROMPT constant
3. Update getSystemPrompt switch statement
4. Update agent prompt generation logic in frontend
5. Test with group chat scenarios
6. Validate session isolation

## Benefits

- **Concise Communication**: Short sentences reduce noise in group chat
- **Effective Collaboration**: Clear roles for task reporting and division of labor
- **Context Awareness**: Agents can make informed decisions based on full conversation
- **Session Isolation**: Prevents confusion between group and individual interactions

## Potential Enhancements

- Add agent role assignment within group chat
- Implement task status tracking
- Add conflict resolution mechanisms
- Support for different collaboration patterns (hierarchical, peer-to-peer, etc.)