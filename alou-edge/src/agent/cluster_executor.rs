use crate::agent::cluster_action::{ClusterAction, Task, TaskStatus};
use crate::agent::core::AgentCore;
use crate::agent::session::SessionManager;
use crate::router::pubsub::PubSubManager;
use crate::utils::error::{AloudError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// 集群执行器
pub struct ClusterExecutor {
    agent_core: Arc<AgentCore>,
    #[allow(dead_code)]
    session_manager: SessionManager,
    pubsub_manager: PubSubManager,
}

impl ClusterExecutor {
    pub fn new(
        agent_core: Arc<AgentCore>,
        session_manager: SessionManager,
        pubsub_manager: PubSubManager,
    ) -> Self {
        Self {
            agent_core,
            session_manager,
            pubsub_manager,
        }
    }

    /// 为指定智能体执行任务
    pub async fn execute_task_for_agent(
        &self,
        task: &mut Task,
        agent_id: &str,
        wallet_address: Option<String>,
        chain: Option<String>,
    ) -> Result<Value> {
        // 构建任务消息
        let task_message = format!(
            "执行任务: {}\n任务类型: {}\n输入参数: {}",
            task.description,
            task.task_type,
            serde_json::to_string(&task.input).unwrap_or_default()
        );

        // 调用 AgentCore 处理消息
        let response = self
            .agent_core
            .handle_message(
                agent_id,
                &task_message,
                wallet_address,
                chain,
                Vec::new(),
                None,
                None,
            )
            .await?;

        // 构建结果
        let result = json!({
            "content": response.content,
            "tool_calls": response.tool_calls,
            "session_id": response.session_id,
        });

        Ok(result)
    }

    /// 处理智能体响应
    pub async fn handle_agent_response(
        &self,
        task: &mut Task,
        response: Value,
    ) -> Result<()> {
        // 检查响应是否成功
        if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
            task.mark_failed(error.to_string());
            return Err(AloudError::AgentError(error.to_string()));
        }

        // 提取结果
        let result = response.get("content")
            .or_else(|| response.get("result"))
            .cloned()
            .unwrap_or(response);

        task.mark_completed(result);
        Ok(())
    }

    /// 合并多个智能体的结果
    pub fn merge_results(
        &self,
        results: &HashMap<String, Value>,
    ) -> Result<Value> {
        let mut merged = serde_json::Map::new();
        
        for (agent_id, result) in results {
            merged.insert(agent_id.clone(), result.clone());
        }

        Ok(Value::Object(merged))
    }

    /// 处理执行失败情况
    #[allow(dead_code)]
    pub async fn handle_failure(
        &self,
        task: &mut Task,
        error: String,
        retry: bool,
    ) -> Result<()> {
        if retry && task.can_retry() {
            task.increment_retry();
            task.status = TaskStatus::Pending;
            task.error = None;
            task.started_at = None;
            Ok(())
        } else {
            task.mark_failed(error.clone());
            Err(AloudError::AgentError(error))
        }
    }

    /// 执行集群行动
    pub async fn execute_cluster_action(
        &self,
        mut action: ClusterAction,
        wallet_address: Option<String>,
        chain: Option<String>,
    ) -> Result<ClusterAction> {
        action.mark_started();

        // 创建 PubSub 群聊主题
        let topic = self.pubsub_manager
            .create_group_topic(&action.action_id)
            .await
            .map_err(|e| AloudError::InternalError(format!("Failed to create group topic: {}", e)))?;
        action.group_topic = Some(topic.clone());

        let mut completed_tasks = Vec::new();
        let mut task_results = HashMap::new();

        // 按执行顺序处理任务
        for task_id in &action.execution_plan.execution_order {
            if let Some(task) = action
                .execution_plan
                .tasks
                .iter_mut()
                .find(|t| &t.task_id == task_id)
            {
                // 检查依赖是否满足
                let completed_strs: Vec<&str> = completed_tasks.iter().map(|s: &String| s.as_str()).collect();
                if !task.can_execute(&completed_strs) {
                    continue;
                }

                // 获取分配的智能体
                let agent_id = if let Some(ref assignment) = task.assigned_agent {
                    assignment.agent_id.clone()
                } else {
                    return Err(AloudError::InternalError(format!(
                        "Task {} has no assigned agent",
                        task.task_id
                    )));
                };

                // 发布任务请求到 PubSub
                let _ = self.pubsub_manager
                    .publish_task_request(
                        &topic,
                        &task.task_id,
                        &task.description,
                        &agent_id,
                    )
                    .await;

                // 执行任务（带重试）
                let mut retry_count = 0;
                let task_result = loop {
                    task.mark_started();

                    match self
                        .execute_task_for_agent(task, &agent_id, wallet_address.clone(), chain.clone())
                        .await
                    {
                        Ok(result) => {
                            self.handle_agent_response(task, result.clone()).await?;
                            task_results.insert(agent_id.clone(), result.clone());
                            completed_tasks.push(task.task_id.clone());
                            break Ok(result);
                        }
                        Err(e) => {
                            if task.can_retry() && retry_count < task.max_retries {
                                retry_count += 1;
                                task.increment_retry();
                                task.status = TaskStatus::Pending;
                                // 继续重试
                            } else {
                                task.mark_failed(e.to_string());
                                break Err(AloudError::InternalError(format!(
                                    "Task {} failed after {} retries",
                                    task.task_id, retry_count
                                )));
                            }
                        }
                    }
                };

                // 发布任务结果到 PubSub
                match &task_result {
                    Ok(result) => {
                        let _ = self.pubsub_manager
                            .publish_task_result(
                                &topic,
                                &task.task_id,
                                result,
                                &agent_id,
                            )
                            .await;
                    }
                    Err(_) => {
                        // 任务失败时也发布结果（包含错误信息）
                        let error_result = json!({
                            "error": task.error.as_ref().unwrap_or(&"Unknown error".to_string()),
                            "status": "failed",
                        });
                        let _ = self.pubsub_manager
                            .publish_task_result(
                                &topic,
                                &task.task_id,
                                &error_result,
                                &agent_id,
                            )
                            .await;
                    }
                }

                // 如果任务失败，返回错误
                task_result?;
            }
        }

        // 检查是否所有任务都完成
        if action.all_tasks_completed() {
            // 合并结果
            let final_result = self.merge_results(&task_results)?;
            action.mark_completed(final_result);
        } else {
            // 部分任务失败
            let failed_tasks = action.get_failed_tasks();
            let error_msg = format!(
                "Some tasks failed: {:?}",
                failed_tasks.iter().map(|t| &t.task_id).collect::<Vec<_>>()
            );
            action.mark_failed(error_msg.clone());
            return Err(AloudError::InternalError(error_msg));
        }

        Ok(action)
    }
}

