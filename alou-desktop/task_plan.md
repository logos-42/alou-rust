# Alou Cron Frontend Implementation Plan

## Goal
为 Alou 桌面端前端实现 Cron 定时任务管理界面和服务。

## Files to Create

1. `/Users/apple/Downloads/alou/alou-desktop/src/shared/types/cron.ts` - 类型定义
2. `/Users/apple/Downloads/alou/alou-desktop/src/utils/cronParser.ts` - Cron 表达式解析工具
3. `/Users/apple/Downloads/alou/alou-desktop/src/services/cronService.ts` - 前端服务
4. `/Users/apple/Downloads/alou/alou-desktop/src/components/Cron/CronManager.vue` - Vue 组件

## Implementation Phases

### Phase 1: Type Definitions
- [ ] Create `src/shared/types/cron.ts`
- [ ] Define CronJob, CronJobResult, CronConfig interfaces
- [ ] Define state enums and response types
- [ ] Export all types

### Phase 2: Cron Parser Utility
- [ ] Create `src/utils/cronParser.ts`
- [ ] Implement `parseCronExpression()` - 验证和解析
- [ ] Implement `getNextRunTime()` - 计算下次运行时间
- [ ] Implement `getHumanReadable()` - 转换为人类可读文本
- [ ] Add error handling for invalid expressions

### Phase 3: Cron Service
- [ ] Create `src/services/cronService.ts`
- [ ] Implement scheduler control methods:
  - startScheduler()
  - stopScheduler()
  - pauseScheduler()
  - resumeScheduler()
  - getSchedulerState()
- [ ] Implement job management methods:
  - listJobs()
  - addJob()
  - removeJob()
  - runJobNow()
  - toggleJob()
- [ ] Implement history and config methods:
  - getJobHistory()
  - getConfig()
  - updateConfig()
  - createDefaultConfig()
- [ ] Add error formatting
- [ ] Add response type handling

### Phase 4: Vue Component
- [ ] Create `src/components/Cron/CronManager.vue`
- [ ] Create component directory structure
- [ ] Implement UI sections:
  - 任务列表区域 (表格显示)
  - 添加/编辑对话框
  - 执行历史区域
  - 控制按钮区域
- [ ] Add Cron expression validation
- [ ] Implement human-readable conversion display
- [ ] Add responsive state management
- [ ] Style with scoped CSS

### Phase 5: Testing & Integration
- [ ] Verify Tauri invoke calls
- [ ] Test error handling
- [ ] Verify UI responsiveness
- [ ] Check type safety

## Backend Commands Available

From `src-tauri/src/cron/commands.rs`:
- `start_cron_scheduler` - 启动调度器
- `stop_cron_scheduler` - 停止调度器
- `pause_cron_scheduler` - 暂停调度器
- `resume_cron_scheduler` - 恢复调度器
- `get_cron_scheduler_state` - 获取状态
- `list_cron_jobs` - 列出所有任务
- `add_cron_job` - 添加任务
- `remove_cron_job` - 删除任务
- `run_cron_job_now` - 立即运行
- `toggle_cron_job` - 启用/禁用
- `get_cron_job_history` - 获取执行历史
- `get_cron_config` - 获取配置
- `update_cron_config` - 更新配置
- `create_default_cron_config` - 创建默认配置
- `clear_cron_job_history` - 清除历史
- `get_cron_config_path` - 获取配置文件路径

## Type Mapping (Rust -> TypeScript)

```rust
// Rust
pub struct CronJob {
    pub name: String,
    pub schedule: String,
    pub prompt: String,
    pub session_isolation: bool,
    pub result_file: Option<String>,
    pub enabled: bool,
}

pub enum CronJobState {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct CronJobResult {
    pub job_name: String,
    pub status: CronJobState,
    pub output: Option<String>,
    pub error: Option<String>,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: Option<u64>,
    pub session_id: Option<String>,
}

pub struct CronConfig {
    pub jobs: Vec<CronJob>,
    pub enabled: bool,
    pub check_interval_seconds: u64,
}
```

```typescript
// TypeScript
interface CronJob {
  name: string;
  schedule: string;
  prompt: string;
  session_isolation: boolean;
  result_file?: string;
  enabled: boolean;
}

type CronJobState = 'pending' | 'running' | 'completed' | 'failed';

interface CronJobResult {
  job_name: string;
  status: CronJobState;
  output?: string;
  error?: string;
  executed_at: string;
  execution_time_ms?: number;
  session_id?: string;
}

interface CronConfig {
  jobs: CronJob[];
  enabled: boolean;
  check_interval_seconds: number;
}
```

## Notes
- Use Composition API for Vue component
- Use `invoke` from `@tauri-apps/api/core`
- Follow existing service patterns (like heartbeatService.ts)
- Cron expression validation should be user-friendly
- UI should be clean and intuitive
- Error messages should be helpful
