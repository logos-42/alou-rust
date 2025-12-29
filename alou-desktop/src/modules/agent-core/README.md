# agent-core 模块

智能代理核心业务模块，提供代理、频道、消息、会话等核心功能。

## 功能特性

### 代理管理
- 创建、读取、更新、删除代理
- 代理配置管理
- 代理状态跟踪

### 频道管理
- 频道创建和删除
- 频道切换
- 频道搜索和过滤

### 消息管理
- 消息发送和接收
- 消息历史记录
- 消息流处理

### 会话管理
- 会话创建和关闭
- 会话状态管理
- 会话持久化

## 目录结构

```
agent-core/
├── entities/           # 实体类
│   ├── Agent.ts       # 代理实体
│   ├── Channel.ts     # 频道实体
│   ├── Message.ts     # 消息实体
│   └── Session.ts     # 会话实体
├── ports/             # 端口（接口）
│   ├── IAgentService.ts
│   ├── IChannelService.ts
│   ├── IMessageService.ts
│   └── ISessionService.ts
├── services/          # 服务实现
│   ├── AgentService.ts
│   ├── ChannelService.ts
│   ├── MessageService.ts
│   └── SessionService.ts
├── adapters/          # 适配器
│   └── LegacyAdapter.ts
├── hooks/             # React Hooks
│   ├── useAgentManagement.ts
│   ├── useChannelManagement.ts
│   ├── useMessageManagement.ts
│   └── useSessionManagement.ts
├── utils/             # 工具函数
│   ├── agentUtils.ts
│   └── channelUtils.ts
├── config/            # 配置
│   └── module.config.ts
├── tests/             # 测试
├── index.ts           # 模块入口
└── README.md          # 本文档
```

## 快速开始

### 安装和导入

```typescript
// 导入整个模块
import * as agentCore from '@modules/agent-core';

// 或按需导入
import { AgentService, useAgentManagement } from '@modules/agent-core';
```

### 基本使用

#### 使用服务
```typescript
import { AgentService } from '@modules/agent-core';

const agentService = new AgentService();

// 创建代理
const agent = await agentService.createAgent({
  name: '我的代理',
  model: 'gpt-4',
  temperature: 0.7,
  maxTokens: 2000,
});

// 获取代理列表
const agents = await agentService.listAgents();
```

#### 使用Hooks
```typescript
import { useAgentManagement, useMessageManagement } from '@modules/agent-core';

function MyComponent() {
  const { agents, createAgent, loading, error } = useAgentManagement();
  const { messages, sendMessage } = useMessageManagement('channel-id');
  
  const handleCreateAgent = async () => {
    const agent = await createAgent({
      name: '新代理',
      model: 'claude-3',
      temperature: 0.8,
    });
    console.log('代理创建成功:', agent);
  };
  
  return (
    <div>
      <button onClick={handleCreateAgent} disabled={loading}>
        创建代理
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
```

## API 参考

### 实体类

#### Agent
```typescript
interface Agent {
  id: string;
  name: string;
  description?: string;
  config: AgentConfig;
  status: AgentStatus;
  avatar?: string;
  createdAt: Date;
  updatedAt: Date;
}

type AgentStatus = 'idle' | 'active' | 'error' | 'offline' | 'initializing';
```

#### Channel
```typescript
interface Channel {
  id: string;
  name: string;
  agentId: string;
  status: 'online' | 'offline' | 'busy';
  statusLabel: string;
  icon: string;
  color: string;
  updatedAt: number;
  meta?: ChannelMeta;
}
```

### 服务接口

#### IAgentService
```typescript
interface IAgentService {
  createAgent(config: AgentConfig): Promise<Agent>;
  getAgent(id: string): Promise<Agent | null>;
  updateAgent(id: string, updates: Partial<Agent>): Promise<Agent>;
  deleteAgent(id: string): Promise<void>;
  listAgents(): Promise<Agent[]>;
}
```

#### IChannelService
```typescript
interface IChannelService {
  createChannel(name: string, agentId: string): Promise<Channel>;
  getChannel(id: string): Promise<Channel | null>;
  updateChannel(id: string, updates: Partial<Channel>): Promise<Channel>;
  deleteChannel(id: string): Promise<void>;
  listChannels(): Promise<Channel[]>;
  setActiveChannel(id: string): Promise<void>;
}
```

### Hooks

#### useAgentManagement
```typescript
function useAgentManagement(): {
  agents: Agent[];
  loading: boolean;
  error: string | null;
  createAgent: (config: AgentConfig) => Promise<Agent>;
  updateAgent: (id: string, updates: Partial<Agent>) => Promise<Agent>;
  deleteAgent: (id: string) => Promise<void>;
  refreshAgents: () => Promise<void>;
};
```

