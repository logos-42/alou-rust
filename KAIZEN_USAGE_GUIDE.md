# Kaizen 自进化循环 - 使用指南

## 📚 概述

Kaizen 自进化循环系统已成功整合到 alou-cli 中，基于 hyperagent 的强大能力，提供两种运行模式：

1. **进化引擎模式** - 进化"解决任务的代码"，通过多代迭代优化代码质量
2. **自动研究模式** - Karpathy 风格，进化"系统自身的代码"，自动改进项目源码

## 🚀 快速开始

### 前置准备

#### 1. 设置 LLM API Key

Kaizen 循环需要 LLM API 来执行智能分析和代码改进。选择以下任一方式：

**方式 A: 环境变量**
```bash
export LLM_API_KEY=your_api_key_here
export LLM_PROVIDER=openai  # 可选: openai, ollama, qwen
export LLM_MODEL=gpt-4o     # 可选，默认 gpt-4o
export LLM_BASE_URL=https://api.openai.com/v1  # 可选，自定义端点
```

**方式 B: .env 文件**
在项目根目录创建 `.env` 文件：
```bash
LLM_API_KEY=your_api_key_here
LLM_PROVIDER=openai
LLM_MODEL=gpt-4o
```

#### 2. 构建项目

```bash
cd alou-cli
cargo build --release
```

## 📖 命令参考

### 1. 进化引擎模式

进化解决特定任务的代码，适合优化算法、提升性能等场景。

```bash
# 基本用法（5 代进化）
alou kaizen evolution

# 自定义迭代次数和任务
alou kaizen evolution --iterations 10 --task "优化排序算法性能"

# 使用环境变量控制行为
ITERATIONS=10 alou kaizen evolution --task "提高代码可读性"
```

**选项说明：**
- `--iterations <N>` 或 `-n <N>`: 迭代次数（默认 5）
- `--task <描述>` 或 `-t <描述>`: 任务描述

**环境变量：**
- `KAIZEN_ITERATIONS`: 迭代次数
- `KAIZEN_DRY_RUN`: 安全模式（true/false）
- `LLM_API_KEY`: LLM API 密钥

### 2. 自动研究模式（Karpathy 风格）

自动改进项目自身的代码，遵循"假设 → 实验 → 观察 → 反思"循环。

```bash
# 基本用法（安全模式，不实际修改）
alou kaizen research

# 自动提交并推送到 GitHub
alou kaizen research --auto-push --no-dry-run

# 严格模式（测试 100% 通过才接受）
alou kaizen research --strict --iterations 10

# 指定目标文件
alou kaizen research --target "lib.rs" --target "runtime/loop_.rs"
```

**选项说明：**
- `--iterations <N>` 或 `-n <N>`: 迭代次数（默认 5）
- `--auto-push`: 自动推送到 GitHub
- `--strict`: 严格模式
- `--dry-run`: 安全模式（默认启用）
- `--no-dry-run`: 禁用安全模式
- `--target <文件>`: 目标文件（可多次指定）

**环境变量：**
- `RESEARCH_AUTO_PUSH`: 自动推送（true/false）
- `RESEARCH_DRY_RUN`: 安全模式（true/false）
- `RESEARCH_STRICT`: 严格模式（true/false）
- `RESEARCH_ITERATIONS`: 迭代次数
- `RESEARCH_WEB`: 启用 Web 搜索（true/false）

### 3. 状态监控

```bash
# 查看循环状态
alou kaizen status

# 查看实验日志（最近 20 行）
alou kaizen log

# 查看指定行数
alou kaizen log --last 50
```

### 4. 停止循环

```bash
alou kaizen stop
```

## 💡 使用示例

### 示例 1: 优化代码性能

```bash
# 设置 API Key
export LLM_API_KEY=sk-xxx

# 启动进化引擎，优化性能
alou kaizen evolution --iterations 10 --task "优化 Fibonacci 数列计算性能"
```

**预期输出：**
```
=== 启动 Kaizen 进化引擎 ===
模式: 进化引擎
迭代次数: 10
任务: 优化 Fibonacci 数列计算性能
安全模式: 是

🚀 启动 Kaizen 进化引擎
📡 使用 LLM: OpenAI, 模型: gpt-4o
📋 任务: 优化 Fibonacci 数列计算性能
🔄 开始进化循环 (10 代)...
[进化日志...]
✅ 进化完成!
🏆 最佳智能体:
   ID: gen10_init0
   代数: 10
```

### 示例 2: 自动改进项目代码

```bash
# 在项目根目录运行
cd /path/to/your/project

# 启动自动研究（安全模式）
alou kaizen research --iterations 5

# 确认改进后，实际运行
alou kaizen research --iterations 5 --no-dry-run --auto-push
```

### 示例 3: 持续集成场景

