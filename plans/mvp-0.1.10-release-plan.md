# Alou MVP 0.1.10 版本发布计划

## 版本目标
在三天内推出MVP 0.1.10版本，实现高级群聊功能和CLI完整交互。

## 核心功能需求

### 1. 群聊功能（高级）
- ✅ 基础功能：创建群聊、发送消息、接收消息
- ✅ 人类和AI都能参与
- ✅ AI自主交流、多Agent协作
- ✅ 消息存档
- ✅ 任务分配、进度汇报、Agent间协调

### 2. PubSub/IPFS
- 保留IPFS作为主要通信方案
- 实现iroh作为降级方案

### 3. 桌面端优化
- 修复群聊按钮打不开问题
- 完善存档功能
- AI操作和人类操作更流畅

### 4. CLI增强
- Agent可以在CLI中自主行动
- 执行任务、报告进度

### 5. 配置完善
- 本地API配置
- 项目配置完善

---

## 详细实施步骤

### 阶段一：桌面端群聊按钮修复

#### 1.1 定位群聊按钮问题
- 检查AgentSidebarLeft中的群聊按钮绑定事件
- 检查DiapGroupChatManager的打开逻辑
- 验证状态管理中的群聊面板开关

#### 1.2 修复按钮事件
```
文件位置:
- alou-desktop/src/components/agent/AgentSidebarLeft.jsx
- alou-desktop/src/components/agent/DiapGroupChatManager.jsx
- alou-desktop/src/components/agent/GroupChatPanel.jsx
```

---

### 阶段二：群聊核心功能完善

#### 2.1 群聊创建流程
- 前端：GroupChatPanel创建群聊表单
- 服务：localIpfsGroupChatService.createGroup()
- 存储：保存到KV和内存
- PubSub：发布群聊创建消息

#### 2.2 群聊加入流程
- 支持通过群聊ID或邀请链接加入
- 订阅群聊主题
- 接收历史消息

#### 2.3 消息发送和接收
- 使用IPFS PubSub发送消息
- 轮询接收消息
- 本地缓存消息历史

#### 2.4 消息存档
- 实现GroupChatArchiveList组件
- 保存到本地存储/IndexedDB
- 支持分页加载历史消息

---

### 阶段三：AI Agent群聊协作

#### 3.1 Agent身份识别
- 为每个Agent分配唯一DID
- 在消息中包含Agent标识
- 区分人类消息和AI消息

#### 3.2 任务分配系统
- 群聊中支持@指定Agent
- 任务消息类型定义
- 任务状态跟踪

#### 3.3 进度汇报
- Agent定期汇报任务进度
- 消息类型：TASK_PROGRESS
- 进度可视化显示

#### 3.4 Agent间协调
- 实现群聊协调器
- 避免重复任务
- 任务冲突解决

---

### 阶段四：PubSub服务优化

#### 4.1 IPFS PubSub实现
- 完善ipfs_pubsub_publish命令
- 完善ipfs_pubsub_subscribe_once命令
- 错误处理和重试机制

#### 4.2 iroh降级方案
- 检测IPFS可用性
- 自动切换到iroh
- iroh pubsub实现

#### 4.3 后端API降级
- Workers端pubsub API
- 消息队列实现
- 离线消息存储

---

### 阶段五：桌面端操作优化

#### 5.1 交互流畅度
- 减少不必要的重渲染
- 优化状态更新逻辑
- 添加加载状态指示

#### 5.2 存档功能
- 实现消息持久化
- 群聊历史保存
- 导出聊天记录

#### 5.3 UI/UX改进
- 优化群聊面板动画
- 改善滚动体验
- 添加快捷键支持

---

### 阶段六：CLI增强

#### 6.1 架构改进
```
alou-cli/src/
├── main.rs           # 入口
├── agent.rs           # Agent交互逻辑
├── config.rs          # 配置管理
└── api.rs             # API客户端
```

#### 6.2 Agent对话功能
- 连接本地/远程AI API
- 流式响应处理
- 多轮对话支持

#### 6.3 自主行动能力
- 任务队列管理
- 定时任务执行
- 进度报告

#### 6.4 群聊支持
- 订阅群聊主题
- 发送消息到群聊
- 接收群聊消息

---

### 阶段七：配置完善

#### 7.1 本地API配置
```json
// alou-desktop/config/local-api-config.json
{
  "apiEndpoint": "http://localhost:8787",
  "aiProvider": "deepseek",
  "apiKey": "本地配置的API密钥"
}
```

#### 7.2 环境变量配置
- VITE_API_BASE_URL
- VITE_IPFS_API_URL
- VITE_DEEPSEEK_API_KEY

#### 7.3 Agent配置
- 系统提示词
- 工具权限
- 响应参数

---

### 阶段八：版本发布

#### 8.1 版本更新
- package.json: 0.1.9 → 0.1.10
- 更新CHANGELOG

#### 8.2 Git提交
```bash
git add .
git commit -m "feat: MVP 0.1.10 - 群聊功能完善、CLI增强"
git push origin main
```

---

## 技术架构图

```mermaid
graph TB
    subgraph 桌面端
        UI[React UI组件]
        Hook[React Hooks]
        Service[业务服务层]
    end
    
    subgraph 通信层
        PubSub[PubSub服务]
        IPFS[IPFS节点]
        iroh[iroh降级]
        Workers[Cloudflare Workers]
    end
    
    subgraph CLI
        CLI[alou-cli]
        AgentCLI[Agent交互模块]
    end
    
    UI --> Hook
    Hook --> Service
    Service --> PubSub
    PubSub --> IPFS
    PubSub --> iroh
    PubSub --> Workers
    CLI --> AgentCLI
    AgentCLI --> Service
```

---

## 文件修改清单

### 需要修改的核心文件
1. `alou-desktop/package.json` - 版本升级
2. `alou-desktop/src/components/agent/AgentSidebarLeft.jsx` - 修复按钮
3. `alou-desktop/src/components/agent/GroupChatPanel.jsx` - 完善功能
4. `alou-desktop/src/services/pubsubService.ts` - iroh降级
5. `alou-desktop/src/services/localIpfsGroupChatService.ts` - 完善群聊
6. `alou-cli/src/main.rs` - CLI增强
7. `alou-desktop/config/local-api-config.json` - 新增配置文件

### 需要创建的新文件
1. `alou-desktop/src/services/irohPubsubService.ts` - iroh服务
2. `alou-cli/src/agent.rs` - Agent交互
3. `alou-cli/src/config.rs` - 配置管理
4. `plans/mvp-0.1.10-release-plan.md` - 本计划文档

---

## 实施优先级

### P0 - 必须完成
1. 修复群聊按钮
2. 群聊基础功能
3. 版本发布

### P1 - 重要
4. AI Agent协作
5. 消息存档
6. PubSub优化

### P2 - 增强
7. CLI增强
8. 配置完善
9. UI优化

---

## 预期成果

完成0.1.10版本后，将具备：
- ✅ 完整可用的群聊功能
- ✅ 人类和AI Agent都能参与群聊
- ✅ Agent间任务分配和进度汇报
- ✅ 群聊消息存档
- ✅ CLI中Agent可自主行动
- ✅ 稳定的pubsub通信（IPFS + iroh降级）
- ✅ 流畅的桌面端操作体验
