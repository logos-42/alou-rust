# AgentChat.jsx 迁移计划

## 迁移目标
将 `AgentChat.jsx` (881行) 重构为：
1. **精简版UI组件** (`@ui/views/AgentChat.jsx`, <300行)
2. **业务逻辑模块** (`@modules/agent-core/`)
3. **保持功能完全一致**

## 迁移原则
1. **渐进式迁移**：每次只迁移一个小功能
2. **向后兼容**：新旧代码并存，通过适配器连接
3. **测试保障**：每个步骤都有测试验证
4. **可回滚**：每个改动都可独立回滚

## 迁移步骤

### 步骤1: 创建实体类 (第1天)
**目标**: 定义核心数据模型

#### 1.1 创建Agent实体
```typescript
// @modules/agent-core/entities/Agent.ts
export interface Agent {
  id: string;
  name: string;
  description?: string;
  config: AgentConfig;
  status: AgentStatus;
  avatar?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface AgentConfig {
  model: string;
  temperature: number;
  maxTokens: number;
  systemPrompt?: string;
}

export type AgentStatus = 'idle' | 'active' | 'error' | 'offline' | 'initializing';
```

#### 1.2 创建Channel实体
```typescript
// @modules/agent-core/entities/Channel.ts
export interface Channel {
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

export interface ChannelMeta {
  mode?: string;
  // 其他元数据
}
```

#### 1.3 创建Message实体
```typescript
// @modules/agent-core/entities/Message.ts
export interface Message {
  id: string;
  channelId: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  source?: string;
  metadata?: Record<string, any>;
}
```

#### 1.4 创建Session实体
```typescript
// @modules/agent-core/entities/Session.ts
export interface Session {
  id: string;
  userId: string;
  agentId?: string;
  channelId?: string;
  status: 'active' | 'closed' | 'archived';
  createdAt: Date;
  updatedAt: Date;
  metadata?: Record<string, any>;
}
```

### 步骤2: 创建业务服务 (第2-3天)
**目标**: 实现核心业务逻辑

#### 2.1 创建AgentService
```typescript
// @modules/agent-core/services/AgentService.ts
export interface IAgentService {
  createAgent(config: AgentConfig): Promise<Agent>;
  getAgent(id: string): Promise<Agent | null>;
  updateAgent(id: string, updates: Partial<Agent>): Promise<Agent>;
  deleteAgent(id: string): Promise<void>;
  listAgents(): Promise<Agent[]>;
}

export class AgentService implements IAgentService {
  // 实现具体逻辑
}
```

#### 2.2 创建ChannelService
```typescript
// @modules/agent-core/services/ChannelService.ts
export interface IChannelService {
  createChannel(name: string, agentId: string): Promise<Channel>;
  getChannel(id: string): Promise<Channel | null>;
  updateChannel(id: string, updates: Partial<Channel>): Promise<Channel>;
  deleteChannel(id: string): Promise<void>;
  listChannels(): Promise<Channel[]>;
  setActiveChannel(id: string): Promise<void>;
}
```

#### 2.3 创建MessageService
```typescript
// @modules/agent-core/services/MessageService.ts
export interface IMessageService {
  sendMessage(channelId: string, content: string, type: MessageType): Promise<Message>;
  getMessages(channelId: string, limit?: number): Promise<Message[]>;
  deleteMessage(messageId: string): Promise<void>;
  clearMessages(channelId: string): Promise<void>;
}
```

#### 2.4 创建SessionService
```typescript
// @modules/agent-core/services/SessionService.ts
export interface ISessionService {
  createSession(userId: string): Promise<Session>;
  getSession(id: string): Promise<Session | null>;
  updateSession(id: string, updates: Partial<Session>): Promise<Session>;
  closeSession(id: string): Promise<void>;
}
```

### 步骤3: 创建适配器 (第1天)
**目标**: 实现向后兼容

#### 3.1 创建LegacyAdapter
```typescript
// @modules/agent-core/adapters/LegacyAdapter.ts
export class LegacyAdapter {
  private agentService: IAgentService;
  private channelService: IChannelService;
  private messageService: IMessageService;
  private sessionService: ISessionService;
  
  constructor() {
    // 初始化服务
  }
  
  // 提供与原有代码兼容的接口
  async createAgentLegacy(config: any): Promise<any> {
    // 转换参数，调用新服务
    const agentConfig: AgentConfig = this.convertLegacyConfig(config);
    return this.agentService.createAgent(agentConfig);
  }
  
  // 其他兼容方法...
}
```

### 步骤4: 创建自定义Hooks (第3-4天)
**目标**: 基于业务服务创建React hooks

#### 4.1 创建useAgentManagement
```typescript
// @modules/agent-core/hooks/useAgentManagement.ts
export function useAgentManagement() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const agentService = useAgentService();
  
  const createAgent = async (config: AgentConfig) => {
    // 实现创建代理逻辑
  };
  
  // 其他方法...
  
  return {
    agents,
    loading,
    error,
    createAgent,
    // 其他返回值...
  };
}
```

#### 4.2 创建useChannelManagement
```typescript
// @modules/agent-core/hooks/useChannelManagement.ts
export function useChannelManagement() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [activeChannelId, setActiveChannelId] = useState<string | null>(null);
  
  const channelService = useChannelService();
  
  // 实现频道管理逻辑
}
```

#### 4.3 创建useMessageManagement
```typescript
// @modules/agent-core/hooks/useMessageManagement.ts
export function useMessageManagement(channelId: string) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [sending, setSending] = useState(false);
  
  const messageService = useMessageService();
  
  const sendMessage = async (content: string, type: MessageType = 'user') => {
    // 实现发送消息逻辑
  };
}
```