```bash
# 在 CI/CD 中使用
export LLM_API_KEY=$CI_LLM_API_KEY

# 严格模式，自动提交
RESEARCH_STRICT=true \
RESEARCH_AUTO_PUSH=true \
RESEARCH_ITERATIONS=10 \
alou kaizen research
```

## 📊 数据目录

Kaizen 循环会自动创建以下目录存储数据：

```
~/.alou/
├── kaizen/
│   ├── config.json          # 配置文件
│   ├── progress.json        # 进度追踪
│   └── logs/                # 运行日志
│       └── kaizen_20260408.log
└── .hyperagent/
    ├── data/                # 进化数据
    │   ├── archive.json     # 进化档案
    │   └── lineage.json     # 血统树
    └── experiments/
        └── research_log.md  # 实验日志
```

## 🔧 配置说明

### 配置文件

编辑 `~/.alou/kaizen/config.json`：

```json
{
  "mode": "evolution",
  "max_iterations": 5,
  "auto_commit": true,
  "auto_push": false,
  "push_interval": 5,
  "dry_run": true,
  "strict": false,
  "enable_web_search": true,
  "llm_provider": "openai",
  "llm_model": "gpt-4o",
  "llm_api_key": "",
  "llm_base_url": null
}
```

### 优先级

配置优先级（从高到低）：
1. 命令行参数
2. 环境变量
3. 配置文件
4. 默认值

## 🛡️ 安全机制

### 1. 安全模式（Dry Run）

**默认启用**，在此模式下：
- ✅ 执行所有分析和测试
- ✅ 生成改进建议
- ❌ 不实际修改文件
- ❌ 不提交 Git

### 2. 自动回滚

如果改进导致编译失败或测试退化：
- 自动回滚到修改前的状态
- 记录失败原因到实验日志
- 跳过当前迭代，继续下一次

### 3. Git Checkpoint

每次修改前自动创建 Git tag：
```bash
kaizen-checkpoint-before-iteration-5
```

方便手动回滚：
```bash
git checkout kaizen-checkpoint-before-iteration-5
```

## 📈 监控和日志

### 实时进度

```bash
watch -n 5 'alou kaizen status'
```

### 实验日志

```bash
# 查看实时日志
tail -f ~/.alou/.hyperagent/experiments/research_log.md

# 查看进化档案
cat ~/.alou/.hyperagent/data/archive.json

# 查看血统树
cat ~/.alou/.hyperagent/data/lineage.json
```

## 🎯 最佳实践

### 1. 渐进式使用

```bash
# 第 1 步: 安全模式观察
alou kaizen research --dry-run

# 第 2 步: 确认效果后，实际运行
alou kaizen research --no-dry-run

# 第 3 步: 自动提交
alou kaizen research --no-dry-run --auto-push
```

### 2. 针对特定模块

```bash
# 只改进特定文件
alou kaizen research --target "src/runtime/loop_.rs" --target "src/eval/evaluator.rs"
```

### 3. 持续集成

```bash
# 在 CI 中定期运行
0 2 * * * cd /path/to/project && alou kaizen research --strict --auto-push >> /var/log/kaizen.log 2>&1
```

## ❓ 常见问题

### Q: 如何知道循环是否在工作？

A: 运行 `alou kaizen status` 查看状态，或检查日志文件。

### Q: 如何回滚不想要的改进？

A: 使用 Git 回滚：
```bash
git log --oneline  # 找到要回滚的 commit
git revert <commit-hash>
```

### Q: 支持哪些 LLM 提供商？

A: 目前支持：
- OpenAI（默认）
- Ollama（本地）
- Qwen（阿里云）
- 任何 OpenAI 兼容 API

### Q: 如何提高改进质量？

A: 
1. 使用更强大的模型（如 gpt-4o）
2. 启用严格模式（`--strict`）
3. 增加迭代次数
4. 启用 Web 搜索获取外部知识

## 🔄 Kaizen 循环流程

```
┌─────────────────────────────────────────┐
│           Kaizen 循环                     │
│                                          │
│  1. 读取代码 → 2. 基线测试               │
│  3. 提出假设 → 4. 应用改进               │
│  5. 编译检查 → 6. 运行测试               │
│  7. LLM 反思 → 8. 提交改进               │
│  9. 推送代码 → 10. 记录日志              │
│         ↓                                │
│    返回步骤 1（下一代）                   │
└─────────────────────────────────────────┘
```

## 📚 相关资源

- [Kaizen 整合计划](./KAIZEN_INTEGRATION_PLAN.md)
- [Hyperagent 文档](./alou-desktop/hyperagent/README.md)
- [Alou CLI 文档](./alou-cli/README.md)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**开始使用 Kaizen，让代码持续自我改进！** 🚀
