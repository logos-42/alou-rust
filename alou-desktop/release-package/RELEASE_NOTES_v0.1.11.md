# Alou Desktop v0.1.11 发布说明

## 🚀 新特性

### Agent Swarm 系统
基于《人月神话》原则的分布式 AI Agent 协作系统：

- **并行任务执行** - 无依赖任务同时执行，充分利用多核 CPU
- **智能任务调度** - 基于依赖图自动计算关键路径
- **Agent 协作模式** - 支持 Leader-Follower、P2P、Hive-Mind 三种模式
- **事件驱动架构** - 完整的任务生命周期事件系统
- **负载均衡** - 智能分配任务给最合适的 Agent

### Skill 系统 v2.0
- 全局可复用的能力单元
- YAML 规范定义
- 版本管理和依赖解析
- 权限控制和资源限制

### Task 系统 v2.0
- 一次性执行单元
- 支持串行、并行、流水线、Map-Reduce 模式
- 依赖管理和条件执行
- 检查点和恢复机制

## 📦 安装包

| 平台 | 文件 | 大小 |
|------|------|------|
| macOS (Intel) | `Alou_0.1.11_x64.dmg` | 89 MB |

## 🔧 系统要求

- macOS 10.15+ (Intel)
- 4GB+ RAM
- 500MB 可用磁盘空间

## 📚 文档

- [架构设计](../../docs/agent-swarm/ARCHITECTURE.md)
- [Skill 规范](../../docs/agent-swarm/SKILL_SPEC.md)
- [Task 规范](../../docs/agent-swarm/TASK_SPEC.md)
- [使用示例](../../docs/agent-swarm/USAGE_EXAMPLES.md)

## 🐛 修复

- 优化了任务调度算法
- 改进了内存使用效率
- 修复了若干稳定性问题

## 🔜 下一步

- 支持更多平台 (Windows, Linux)
- 分布式远程 Agent 支持
- 可视化任务监控面板
- 更多内置 Skills

---

**完整变更日志**: 参见 [CHANGELOG.md](../../CHANGELOG.md)

**GitHub**: https://github.com/logos-42/alou-rust