#### 4.4 创建useSessionManagement
```typescript
// @modules/agent-core/hooks/useSessionManagement.ts
export function useSessionManagement() {
  const [session, setSession] = useState<Session | null>(null);
  
  const sessionService = useSessionService();
  
  // 实现会话管理逻辑
}
```

### 步骤5: 重构UI组件 (第2-3天)
**目标**: 创建精简版AgentChat组件

#### 5.1 创建新的UI组件
```jsx
// @ui/views/AgentChat.jsx
import React from 'react';
import { useAgentManagement } from '@modules/agent-core/hooks/useAgentManagement';
import { useChannelManagement } from '@modules/agent-core/hooks/useChannelManagement';
import { useMessageManagement } from '@modules/agent-core/hooks/useMessageManagement';

// 导入UI组件
import ChatHeader from '@ui/components/ChatHeader';
import AgentSidebarLeft from '@ui/components/AgentSidebarLeft';
import AgentSidebarRight from '@ui/components/AgentSidebarRight';
import AgentCanvas from '@ui/components/AgentCanvas';

const AgentChat = () => {
  // 使用新的hooks
  const { agents, createAgent } = useAgentManagement();
  const { channels, activeChannelId, setActiveChannel } = useChannelManagement();
  const { messages, sendMessage } = useMessageManagement(activeChannelId);
  
  // 精简的状态管理
  const [uiState, setUiState] = useState({
    isSidebarCollapsed: false,
    isDarkMode: false,
    // 其他UI状态...
  });
  
  // 渲染逻辑
  return (
    <div className="agent-chat">
      <ChatHeader />
      <div className="main-content">
        <AgentSidebarLeft
          channels={channels}
          activeChannelId={activeChannelId}
          onSelectChannel={setActiveChannel}
        />
        <AgentCanvas
          messages={messages}
          onSendMessage={sendMessage}
        />
        <AgentSidebarRight
          agents={agents}
          onCreateAgent={createAgent}
        />
      </div>
    </div>
  );
};

export default AgentChat;
```

#### 5.2 迁移UI组件
1. 将现有UI组件复制到 `@ui/components/`
2. 移除业务逻辑，只保留UI渲染
3. 更新导入路径
4. 确保样式正常工作

### 步骤6: 测试和验证 (第1-2天)
**目标**: 确保功能一致

#### 6.1 单元测试
```typescript
// 测试业务服务
describe('AgentService', () => {
  it('should create agent', async () => {
    const service = new AgentService();
    const agent = await service.createAgent(testConfig);
    expect(agent.id).toBeDefined();
    expect(agent.name).toBe(testConfig.name);
  });
});
```

#### 6.2 集成测试
```typescript
// 测试hooks集成
describe('useAgentManagement', () => {
  it('should manage agents state', async () => {
    const { result } = renderHook(() => useAgentManagement());
    await act(async () => {
      await result.current.createAgent(testConfig);
    });
    expect(result.current.agents).toHaveLength(1);
  });
});
```

#### 6.3 端到端测试
```typescript
// 测试完整流程
describe('AgentChat flow', () => {
  it('should create agent and send message', async () => {
    // 模拟用户操作
    await user.click(createAgentButton);
    await user.type(agentNameInput, 'Test Agent');
    await user.click(saveButton);
    
    // 验证结果
    expect(screen.getByText('Test Agent')).toBeInTheDocument();
  });
});
```

## 风险控制

### 技术风险
1. **状态同步问题**：新旧代码状态可能不同步
2. **性能影响**：新架构可能影响性能
3. **依赖冲突**：新旧依赖可能冲突

### 应对措施
1. **状态同步机制**：实现状态同步监听
2. **性能监控**：监控关键性能指标
3. **依赖隔离**：使用适配器隔离依赖

### 回滚计划
1. **每个步骤可独立回滚**
2. **保留完整的迁移日志**
3. **定期创建代码快照**

## 成功标准

### 功能标准
- [ ] 所有现有功能正常工作
- [ ] 用户界面保持一致
- [ ] 性能无退化
- [ ] 错误处理正常

### 代码标准
- [ ] AgentChat.jsx < 300行
- [ ] 业务逻辑完全迁移到modules
- [ ] UI逻辑完全迁移到ui
- [ ] 类型定义完整

### 测试标准
- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过
- [ ] 端到端测试通过

## 时间安排

### 第1周
- 周一：创建实体类
- 周二：创建业务服务
- 周三：创建适配器
- 周四：创建基础hooks
- 周五：测试和调试

### 第2周
- 周一：完善hooks
- 周二：开始UI重构
- 周三：完成UI重构
- 周四：集成测试
- 周五：性能优化

## 交付物

### 代码交付
1. `@modules/agent-core/` - 完整业务模块
2. `@ui/views/AgentChat.jsx` - 精简版UI组件
3. `@ui/components/` - 纯UI组件
4. 测试套件

### 文档交付
1. 架构设计文档
2. API接口文档
3. 迁移指南
4. 开发规范

## 下一步行动

1. **开始实体类创建**
2. **建立测试环境**
3. **逐步迁移功能**
4. **持续测试验证**

## 相关文档
- [AgentChat分析报告](./AGENTCHAT_ANALYSIS.md)
- [架构演进计划](../docs/REFACTORING_PLAN.md)
- [构建架构说明](./BUILD_ARCHITECTURE.md)
