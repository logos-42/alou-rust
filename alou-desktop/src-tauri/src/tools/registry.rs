//! 工具注册表
//!
//! 管理所有已注册的工具，提供工具查询、执行、状态管理等功能
//!
//! 优化：使用 DashMap 替代 RwLock<HashMap> 以实现无锁并发读取

use super::{ToolExecutor, ToolMetadata, ToolStatus, ExecutionContext, ToolResult, ToolError, ToolCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

/// 工具注册表
///
/// 使用 DashMap 实现无锁并发读取，适合读多写少的场景
pub struct ToolRegistry {
    /// 工具映射 - 使用 DashMap 实现并发无锁读取
    tools: Arc<DashMap<String, Arc<dyn ToolExecutor>>>,
    /// 工具分类索引 - 使用 DashMap
    category_index: Arc<DashMap<ToolCategory, Vec<String>>>,
    /// 工具状态 - 使用 DashMap
    tool_status: Arc<DashMap<String, ToolStatus>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            category_index: Arc::new(DashMap::new()),
            tool_status: Arc::new(DashMap::new()),
        }
    }

    /// 注册工具
    pub async fn register(&mut self, tool: Arc<dyn ToolExecutor>) -> Result<(), ToolError> {
        let metadata = tool.metadata().clone();
        let tool_id = metadata.id.clone();

        // 检查是否已存在
        if self.tools.contains_key(&tool_id) {
            return Err(ToolError::ExecutionFailed(format!(
                "Tool '{}' already registered", tool_id
            )));
        }

        // 添加工具
        self.tools.insert(tool_id.clone(), tool);

        // 更新分类索引
        self.category_index
            .entry(metadata.category)
            .or_insert_with(Vec::new)
            .push(tool_id.clone());

        // 更新状态
        self.tool_status.insert(tool_id.clone(), metadata.status);

        log::info!("Tool '{}' registered successfully", tool_id);
        Ok(())
    }

    /// 注销工具
    pub async fn unregister(&self, tool_id: &str) -> Result<(), ToolError> {
        // 获取工具信息（用于清理索引）
        let metadata = {
            let tool = self.tools.get(tool_id)
                .ok_or_else(|| ToolError::ToolUnavailable(format!("Tool '{}' not found", tool_id)))?;
            tool.metadata().clone()
        };

        // 移除工具
        self.tools.remove(tool_id);

        // 从分类索引中移除
        if let Some(mut tool_list) = self.category_index.get_mut(&metadata.category) {
            tool_list.retain(|id| id != tool_id);
            if tool_list.is_empty() {
                self.category_index.remove(&metadata.category);
            }
        }

        // 移除状态
        self.tool_status.remove(tool_id);

        log::info!("Tool '{}' unregistered successfully", tool_id);
        Ok(())
    }

    /// 获取工具
    pub async fn get_tool(&self, tool_id: &str) -> Option<Arc<dyn ToolExecutor>> {
        self.tools.get(tool_id).map(|ref_multi| ref_multi.clone())
    }

    /// 检查工具是否存在
    pub async fn has_tool(&self, tool_id: &str) -> bool {
        self.tools.contains_key(tool_id)
    }

    /// 列出所有工具
    pub async fn list_all(&self) -> Vec<ToolMetadata> {
        // DashMap 迭代不需要显式加锁
        self.tools.iter()
            .map(|ref_multi| ref_multi.value().metadata().clone())
            .collect()
    }

    /// 按分类列出工具
    pub async fn list_by_category(&self, category: ToolCategory) -> Vec<ToolMetadata> {
        if let Some(tool_ids) = self.category_index.get(&category) {
            tool_ids.iter()
                .filter_map(|id| self.tools.get(id))
                .map(|ref_multi| ref_multi.metadata().clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 按状态列出工具
    pub async fn list_by_status(&self, status: ToolStatus) -> Vec<ToolMetadata> {
        self.tool_status.iter()
            .filter(|ref_multi| *ref_multi.value() == status)
            .filter_map(|ref_multi| self.tools.get(ref_multi.key()))
            .map(|ref_multi| ref_multi.metadata().clone())
            .collect()
    }

    /// 获取工具数量
    pub async fn count(&self) -> usize {
        self.tools.len()
    }

    /// 按分类统计工具数量
    pub async fn count_by_category(&self) -> HashMap<ToolCategory, usize> {
        self.category_index.iter()
            .map(|ref_multi| (*ref_multi.key(), ref_multi.value().len()))
            .collect()
    }

    /// 执行工具
    pub async fn execute(
        &self,
        tool_id: &str,
        args: serde_json::Value,
        context: ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self.get_tool(tool_id).await
            .ok_or_else(|| ToolError::ToolUnavailable(format!("Tool '{}' not found", tool_id)))?;

        tool.execute(args, &context).await
    }

    /// 验证工具参数
    pub async fn validate_args(&self, tool_id: &str, args: &serde_json::Value) -> Result<(), ToolError> {
        let tool = self.get_tool(tool_id).await
            .ok_or_else(|| ToolError::ToolUnavailable(format!("Tool '{}' not found", tool_id)))?;

        tool.validate_args(args).await
    }

    /// 获取工具帮助信息
    pub async fn get_help(&self, tool_id: &str) -> Result<String, ToolError> {
        let tool = self.get_tool(tool_id).await
            .ok_or_else(|| ToolError::ToolUnavailable(format!("Tool '{}' not found", tool_id)))?;

        Ok(tool.help())
    }

    /// 更新工具状态
    pub async fn update_tool_status(&self, tool_id: &str, status: ToolStatus) -> Result<(), ToolError> {
        if !self.tool_status.contains_key(tool_id) {
            return Err(ToolError::ToolUnavailable(format!("Tool '{}' not found", tool_id)));
        }

        self.tool_status.insert(tool_id.to_string(), status);
        Ok(())
    }

    /// 获取工具状态
    pub async fn get_tool_status(&self, tool_id: &str) -> Option<ToolStatus> {
        self.tool_status.get(tool_id).map(|ref_multi| *ref_multi.value())
    }

    /// 搜索工具
    pub async fn search(&self, query: &str) -> Vec<ToolMetadata> {
        let query_lower = query.to_lowercase();

        self.tools.iter()
            .filter(|ref_multi| {
                let metadata = ref_multi.value().metadata();
                metadata.name.to_lowercase().contains(&query_lower)
                    || metadata.description.to_lowercase().contains(&query_lower)
                    || metadata.id.to_lowercase().contains(&query_lower)
            })
            .map(|ref_multi| ref_multi.value().metadata().clone())
            .collect()
    }

    /// 获取所有分类
    pub async fn get_categories(&self) -> Vec<ToolCategory> {
        self.category_index.iter()
            .map(|ref_multi| *ref_multi.key())
            .collect()
    }

    /// 检查工具是否可用
    pub async fn is_available(&self, tool_id: &str) -> bool {
        if let Some(tool) = self.get_tool(tool_id).await {
            tool.is_available().await
        } else {
            false
        }
    }

    /// 批量执行工具
    pub async fn execute_batch(
        &self,
        requests: Vec<ToolExecutionRequest>,
    ) -> Vec<Result<ToolResult, ToolError>> {
        use futures::future::join_all;

        let futures = requests.into_iter().map(|req| {
            let registry = self.clone();
            async move {
                registry.execute(&req.tool_id, req.args, req.context).await
            }
        });

        join_all(futures).await
    }

    /// 清理所有工具
    pub async fn clear(&self) {
        self.tools.clear();
        self.category_index.clear();
        self.tool_status.clear();
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            tools: Arc::clone(&self.tools),
            category_index: Arc::clone(&self.category_index),
            tool_status: Arc::clone(&self.tool_status),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具执行请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    /// 工具 ID
    pub tool_id: String,
    /// 参数
    pub args: serde_json::Value,
    /// 执行上下文
    pub context: ExecutionContext,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具分类
    pub category: ToolCategory,
    /// 工具状态
    pub status: ToolStatus,
    /// 工具版本
    pub version: String,
}

impl From<ToolMetadata> for ToolDefinition {
    fn from(metadata: ToolMetadata) -> Self {
        Self {
            id: metadata.id,
            name: metadata.name,
            description: metadata.description,
            category: metadata.category,
            status: metadata.status,
            version: metadata.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock 工具用于测试
    struct MockTool {
        metadata: ToolMetadata,
    }

    #[async_trait]
    impl ToolExecutor for MockTool {
        fn metadata(&self) -> &ToolMetadata {
            &self.metadata
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
            _context: &ExecutionContext,
        ) -> Result<ToolResult, ToolError> {
            Ok(ToolResult {
                success: true,
                data: serde_json::json!({"result": "mock"}),
                error: None,
                execution_time_ms: 0,
                output: None,
                warnings: vec![],
                context: None,
            })
        }

        async fn validate_args(&self, _args: &serde_json::Value) -> Result<(), ToolError> {
            Ok(())
        }

        fn help(&self) -> String {
            "Mock tool".to_string()
        }
    }

    #[tokio::test]
    async fn test_register_tool() {
        let mut registry = ToolRegistry::new();

        let mock_tool = Arc::new(MockTool {
            metadata: ToolMetadata {
                id: "test_tool".to_string(),
                name: "Test Tool".to_string(),
                description: "A test tool".to_string(),
                category: ToolCategory::Other,
                priority: super::ToolPriority::Low,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                created_at: 0,
                updated_at: 0,
                dependencies: vec![],
                platforms: vec![],
                permissions: vec![],
                tags: vec![],
            },
        });

        let result = registry.register(mock_tool).await;
        assert!(result.is_ok());
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let mut registry = ToolRegistry::new();

        let mock_tool = Arc::new(MockTool {
            metadata: ToolMetadata {
                id: "test_tool".to_string(),
                name: "Test Tool".to_string(),
                description: "A test tool".to_string(),
                category: ToolCategory::Other,
                priority: super::ToolPriority::Low,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                created_at: 0,
                updated_at: 0,
                dependencies: vec![],
                platforms: vec![],
                permissions: vec![],
                tags: vec![],
            },
        });

        registry.register(mock_tool.clone()).await.unwrap();
        let result = registry.register(mock_tool).await;
        assert!(result.is_err());
    }
}
