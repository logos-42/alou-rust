# JS 到 TypeScript 转换计划

## 项目概述

将 alou-desktop 项目的 JavaScript 文件逐步转换为 TypeScript 文件。

## 当前状态

### 已转换的文件 (4个)
| 文件 | 路径 |
|------|------|
| api.ts | src/services/api.ts |
| authService.ts | src/services/authService.ts |
| agentAssetsService.ts | src/services/agentAssetsService.ts |
| skillsService.ts | src/services/skillsService.ts |

### 已部分转换 (stores)
| 文件 | 路径 |
|------|------|
| authStore.ts | src/stores/authStore.ts |

---

## 待转换文件清单

### 第一优先级：核心服务层 (核心依赖)
这些文件被其他服务广泛引用，需优先转换：

1. **userService.js** - 用户基础服务，其他服务依赖
2. **walletService.js** - 钱包服务，核心业务
3. **authService.js** - 已部分转换，需完成
4. **blockchainService.js** - 区块链交互
5. **ipfsService.js** - IPFS 核心服务

### 第二优先级：业务服务层
6. **agentService.js** - Agent 核心服务
7. **agentCoordinatorService.js** - Agent 协调服务
8. **agentResolverService.js** - Agent 解析服务
9. **subscriptionService.js** - 订阅服务
10. **workflowService.js** - 工作流服务
11. **workflowApi.js** - 工作流 API
12. **specService.js** - 规范服务

### 第三优先级：工具服务层
13. **clusterActionService.js** - 集群操作服务
14. **toolService.js** - 工具服务
15. **sdkToolsService.js** - SDK 工具服务
16. **sdkAgentTools.js** - SDK Agent 工具
17. **mcpUiService.js** - MCP UI 服务
18. **asyncTaskService.js** - 异步任务服务
19. **promptService.js** - 提示词服务
20. **skillGenerator.js** - 技能生成器
21. **agentSkillsStorage.js** - Agent 技能存储

### 第四优先级：IPFS 相关
22. **ipfsContentService.js** - IPFS 内容服务
23. **localIpfsGroupChatService.js** - 本地 IPFS 群聊服务
24. **pubsubService.js** - 发布订阅服务
25. **lspService.js** - LSP 服务
26. **imageProxyService.js** - 图片代理服务
27. **didDocumentParser.js** - DID 文档解析器
28. **gatewayUtils.js** - 网关工具
29. **fallbackStrategy.js** - 回退策略
30. **ipnsUtils.js** - IPNS 工具

### 第五优先级：钱包与 DIAP
31. **desktopWalletService.js** - 桌面钱包服务
32. **walletSyncService.js** - 钱包同步服务
33. **diapService.js** - DIAP 服务
34. **diapIntegrationService.js** - DIAP 集成服务

### 第六优先级：Stores
35. **agentStore.js** - Agent 状态存储
36. **clusterActionStore.js** - 集群操作状态
37. **memoryStore.js** - 内存存储

### 第七优先级：Utils
38. **storageAdapter.js** - 存储适配器
39. **storageMonitor.js** - 存储监控
40. **diapIdentityManager.js** - DIAP 身份管理
41. **diapTestHelper.js** - DIAP 测试助手
42. **diapIdentityCleanupTool.js** - DIAP 身份清理工具
43. **agentCreationDiagnosticTool.js** - Agent 创建诊断工具
44. **avatarProtectionTool.js** - 头像保护工具
45. **imageBlur.js** - 图片模糊工具
46. **memoryStorage.js** - 内存存储
47. **rustMemoryStore.js** - Rust 内存存储
48. **rustMemoryExample.js** - Rust 内存示例

### 第八优先级：i18n
49. **index.js** - i18n 入口
50. **namespaces/agent.js** - Agent 命名空间
51. **namespaces/apiConfig.js** - API 配置命名空间
52. **namespaces/common.js** - 通用命名空间
53. **namespaces/legacy.js** - 遗留命名空间
54. **namespaces/login.js** - 登录命名空间
55. **namespaces/subscription.js** - 订阅命名空间
56. **namespaces/wallet.js** - 钱包命名空间

---

## 转换策略

### 转换步骤
1. **创建 .d.ts 类型声明文件** - 定义共享类型
2. **重命名文件** - 将 `.js` 重命名为 `.ts`
3. **添加类型注解** - 为函数参数、返回值添加类型
4. **更新 import/export** - 确保类型正确导入导出
5. **处理 any 类型** - 逐步替换为具体类型
6. **验证编译** - 运行 `npx tsc` 验证无错误

### 命名规范
- 文件名使用 camelCase: `myService.ts`
- 类型定义使用 PascalCase: `interface UserResponse`
- 常量使用 UPPER_SNAKE_CASE

### 类型定义位置
- **共享类型** - `src/shared/types/` 目录
- **模块类型** - 同目录下的 `types.ts` 或 `*.d.ts`
- **全局类型** - `src/shared/types/global.d.ts`

---

## 转换模板示例

```typescript
// 转换前 (JS)
export const fetchData = async (url, params) => {
  const response = await fetch(url, params);
  return response.json();
};

// 转换后 (TS)
interface FetchParams {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

interface ApiResponse<T> {
  data: T;
  status: number;
}

export const fetchData = async <T>(
  url: string,
  params?: FetchParams
): Promise<ApiResponse<T>> => {
  const response = await fetch(url, params);
  const data = await response.json();
  return { data, status: response.status };
};
```

---

## 验证步骤

```bash
# 1. 运行 TypeScript 编译检查
cd alou-desktop
npx tsc --noEmit

# 2. 运行项目确保无错误
npm run dev

# 3. 运行测试（如有）
npm test
```

---

## 风险与注意事项

1. **渐进式转换** - 保持 JS 和 TS 并存，逐步迁移
2. **类型宽松** - 初期可使用 `any` 快速转换，后续逐步严格
3. **测试覆盖** - 转换后确保功能测试通过
4. **Git 提交** - 小批量提交，便于回滚

---

## 预计工作量

| 类别 | 数量 | 估计复杂度 |
|------|------|-----------|
| Services | 30 | 中-高 |
| Stores | 3 | 中 |
| Utils | 12 | 低-中 |
| i18n | 7 | 低 |
| **总计** | **52** | - |

---

## 下一步行动

1. ✅ 本计划文档评审
2. 开始按优先级转换第一优先级文件
3. 每转换完一个模块，验证编译和功能
