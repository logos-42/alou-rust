# useAgentMessages 重构计划

## 当前状态

### 问题
- **原始文件**: 1481 行，难以维护
- **单一职责**: 包含过多职责（状态管理、IPFS 持久化、智能体执行、群聊处理）
- **测试困难**: 无法独立测试各个功能模块

### ✅ 已完成
- ✅ 创建模块化目录结构
- ✅ 提取类型定义到 `types.ts`
- ✅ 提取工具函数到 `utils/`
- ✅ 创建 `useMessageState` Hook
- ✅ 创建 `useMessagePersistence` Hook
- ✅ 创建 `useAgentExecution` Hook
- ✅ 创建 `useGroupChatMessages` Hook
- ✅ 创建 `messageQueueService`
- ✅ 创建 `agentCommunicationService`
- ✅ 创建主入口 `index.ts`
- ✅ 将原文件重命名为 `useAgentMessages.legacy.ts`

## 重构步骤

### 第一阶段：基础架构 ✅
- [x] 创建目录结构
- [x] 提取类型定义
- [x] 提取工具函数
- [x] 完成 `useMessageState` Hook
- [x] 完成 `useMessagePersistence` Hook
- [x] 完成 `useAgentExecution` Hook
- [x] 完成 `useGroupChatMessages` Hook

### 第二阶段：服务层 ✅
- [x] 创建 `messageQueueService`
- [x] 创建 `agentCommunicationService`

### 第三阶段：集成测试（待办）
- [ ] 为每个 hook 编写单元测试
- [ ] 集成测试确保功能一致
- [ ] 性能测试确保无退化

### 第四阶段：清理（待办）
- [ ] 删除 `useAgentMessages.legacy.ts`
- [ ] 更新所有导入路径
- [ ] 更新文档

## 模块职责

```
useAgentMessages/
├── index.ts                      # 主入口（~50 行）✅
├── types.ts                      # 类型定义（~100 行）✅
├── useAgentMessages.legacy.ts    # 原文件（1481 行，待删除）
│
├── utils/
│   ├── index.ts                  # 工具导出 ✅
│   ├── progressUtils.ts          # 进度转换（~40 行）✅
│   └── messageUtils.ts           # 消息创建（~80 行）✅
│
├── hooks/
│   ├── index.ts                  # Hooks 导出 ✅
│   ├── useMessageState.ts        # 状态管理（~120 行）✅
│   ├── useMessagePersistence.ts  # IPFS 持久化（~150 行）✅
│   ├── useAgentExecution.ts      # 智能体执行（~250 行）✅
│   └── useGroupChatMessages.ts   # 群聊消息（~200 行）✅
│
└── services/
    ├── index.ts                  # 服务导出 ✅
    ├── messageQueueService.ts    # 消息队列（~100 行）✅
    └── agentCommunicationService.ts # 通信服务（~200 行）✅
```

## 预期收益

| 指标 | 重构前 | 重构后 | 改善 |
|------|--------|--------|------|
| 主文件大小 | 1481 行 | ~50 行 | 96% ↓ |
| 最大文件 | 1481 行 | ~250 行 | 83% ↓ |
| 模块数量 | 1 | 11 | 可维护性 ↑ |
| 测试覆盖率 | 低 | 高 | 可测试性 ↑ |
| 代码复用 | 低 | 高 | 复用性 ↑ |

## 迁移指南

### 导入路径变更

**重构前:**
```typescript
import { useAgentMessages } from './useAgentMessages'
```

**重构后:**
```typescript
// 主入口保持不变，向后兼容
import { useAgentMessages } from './useAgentMessages'

// 也可以单独导入子模块
import { useMessageState } from './useAgentMessages/hooks'
import { createMessage } from './useAgentMessages/utils'
import { createAgentCommunicationService } from './useAgentMessages/services'
```

### API 变更

大部分 API 保持不变，确保向后兼容。

新模块提供额外的灵活性：

```typescript
// 单独使用状态管理 Hook
const { messages, appendMessage, isAgentLoading } = useMessageState()

// 单独使用持久化 Hook
const { saveMessagesToIpfs, loadMessagesFromIpfs } = useMessagePersistence({ ... })

// 单独使用执行 Hook
const { sendMessageToAgent, cancelAgentExecution } = useAgentExecution({ ... })

// 单独使用群聊 Hook
const { handleGroupChatMessage, sendGroupChatMessage } = useGroupChatMessages({ ... })
```

## 进度追踪

- ✅ 2024-03-14: 创建模块结构，完成基础拆分
- ✅ 2024-03-14: 完成所有 Hooks 迁移
- ✅ 2024-03-14: 完成服务层迁移
- ⏳ 待定：完成测试并删除 legacy 文件

## 文件清单

### 类型定义
- `types.ts` - 所有 TypeScript 类型定义

### 工具函数
- `utils/progressUtils.ts` - 进度事件转文本
- `utils/messageUtils.ts` - 消息创建工具

### Hooks
- `hooks/useMessageState.ts` - 消息状态管理
- `hooks/useMessagePersistence.ts` - IPFS 持久化
- `hooks/useAgentExecution.ts` - 智能体执行
- `hooks/useGroupChatMessages.ts` - 群聊消息处理

### 服务
- `services/messageQueueService.ts` - 消息队列管理
- `services/agentCommunicationService.ts` - 智能体通信

## 参考

- [React Hooks 最佳实践](https://react.dev/learn/reusing-logic-with-custom-hooks)
- [TypeScript 模块组织](https://www.typescriptlang.org/docs/handbook/modules.html)
