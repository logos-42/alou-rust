# AgentChat 组件结构

这个文件夹包含了从大型单体组件 `AgentChat.jsx` 拆分出来的功能模块。

## 文件结构

- `agentUtils.js` - 工具函数，包括：
  - `resolveAgentAvatar` - 解析智能体头像
  - `buildChannelFromAgent` - 从智能体数据构建频道对象
  - `extractAgentTarget` - 提取智能体目标标识
  - `extractErrorMessage` - 提取错误信息
  - `computeAgentProfile` - 计算智能体配置文件

- `agentConstants.js` - 常量定义

- `useChannelManager.js` - 频道管理逻辑的 Hook，封装了：
  - 频道列表加载
  - 频道搜索
  - 频道选择
  - 智能体解析

- `AgentChatContext.jsx` - Context 定义和 Provider 组件（可选，用于未来扩展）

- `AgentChatProvider.jsx` - Context Provider 组件（可选，用于未来扩展）

- `index.css` - 样式文件（从 `AgentChat.css` 移动）

- `index.js` - 导出文件

## 使用方式

主组件 `AgentChat.jsx` 现在导入这些工具函数和 hooks：

```javascript
import { resolveAgentAvatar, buildChannelFromAgent, extractAgentTarget, extractErrorMessage, computeAgentProfile } from './AgentChat/agentUtils'
import { useChannelManager } from './AgentChat/useChannelManager'
```

## 未来扩展

可以根据需要进一步拆分：
- 钱包管理逻辑可以提取到 `useWalletManager.js`
- 消息处理逻辑可以提取到 `useMessageHandler.js`
- UI 交互逻辑可以提取到 `useUIController.js`

