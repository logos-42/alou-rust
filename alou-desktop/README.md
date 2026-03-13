# Alou 自主智能体系统

## 📖 概述

Alou 是一个具备自主学习和进化能力的 AI Agent 系统。

## 🏗️ 系统架构

```
用户
  ↓
自主循环层
  ├── 心跳检查 (30秒)
  ├── 任务发现
  ├── 技能选择 (AI 匹配)
  ├── 执行监控
  └── 学习反馈
  ↓
Skills 执行层
  ├── 技能发现
  ├── 技能理解
  ├── 技能选择
  ├── 技能执行
  └── 效果评估
  ↓
工具层
  ├── 文件工具
  ├── 网络工具
  ├── 系统工具
  └── AI 工具
```

## 🚀 快速开始

### 1. 启动 Alou

```bash
cd /Users/apple/Downloads/alou/alou-desktop
npm run dev
# 打开 http://localhost:1420
```

### 2. 使用 CLI

```bash
# 查看状态
alou status

# 添加任务
alou task add "检查邮件" "检查未读邮件" high

# AI 对话
alou agent chat "你好"

# 启动自主循环
alou start
```

### 3. 运行测试

```bash
# 完整测试
node scripts/test-full-loop.cjs

# 匹配测试
node scripts/skill-matcher.cjs

# 自主演示
node scripts/demo-complete.cjs
```

## 📁 文件结构

```
alou-desktop/
├── src/
│   ├── services/
│   │   ├── skillsExecutorService.ts    # Skills 执行服务
│   │   ├── feedbackService.ts          # 用户反馈服务
│   │   ├── emailCheckerService.ts      # 邮件检查服务
│   │   └── autonomousLoopService.ts    # 自主循环服务
│   │
│   ├── skills/
│   │   └── WorkflowSkill.ts           # 工作流技能
│   │
│   └── views/
│       └── AutonomousLoopPanel.jsx    # 控制面板
│
├── scripts/
│   ├── test-full-loop.cjs           # 完整测试
│   ├── skill-matcher.cjs            # 匹配测试
│   └── demo-complete.cjs             # 完整演示
│
├── docs/
│   ├── ARCHITECTURE.md              # 架构文档
│   └── PROGRESS.md                  # 进度汇报
│
└── src-tauri/src/
    ├── autonomous_loop.rs           # Rust 自主循环
    └── autonomous_loop_commands.rs   # 命令接口
```

## 🎯 核心功能

### 1. 自主任务执行

```
接收任务 → 分析需求 → 选择技能 → 执行 → 学习
```

### 2. Skills 自动选择

系统会根据任务描述自动选择最佳技能：

| 任务类型 | 匹配技能 |
|---------|---------|
| 检查邮件 | EmailChecker |
| 管理工作流 | WorkflowSkill |
| 网络搜索 | WebSearch |
| 代码分析 | CodeAnalyzer |
| Git 操作 | GitHelper |

### 3. 用户反馈机制

用户可以对执行结果评分 (1-5)，系统会：
- 记录每次反馈
- 计算技能统计
- 提供改进建议

### 4. 学习进化

系统会从反馈中学习：
- 调整技能选择权重
- 优化匹配算法
- 改进执行策略

## 🧪 测试结果

### 匹配测试
```
检查邮件     → EmailChecker (100%)
管理工作流   → WorkflowSkill (92.5%)
搜索资讯     → WebSearch (72.5%)
查看代码     → CodeAnalyzer (77.0%)
提交代码     → GitHelper (59.3%)
```

**合理率: 87.5%**

### 完整测试
- 通过率: 94.1% (16/17)

## 📊 配置

### 环境变量
```bash
DEEPSEEK_API_KEY=sk-xxx  # DeepSeek API Key
```

### 项目配置
- `.env.local` - 前端环境变量
- `~/.zshrc` - Shell 环境变量

## 🔧 开发

### 添加新 Skill

1. 在 `src/skills/` 添加 Skill 文件
2. 定义技能描述和关键词
3. 实现执行逻辑
4. 注册到系统

### 添加新工具

1. 在 `src/tools/` 添加工具
2. 实现 ToolExecutor trait
3. 注册到 ToolRegistry

## 📈 性能指标

| 指标 | 目标 | 当前 |
|------|------|------|
| 任务完成率 | 90% | 100% |
| 技能选择准确率 | 95% | 87.5% |
| 平均响应时间 | < 500ms | < 200ms |
| 用户满意度 | 4.5/5 | - |

## 🎯 长期目标

1. **短期**: 完善 Skills 执行逻辑
2. **中期**: 建立用户反馈闭环
3. **长期**: 实现完全自主学习和进化

## 📝 文档

- [架构文档](docs/ARCHITECTURE.md)
- [进度汇报](docs/PROGRESS.md)
- [API 文档](docs/API.md)

## 🤝 贡献

1. Fork 项目
2. 创建分支 (`git checkout -b feature/xxx`)
3. 提交更改 (`git commit -am 'Add xxx'`)
4. 推送到分支 (`git push origin feature/xxx`)
5. 创建 Pull Request

## 📄 许可证

MIT License

---

**版本**: v0.2.2
**更新**: 2026-03-13
**状态**: 持续开发中 🚀
