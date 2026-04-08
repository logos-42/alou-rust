# Kaizen 循环整合计划

## 系统现状分析

### 已有组件
1. **alou-cli** - 主命令行工具，已有基础循环命令 (`cmd_loop`)
2. **hyperagent** - 完整的自进化智能体系统，包含两种循环模式：
   - **进化引擎** (Evolution Loop) - 进化"解决任务的代码"
   - **自动研究** (Auto Research) - Karpathy 风格，进化"系统自身的代码"

### 整合目标
将 hyperagent 的 Kaizen 循环能力集成到 alou-cli，实现：
- 从 CLI 直接触发进化循环
- 支持两种模式切换
- 统一的配置和状态管理
- 渐进式整合，保持向后兼容

## 整合架构

```
alou-cli/
├── src/
│   ├── main.rs                    # 现有 CLI 入口
│   ├── commands.rs                # 现有命令
│   ├── kaizen/                    # 新增：Kaizen 循环集成层
│   │   ├── mod.rs                 # 模块入口
│   │   ├── evolution_mode.rs      # 进化引擎模式
│   │   ├── research_mode.rs       # 自动研究模式
│   │   ├── config.rs              # Kaizen 配置
│   │   └── progress.rs            # 进度追踪
│   └── ...
├── Cargo.toml                     # 添加 hyperagent 依赖
└── .hyperagent/                   # 共享数据目录
```

## 实施步骤

### Phase 1: 基础集成 (当前)
1. ✅ 分析 hyperagent 架构
2. 🔄 在 alou-cli 中添加 hyperagent 依赖
3. 🔄 创建 kaizen 模块结构
4. 🔄 实现 CLI 命令集成：
   - `alou kaizen evolution` - 启动进化引擎
   - `alou kaizen research` - 启动自动研究
   - `alou kaizen status` - 查看循环状态
   - `alou kaizen stop` - 停止循环

### Phase 2: 配置与管理
5. 统一配置文件 (`kaizen_config.json`)
6. 环境变量支持
7. 进度监控和日志
8. Git 集成（自动 commit/push）

### Phase 3: 高级功能
9. Web 搜索增强
10. 多智能体协作
11. 可视化进度面板
12. 自动修复和回滚

## 命令设计

```bash
# 启动进化引擎（默认 5 代）
alou kaizen evolution --iterations 10 --task "优化性能"

# 启动自动研究（Karpathy 模式）
alou kaizen research --iterations 5 --auto-push

# 查看状态
alou kaizen status

# 停止循环
alou kaizen stop

# 查看实验日志
alou kaizen log --last 10

# 查看进化历史
alou kaizen history
```

## 关键实现细节

### 1. 依赖添加
在 `alou-cli/Cargo.toml` 中添加：
```toml
[dependencies]
hyperagent = { path = "../alou-desktop/hyperagent" }
```

### 2. 命令实现
复用 hyperagent 的 `EvolutionLoop` 和 `AutoResearch`，包装为 CLI 命令

### 3. 状态管理
- 使用现有的 `~/.alou/` 目录
- 共享 `.hyperagent/` 数据目录
- 统一进度和日志格式

### 4. 安全机制
- Dry run 模式默认
- Git checkpoint 自动创建
- 失败自动回滚

## 预期效果

整合后，用户可以从 alou-cli 直接：
1. 触发自我改进循环
2. 监控系统健康度
3. 查看改进历史
4. 配置循环参数
5. 自动提交和推送改进

系统会持续自我优化，实现真正的 Kaizen 持续改进理念。
