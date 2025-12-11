use crate::agent::cluster_action::{Task, TaskStatus};
use crate::utils::error::{AloudError, Result};
use serde_json::Value;
use std::collections::HashMap;

/// 工作流类型
#[derive(Debug, Clone)]
pub enum WorkflowType {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 条件分支
    Conditional {
        condition: String,
    },
}

/// 任务编排引擎
pub struct TaskOrchestrator;

impl TaskOrchestrator {
    pub fn new() -> Self {
        Self
    }

    /// 构建工作流
    pub fn build_workflow(
        &self,
        tasks: &[Task],
    ) -> Result<Vec<Vec<String>>> {
        // 分析任务依赖关系，构建执行计划
        let mut workflow: Vec<Vec<String>> = Vec::new();
        let mut completed = std::collections::HashSet::new();
        let mut remaining: Vec<String> = tasks.iter().map(|t| t.task_id.clone()).collect();

        while !remaining.is_empty() {
            let mut ready_tasks = Vec::new();

            // 找出所有可以执行的任务（依赖已满足）
            for task_id in &remaining {
                if let Some(task) = tasks.iter().find(|t| &t.task_id == task_id) {
                    let completed_ids: Vec<&str> = completed.iter().map(|s| s.as_str()).collect();
                    if task.can_execute(&completed_ids) {
                        ready_tasks.push(task_id.clone());
                    }
                }
            }

            if ready_tasks.is_empty() {
                // 无法继续，可能存在循环依赖或错误
                return Err(AloudError::InternalError(
                    "Cannot build workflow: circular dependency or missing dependencies".to_string(),
                ));
            }

            // 将就绪的任务添加到当前执行组
            workflow.push(ready_tasks.clone());

            // 标记为已完成
            for task_id in &ready_tasks {
                completed.insert(task_id.clone());
                remaining.retain(|id| id != task_id);
            }
        }

        Ok(workflow)
    }

    /// 顺序执行任务
    pub async fn execute_sequential(
        &self,
        tasks: &mut [Task],
        executor: impl Fn(&mut Task) -> Result<Value>,
    ) -> Result<Vec<Value>> {
        let mut results = Vec::new();

        for task in tasks.iter_mut() {
            if task.status != TaskStatus::Pending {
                continue;
            }

            task.mark_started();

            match executor(task) {
                Ok(result) => {
                    task.mark_completed(result.clone());
                    results.push(result);
                }
                Err(e) => {
                    task.mark_failed(e.to_string());
                    return Err(AloudError::InternalError(format!(
                        "Task {} failed: {}",
                        task.task_id, e
                    )));
                }
            }
        }

        Ok(results)
    }

    /// 并行执行任务
    pub async fn execute_parallel(
        &self,
        tasks: &mut [Task],
        executor: impl Fn(&mut Task) -> Result<Value>,
    ) -> Result<Vec<Value>> {
        // 在实际实现中，这里应该使用异步并发执行
        // 由于 Rust 的限制，这里使用顺序执行模拟并行
        let mut results = Vec::new();

        for task in tasks.iter_mut() {
            if task.status != TaskStatus::Pending {
                continue;
            }

            task.mark_started();

            match executor(task) {
                Ok(result) => {
                    task.mark_completed(result.clone());
                    results.push(result);
                }
                Err(e) => {
                    task.mark_failed(e.to_string());
                    // 并行执行中，一个失败不影响其他任务
                }
            }
        }

        Ok(results)
    }

    /// 条件分支执行
    pub async fn execute_conditional(
        &self,
        condition_task: &mut Task,
        true_branch: &mut [Task],
        false_branch: &mut [Task],
        executor: impl Fn(&mut Task) -> Result<Value>,
        condition_evaluator: impl Fn(&Value) -> bool,
    ) -> Result<Vec<Value>> {
        // 先执行条件任务
        condition_task.mark_started();
        let condition_result = executor(condition_task)?;
        condition_task.mark_completed(condition_result.clone());

        // 根据条件结果选择分支
        let branch_to_execute = if condition_evaluator(&condition_result) {
            true_branch
        } else {
            false_branch
        };

        // 标记未执行的分支为跳过
        let branch_to_skip = if condition_evaluator(&condition_result) {
            false_branch
        } else {
            true_branch
        };

        for task in branch_to_skip.iter_mut() {
            task.status = TaskStatus::Skipped;
        }

        // 执行选定的分支
        let mut results = vec![condition_result];
        for task in branch_to_execute.iter_mut() {
            task.mark_started();
            match executor(task) {
                Ok(result) => {
                    task.mark_completed(result.clone());
                    results.push(result);
                }
                Err(e) => {
                    task.mark_failed(e.to_string());
                    return Err(AloudError::InternalError(format!(
                        "Conditional branch task {} failed: {}",
                        task.task_id, e
                    )));
                }
            }
        }

        Ok(results)
    }

    /// 等待依赖任务完成
    pub fn wait_for_dependencies(
        &self,
        task: &Task,
        completed_tasks: &[&str],
    ) -> bool {
        task.can_execute(completed_tasks)
    }

    /// 聚合多个任务的结果
    pub fn aggregate_results(
        &self,
        results: Vec<Value>,
    ) -> Result<Value> {
        // 简单的聚合：将所有结果合并到一个数组中
        Ok(serde_json::json!(results))
    }

    /// 合并多个任务的结果到单个对象
    pub fn merge_results(
        &self,
        results: &HashMap<String, Value>,
    ) -> Result<Value> {
        // 将 HashMap 转换为 JSON 对象
        let mut merged = serde_json::Map::new();
        for (key, value) in results {
            merged.insert(key.clone(), value.clone());
        }
        Ok(Value::Object(merged))
    }
}

impl Default for TaskOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

