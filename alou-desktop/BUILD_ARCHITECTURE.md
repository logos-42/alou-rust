# 构建架构说明

## 新的模块化架构

### 目录结构
```
src/
├── modules/           # 业务模块 (领域驱动设计)
│   ├── agent-core/   # 代理核心模块
│   ├── wallet-core/  # 钱包核心模块
│   ├── ipfs-core/    # IPFS核心模块
│   └── crypto-core/  # 加密核心模块
├── ui/               # 纯UI层
│   ├── components/   # 展示组件
│   ├── hooks/        # UI相关hooks
│   └── views/        # 页面视图
├── shared/           # 共享代码
│   ├── types/        # 类型定义
│   ├── config/       # 配置管理
│   └── utils/        # 工具函数
└── bridges/          # Rust桥接层 (未来使用)
```

### 路径别名
- `@/` - 原有路径别名，保持向后兼容
- `@modules/` - 业务模块 (`src/modules/`)
- `@ui/` - UI层 (`src/ui/`)
- `@shared/` - 共享代码 (`src/shared/`)
- `@bridges/` - Rust桥接层 (`src/bridges/`)

## 构建配置更新

### TypeScript配置 (`tsconfig.json`)
已更新以支持新的目录结构和路径别名：
```json
{
  "paths": {
    "@/*": ["./src/*"],
    "@modules/*": ["./src/modules/*"],
    "@ui/*": ["./src/ui/*"],
    "@shared/*": ["./src/shared/*"],
    "@bridges/*": ["./src/bridges/*"]
  },
  "include": [
    "src",
    "src/modules",
    "src/ui",
    "src/shared",
    "src/bridges"
  ]
}
```

### Vite配置 (`vite.config.js`)
已更新路径别名：
```javascript
resolve: {
  alias: {
    '@': path.resolve(__dirname, './src'),
    '@modules': path.resolve(__dirname, './src/modules'),
    '@ui': path.resolve(__dirname, './src/ui'),
    '@shared': path.resolve(__dirname, './src/shared'),
    '@bridges': path.resolve(__dirname, './src/bridges'),
  },
}
```

## 迁移状态

### 当前状态 (阶段1完成)
- ✅ 创建了新的目录结构
- ✅ 配置了TypeScript支持
- ✅ 更新了构建配置
- ✅ 保持了向后兼容性

### 类型定义迁移
- ✅ 将原有类型文件迁移到 `@shared/types/`
- ✅ 保持了原有文件的向后兼容性
- ✅ 添加了新的全局类型定义

## 开发指南

### 使用新的路径别名
```javascript
// 旧方式 (仍然可用)
import AgentChat from '@/components/AgentChat';

// 新方式 (推荐)
import AgentChat from '@ui/views/AgentChat';
import { AgentService } from '@modules/agent-core/services';
import { config } from '@shared/config/app.config';
```

### 添加新模块
1. 在 `src/modules/` 下创建模块目录
2. 在 `@shared/config/app.config.js` 中注册模块
3. 在 `@shared/types/` 下添加类型定义
4. 更新构建配置（如果需要）

### 模块迁移流程
1. **分析现有代码** - 识别业务逻辑和UI逻辑
2. **创建模块结构** - 在 `modules/` 下创建对应模块
3. **迁移业务逻辑** - 将业务逻辑移到模块中
4. **重构UI组件** - 创建精简版UI组件
5. **测试验证** - 确保功能一致

## 性能优化

### 代码分割
新的架构支持自动代码分割：
- 按模块分割：每个业务模块独立打包
- 按路由分割：每个页面视图独立打包
- 第三方库分割：vendor chunk包含常用库

### 懒加载
```javascript
// 模块懒加载
const AgentModule = React.lazy(() => import('@modules/agent-core'));

// 路由懒加载
const AgentChat = React.lazy(() => import('@ui/views/AgentChat'));
```

## 向后兼容性

### 保持兼容的措施
1. **路径别名**：保留了 `@/` 别名
2. **类型重新导出**：原有类型文件重新导出新位置的内容
3. **渐进式迁移**：新旧代码可以并存
4. **适配器模式**：通过适配器连接新旧代码

### 迁移检查清单
- [ ] 确保所有现有导入仍然工作
- [ ] 验证构建过程无错误
- [ ] 测试应用功能正常
- [ ] 检查性能无退化

## 下一步计划

### 阶段2：代理模块重构
1. 分析 `AgentChat.jsx` (881行)
2. 创建 `agent-core` 模块
3. 迁移业务逻辑
4. 重构UI组件

### 阶段3：接口契约定义
1. 定义TypeScript接口
2. 实现端口适配器模式
3. 建立依赖管理系统

### 阶段4：性能监控
1. 实现性能监控系统
2. 添加基准测试
3. 实施优化措施

## 故障排除

### 常见问题
1. **导入错误**：检查路径别名是否正确配置
2. **类型错误**：确保类型定义文件已正确迁移
3. **构建失败**：验证Vite和TypeScript配置

### 调试建议
1. 使用 `console.log(config)` 查看配置
2. 检查浏览器开发者工具的Network标签
3. 查看构建日志中的警告和错误

## 相关文档
- [架构演进计划](../docs/REFACTORING_PLAN.md)
- [TypeScript配置](./tsconfig.json)
- [Vite配置](./vite.config.js)
- [应用配置](./src/shared/config/app.config.js)
