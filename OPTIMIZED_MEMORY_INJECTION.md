# 优化的记忆注入策略 - 初次全注入，后续只注入记忆

## 优化目标

**节省 Token** - 避免每次对话都注入所有 7 个文档，减少 token 使用量

## 策略说明

### 初次激活（切换智能体/首次对话）
- ✅ **注入所有 7 个文档**
- 建立完整的上下文
- AI 了解自己的身份、能力、约束等
- 缓存到 `systemPromptCache`

### 后续对话
- ✅ **只注入 MEMORY.md（长期记忆）**
- 节省 token
- AI 需要其他文档时使用 `agent_document` 工具读取

## 修改内容

### 1. `agentPrompts.ts` - 添加 `injectAll` 参数

```typescript
export const getSystemPromptForAgent = async (
  agentInfo: Agent | null,
  mode: 'agent' | 'alou' | 'group_chat',
  walletAddress: string | null,
  chain: string | null,
  injectAll: boolean = false  // 新增参数
): Promise<string> => {
  // ...
  
  // 根据 injectAll 参数决定注入范围
  if (injectAll) {
    // 初次激活：注入所有 7 个文档
    basePrompt += `
=== 你的长期记忆 ===
${documentContents.memory}
=== 核心身份 ===
${documentContents.soul}
... (所有文档)
`;
    console.log('[getSystemPromptForAgent] 初次激活：注入所有 7 个文档，长度:', basePrompt.length);
  } else {
    // 后续对话：只注入记忆
    basePrompt += `
=== 你的长期记忆 ===
${documentContents.memory}
`;
    console.log('[getSystemPromptForAgent] 后续对话：只注入记忆，长度:', basePrompt.length);
  }
}
```

### 2. `useAgentMessages.ts` - 使用 `injectAll` 参数

```typescript
// 预加载（初次激活）
const prefetchSystemPrompt = async () => {
  const prompt = await getSystemPromptForAgent(
    selectedAgent, 
    currentMode, 
    walletAddress, 
    activeChain ?? null, 
    true  // injectAll=true，注入所有文档
  )
  setSystemPromptCache(prev => ({ ...prev, [activeChannelId]: prompt }))
}

// 发送消息时（后续对话）
if (!systemPrompt || systemPrompt.length === 0) {
  systemPrompt = await getSystemPromptForAgent(
    agentInfo, 
    currentMode, 
    walletAddress, 
    activeChain ?? null, 
    false  // injectAll=false，只注入记忆
  )
}
```

## Token 节省对比

### 优化前（每次都注入所有文档）

| 场景 | Token 数量 |
|------|-----------|
| 初次对话 | ~5000-8000 tokens（7 个文档） |
| 后续对话 | ~5000-8000 tokens（7 个文档） |
| 再后续对话 | ~5000-8000 tokens（7 个文档） |

### 优化后（初次全注入，后续只注入记忆）

| 场景 | Token 数量 | 节省 |
|------|-----------|------|
| 初次对话 | ~5000-8000 tokens（7 个文档） | - |
| 后续对话 | ~500-1000 tokens（只记忆） | **~80-90%** |
| 再后续对话 | ~500-1000 tokens（只记忆） | **~80-90%** |

## 工作流程

```
用户选择智能体
    ↓
useAgentMessages.useEffect 触发
    ↓
调用 getSystemPromptForAgent(..., injectAll=true)
    ↓
注入所有 7 个文档（MEMORY, SOUL, IDENTITY, ...）
    ↓
缓存到 systemPromptCache[activeChannelId]
    ↓
控制台："初次激活：注入所有 7 个文档，长度：5800"
    ↓
═══════════════════════════════════
    ↓
用户发送第一条消息
    ↓
使用缓存的系统提示词（包含所有文档）✅
    ↓
AI 回复（基于完整上下文）
    ↓
═══════════════════════════════════
    ↓
用户发送第二条消息
    ↓
使用缓存的系统提示词（包含所有文档）✅
    ↓
AI 回复
    ↓
═══════════════════════════════════
    ↓
如果需要刷新上下文（可选）
    ↓
调用 getSystemPromptForAgent(..., injectAll=false)
    ↓
只注入 MEMORY.md（长期记忆）
    ↓
控制台："后续对话：只注入记忆，长度：800"
```

## AI 如何访问其他文档

如果 AI 需要读取其他文档（如 SOUL、CAPABILITIES 等），可以使用工具：

```json
{
  "action": "read",
  "document_type": "soul"
}
```

或者：

```json
{
  "action": "read",
  "document_type": "capabilities"
}
```

## 控制台日志示例

```
// 初次激活
[useChannelManager] 从文件加载智能体 agent_xxx 的文档：['memory', 'soul', ...]
[useAgentMessages] 预加载系统提示词（初次激活，注入所有文档）...
[getSystemPromptForAgent] 加载文档内容：['memory', 'soul', 'identity', ...]
[getSystemPromptForAgent] 初次激活：注入所有 7 个文档，长度：5800
[useAgentMessages] 系统提示词预加载完成，长度：5800

// 第一次对话
[useAgentMessages] 使用缓存的系统提示词（长度：5800）
[useAgentMessages] 调用本地 AI，provider: xxx, model: xxx, 消息数：3

// 第二次对话
[useAgentMessages] 使用缓存的系统提示词（长度：5800）
[useAgentMessages] 调用本地 AI，provider: xxx, model: xxx, 消息数：5

// 如果需要刷新（可选）
[useAgentMessages] 缓存未命中，动态生成系统提示词（只注入记忆）
[getSystemPromptForAgent] 后续对话：只注入记忆，长度：800
```

## 优势

✅ **节省 Token** - 后续对话节省 80-90% 的 token  
✅ **保持上下文** - 初次激活时 AI 仍有完整上下文  
✅ **按需读取** - AI 可以用工具读取特定文档  
✅ **性能优化** - 减少每次请求的数据量  
✅ **成本降低** - 长期使用显著降低 API 成本  

## 注意事项

1. **初次注入很重要** - 确保 AI 在第一次对话时了解完整信息
2. **缓存管理** - 如果智能体文档更新，需要清除缓存
3. **AI 工具使用** - AI 需要知道可以使用 `agent_document` 工具读取文档

## 相关文件

- `/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/utils/agentPrompts.ts`
- `/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/useAgentMessages.ts`
- `/Users/apple/Downloads/alou/AUTO_MEMORY_INJECTION_COMPLETE.md` - 之前的完整注入方案
