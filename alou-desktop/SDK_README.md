# 桌面版 SDK 功能说明

本文档介绍 Alou Desktop 新增的 SDK 功能，包括 LSP 和 Spec 两大功能模块。

## 功能概述

### 1. LSP (Language Server Protocol) 编辑器

LSP 编辑器提供智能代码编辑功能，支持多种编程语言。

#### 功能特性
- ✅ 智能代码补全 (Ctrl + Space)
- ✅ 代码诊断 (错误、警告检测)
- ✅ 悬停信息 (鼠标悬停查看文档)
- ✅ 文档符号 (函数、类等结构)
- ✅ 代码格式化
- ✅ 跳转到定义
- ✅ 查找引用
- ✅ 支持多种编程语言

#### 支持的编程语言
- JavaScript
- TypeScript
- Python
- Rust
- Go
- Java
- C#
- C++
- HTML
- CSS
- JSON
- Markdown

#### 使用方式
1. 点击右上角设置按钮 (⚙️)
2. 在设置面板中找到 "SDK 功能" 部分
3. 点击 "LSP 编辑器" 旁边的 "打开" 按钮
4. 选择编程语言
5. 开始编辑代码，使用 LSP 功能

### 2. Spec (Specification) 管理

Spec 管理提供规格文档的创建、编辑、验证和导出功能。

#### 功能特性
- ✅ 创建规格文档
- ✅ 编辑规格内容
- ✅ 删除规格文档
- ✅ 规格文档验证
- ✅ 导出为 Markdown
- ✅ 导出为 HTML
- ✅ 从模板生成
- ✅ 按类型筛选

#### 支持的规格类型
- **产品规格 (product)** - 产品需求文档
- **技术规格 (technical)** - 技术实现文档
- **设计规格 (design)** - UI/UX 设计文档
- **API 规格 (api)** - API 接口文档
- **用户故事 (user_story)** - 用户故事列表
- **任务规格 (tasks)** - 开发任务分解
- **结构规格 (structure)** - 项目结构说明

#### 使用方式
1. 点击右上角设置按钮 (⚙️)
2. 在设置面板中找到 "SDK 功能" 部分
3. 点击 "Spec 管理" 旁边的 "打开" 按钮
4. 选择规格类型
5. 创建或管理规格文档

## 技术架构

### 前端架构
```
src/
├── components/
│   ├── SDKComponents.jsx      # LSP 和 Spec 组件
│   ├── SDKComponents.css      # 组件样式
│   └── SDKModal.jsx           # SDK 功能模态框
├── services/
│   ├── lspService.js          # LSP 服务
│   └── specService.js         # Spec 服务
└── views/
    └── SdkTestView.jsx        # 测试页面
```

### 后端架构 (Rust)
```
src-tauri/src/
├── lsp.rs                      # LSP SDK 模块
├── spec.rs                     # Spec SDK 模块
└── main.rs                     # 主程序入口
```

### Node.js 脚本
```
scripts/
├── lsp-agent.js               # LSP 处理脚本
└── spec-agent.js              # Spec 处理脚本
```

## API 参考

### LSP Service API

```javascript
import lspService from '@/services/lspService'

// 代码补全
const completions = await lspService.getCompletions({
  code: 'function test() {}',
  language: 'javascript',
  position: { line: 0, character: 10 }
})

// 代码诊断
const diagnostics = await lspService.getDiagnostics({
  code: 'const x = 10;',
  language: 'javascript'
})

// 获取文档符号
const symbols = await lspService.getDocumentSymbols({
  code: 'function foo() {}',
  language: 'javascript'
})

// 代码格式化
const formatted = await lspService.formatCode({
  code: 'function  test  ()  {}',
  language: 'javascript'
})

// 获取支持的语言
const languages = await lspService.getSupportedLanguages()
```

### Spec Service API

```javascript
import specService from '@/services/specService'

// 创建规格文档
const spec = await specService.createSpec({
  specType: 'product',
  specData: {
    title: '新产品需求',
    content: '# 需求描述\n\n...',
    metadata: {
      version: '1.0.0',
      status: 'draft'
    }
  }
})

// 列出规格文档
const specs = await specService.listSpecs({ specType: 'product' })

// 验证规格文档
const validation = await specService.validateSpec({
  specId: 'spec-123'
})

// 导出规格文档
const exportResult = await specService.exportSpec({
  specId: 'spec-123',
  outputFormat: 'md'
})

// 删除规格文档
await specService.deleteSpec('spec-123')

// 获取模板列表
const templates = await specService.getTemplates()
```

## 开发指南

### 添加新的编程语言支持

1. 在 `scripts/lsp-agent.js` 的 `handleCompletion` 函数中添加语言处理逻辑
2. 在 `getSupportedLanguages` 中添加语言名称
3. 实现该语言的补全、诊断等功能

### 添加新的规格类型

1. 在 `src-tauri/src/spec.rs` 的 `SpecType` 枚举中添加新类型
2. 在 `scripts/spec-agent.js` 中实现相应的验证逻辑
3. 在模板目录中创建对应的模板文件

### 自定义 LSP 功能

修改 `scripts/lsp-agent.js` 中的处理函数：
- `handleCompletion` - 代码补全
- `handleDiagnostics` - 代码诊断
- `handleGoToDefinition` - 跳转定义
- `handleHover` - 悬停信息
- `handleDocumentSymbols` - 文档符号
- `handleFormat` - 代码格式化

## 测试

### 测试 LSP 功能

```bash
# 启动开发服务器
cd alou-desktop
npm run dev

# 在应用中打开设置 > SDK 功能 > LSP 编辑器
# 尝试以下操作：
# 1. 输入代码，按 Ctrl+Space 触发补全
# 2. 查看诊断信息
# 3. 悬停在代码上查看信息
```

### 测试 Spec 功能

```bash
# 启动开发服务器
cd alou-desktop
npm run dev

# 在应用中打开设置 > SDK 功能 > Spec 管理
# 尝试以下操作：
# 1. 创建新的规格文档
# 2. 编辑规格内容
# 3. 验证规格文档
# 4. 导出为 Markdown 或 HTML
```

## 注意事项

1. **Node.js 版本**：需要 Node.js >= 18.0.0
2. **性能**：LSP 和 Spec 功能在处理大型文件时可能会有性能影响
3. **持久化**：Spec 文档存储在 `~/.alou/specs/` 目录下
4. **错误处理**：所有 API 调用都应该进行错误处理
5. **安全性**：LSP 功能不应在不受信任的代码上使用

## 后续改进计划

- [ ] 支持更多的 LSP 功能（如重命名符号、查找引用）
- [ ] 添加实时协作编辑功能
- [ ] 支持导入/导出更多格式的规格文档
- [ ] 添加版本控制集成
- [ ] 支持自定义 LSP 服务器
- [ ] 添加规格文档模板市场
- [ ] 支持多人协作编辑规格文档

## 贡献

欢迎提交 Issue 和 Pull Request 来改进这些功能。

## 许可证

MIT License