#### useChannelManagement
```typescript
function useChannelManagement(): {
  channels: Channel[];
  activeChannelId: string | null;
  loading: boolean;
  error: string | null;
  createChannel: (name: string, agentId: string) => Promise<Channel>;
  setActiveChannel: (id: string) => Promise<void>;
  refreshChannels: () => Promise<void>;
};
```

## 配置

### 模块配置
```typescript
// 默认配置
{
  apiBaseUrl: 'http://127.0.0.1:8787',
  enableCaching: true,
  maxRetries: 3,
}

// 更新配置
import { updateConfig } from '@modules/agent-core/config/module.config';

updateConfig({
  enableCaching: false,
  maxRetries: 5,
});
```

### 功能开关
```typescript
import { getFeatureFlag } from '@modules/agent-core/config/module.config';

// 检查功能是否启用
const enableAgentCreation = getFeatureFlag('agent', 'enableCreation');
const enableMessageDeletion = getFeatureFlag('message', 'enableDeletion');
```

## 迁移指南

### 从旧代码迁移

#### 使用LegacyAdapter
```typescript
import { LegacyAdapter } from '@modules/agent-core';

const adapter = new LegacyAdapter();

// 使用兼容接口
const agent = await adapter.createAgentLegacy(oldConfig);
const channels = await adapter.listChannelsLegacy();
```

#### 逐步迁移
1. **第一阶段**: 使用LegacyAdapter保持兼容
2. **第二阶段**: 逐步替换为新服务接口
3. **第三阶段**: 移除LegacyAdapter依赖

### 验证规则
```typescript
import { validateAgentName, validateMessageContent } from '@modules/agent-core/config/module.config';

// 验证代理名称
const nameValidation = validateAgentName('我的代理');
if (!nameValidation.valid) {
  console.error(nameValidation.error);
}

// 验证消息内容
const contentValidation = validateMessageContent('Hello, World!');
if (!contentValidation.valid) {
  console.error(contentValidation.error);
}
```

## 测试

### 运行测试
```bash
# 运行所有测试
npm test -- agent-core

# 运行特定测试
npm test -- agent-core/entities
npm test -- agent-core/services
```

### 测试示例
```typescript
// 实体测试
describe('Agent Entity', () => {
  it('should create agent with valid config', () => {
    const agent = new Agent({
      name: 'Test Agent',
      config: { model: 'gpt-4', temperature: 0.7, maxTokens: 2000 },
    });
    expect(agent.id).toBeDefined();
    expect(agent.status).toBe('idle');
  });
});

// 服务测试
describe('AgentService', () => {
  let service: AgentService;
  
  beforeEach(() => {
    service = new AgentService();
  });
  
  it('should create and retrieve agent', async () => {
    const agent = await service.createAgent(testConfig);
    const retrieved = await service.getAgent(agent.id);
    expect(retrieved).toEqual(agent);
  });
});
```

## 性能优化

### 缓存策略
- 代理数据缓存: 5分钟
- 频道数据缓存: 2分钟
- 消息数据缓存: 1分钟

### 请求优化
- 请求超时: 30秒
- 重试延迟: 1秒
- 最大并发: 5个请求

### 消息处理
- 批量大小: 50条消息
- 防抖时间: 300ms
- 节流时间: 100ms

## 错误处理

### 错误代码
```typescript
// 常见错误代码
AGENT_NOT_FOUND: 'AGENT_001'
CHANNEL_NOT_FOUND: 'CHANNEL_001'
MESSAGE_SEND_FAILED: 'MESSAGE_001'
SESSION_EXPIRED: 'SESSION_001'
NETWORK_ERROR: 'NETWORK_001'
VALIDATION_ERROR: 'VALIDATION_001'
```

### 错误处理示例
```typescript
import { errorConfig } from '@modules/agent-core/config/module.config';

try {
  const agent = await agentService.getAgent('non-existent-id');
} catch (error) {
  if (error.code === errorConfig.codes.AGENT_NOT_FOUND) {
    console.error(errorConfig.messages.agentNotFound);
    // 处理代理不存在的情况
  }
}
```

## 开发指南

### 添加新实体
1. 在 `entities/` 目录创建实体类
2. 定义类型接口和验证规则
3. 添加单元测试
4. 更新文档

### 添加新服务
1. 在 `ports/` 目录定义接口
2. 在 `services/` 目录实现服务
3. 添加集成测试
4. 更新API文档

### 添加新Hook
1. 在 `hooks/` 目录创建Hook
2. 基于现有服务实现业务逻辑
3. 添加Hook测试
4. 更新使用示例

## 贡献指南

1. **代码规范**: 遵循项目代码规范
2. **测试要求**: 新功能必须包含测试
3. **文档更新**: 更新相关文档
4. **类型安全**: 使用TypeScript确保类型安全

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../LICENSE) 文件。
