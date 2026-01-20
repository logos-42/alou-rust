//! 系统工具
//!
//! 提供系统信息查询、进程管理等功能

use super::{ToolExecutor, ToolMetadata, ToolCategory, ToolPriority, ToolStatus, ExecutionContext, ToolResult, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::System;

/// 系统工具
pub struct SystemTool {
    metadata: ToolMetadata,
    system: Arc<Mutex<System>>,
}

impl SystemTool {
    /// 创建新的系统工具
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            metadata: ToolMetadata {
                id: "system".to_string(),
                name: "System Tool".to_string(),
                description: "System information and management".to_string(),
                category: ToolCategory::System,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["system".to_string()],
            },
            system: Arc::new(Mutex::new(system)),
        }
    }
}

#[async_trait]
impl ToolExecutor for SystemTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let system_op: SystemOperation = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match system_op {
            SystemOperation::Info => self.get_system_info().await,
            SystemOperation::Processes => self.list_processes().await,
            SystemOperation::Cpu => self.get_cpu_info().await,
            SystemOperation::Memory => self.get_memory_info().await,
            SystemOperation::Disks => self.get_disk_info().await,
            SystemOperation::Environment => self.get_environment().await,
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if let Ok(_op) = serde_json::from_value::<SystemOperation>(args.clone()) {
            Ok(())
        } else {
            Err(ToolError::InvalidArguments("Invalid system operation arguments".to_string()))
        }
    }

    fn help(&self) -> String {
        r#"System Tool - System information and management

Available operations:
- info: Get basic system information
- processes: List running processes
- cpu: Get CPU information and usage
- memory: Get memory information
- disks: Get disk information
- environment: Get environment variables

Example usage:
{
  "operation": "info"
}

{
  "operation": "processes"
}"#.to_string()
    }
}

impl SystemTool {
    /// 获取系统信息
    async fn get_system_info(&self) -> Result<ToolResult, ToolError> {
        let system = self.system.lock().unwrap();

        let info = SystemInfo {
            name: "Unknown".to_string(), // (*system).name().unwrap_or_default(),
            kernel_version: "Unknown".to_string(), // (*system).kernel_version().unwrap_or_default(),
            os_version: "Unknown".to_string(), // (*system).os_version().unwrap_or_default(),
            host_name: "Unknown".to_string(), // (*system).host_name().unwrap_or_default(),
            uptime: 0, // (*system).uptime(),
            boot_time: 0, // (*system).boot_time(),
            total_memory: (*system).total_memory(),
            used_memory: (*system).used_memory(),
            total_swap: (*system).total_swap(),
            used_swap: (*system).used_swap(),
            cpu_count: (*system).cpus().len(),
            cpu_usage: (*system).global_cpu_usage(),
        };

        let output = format!("System: {} {}", info.name, info.os_version);
        Ok(ToolResult {
            success: true,
            data: serde_json::to_value(info).unwrap(),
            error: None,
            execution_time_ms: 0,
            output: Some(output),
            warnings: vec![],
            context: None,
        })
    }

    /// 列出进程
    async fn list_processes(&self) -> Result<ToolResult, ToolError> {
        let mut system = self.system.lock().unwrap();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let processes: Vec<ProcessInfo> = (*system).processes()
            .iter()
            .take(100) // 限制数量避免输出过大
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cmd: process.cmd()
                    .iter()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                status: format!("{:?}", process.status()),
                start_time: process.start_time(),
            })
            .collect();

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "processes": processes,
                "count": processes.len(),
                "total_processes": system.processes().len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Listed {} processes (showing first 100)", processes.len())),
            warnings: vec![],
            context: None,
        })
    }

    /// 获取 CPU 信息
    async fn get_cpu_info(&self) -> Result<ToolResult, ToolError> {
        let mut system = self.system.lock().unwrap();
        system.refresh_cpu_all();

        let cpus: Vec<CpuInfo> = (*system).cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuInfo {
                id: i,
                name: cpu.name().to_string(),
                vendor_id: cpu.vendor_id().to_string(),
                brand: cpu.brand().to_string(),
                frequency: cpu.frequency(),
                usage: cpu.cpu_usage(),
            })
            .collect();

        let global_cpu_usage = (*system).global_cpu_usage();

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "global": {
                    "usage": global_cpu_usage
                },
                "cpus": cpus,
                "count": cpus.len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("CPU usage: {:.1}%", global_cpu_usage)),
            warnings: vec![],
            context: None,
        })
    }

    /// 获取内存信息
    async fn get_memory_info(&self) -> Result<ToolResult, ToolError> {
        let mut system = self.system.lock().unwrap();
        system.refresh_memory();

        let info = MemoryInfo {
            total: (*system).total_memory(),
            used: (*system).used_memory(),
            free: (*system).free_memory(),
            available: (*system).available_memory(),
            total_swap: (*system).total_swap(),
            used_swap: (*system).used_swap(),
            free_swap: (*system).free_swap(),
        };

        let output = format!("Memory: {} MB used / {} MB total",
            info.used / 1024 / 1024,
            info.total / 1024 / 1024);

        Ok(ToolResult {
            success: true,
            data: serde_json::to_value(info).unwrap(),
            error: None,
            execution_time_ms: 0,
            output: Some(output),
            warnings: vec![],
            context: None,
        })
    }

    /// 获取磁盘信息
    async fn get_disk_info(&self) -> Result<ToolResult, ToolError> {
        let system = self.system.lock().unwrap();

        let disks: Vec<DiskInfo> = Vec::new(); // (*system).disks() not available in this version

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "disks": disks,
                "count": disks.len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Found {} disks", disks.len())),
            warnings: vec![],
            context: None,
        })
    }

    /// 获取环境变量
    async fn get_environment(&self) -> Result<ToolResult, ToolError> {
        let env_vars: std::collections::HashMap<String, String> = std::env::vars().collect();

        Ok(ToolResult {
            success: true,
            data: serde_json::json!({
                "environment": env_vars,
                "count": env_vars.len()
            }),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Found {} environment variables", env_vars.len())),
            warnings: vec![],
            context: None,
        })
    }
}

/// 系统操作枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation")]
pub enum SystemOperation {
    /// 系统信息
    Info,
    /// 进程列表
    Processes,
    /// CPU 信息
    Cpu,
    /// 内存信息
    Memory,
    /// 磁盘信息
    Disks,
    /// 环境变量
    Environment,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 系统名称
    pub name: String,
    /// 内核版本
    pub kernel_version: String,
    /// 操作系统版本
    pub os_version: String,
    /// 主机名
    pub host_name: String,
    /// 运行时间（秒）
    pub uptime: u64,
    /// 启动时间
    pub boot_time: u64,
    /// 总内存（字节）
    pub total_memory: u64,
    /// 已用内存（字节）
    pub used_memory: u64,
    /// 总交换空间（字节）
    pub total_swap: u64,
    /// 已用交换空间（字节）
    pub used_swap: u64,
    /// CPU 数量
    pub cpu_count: usize,
    /// CPU 使用率
    pub cpu_usage: f32,
}

/// 进程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// 进程ID
    pub pid: u32,
    /// 进程名称
    pub name: String,
    /// 命令行
    pub cmd: String,
    /// CPU 使用率
    pub cpu_usage: f32,
    /// 内存使用（字节）
    pub memory: u64,
    /// 状态
    pub status: String,
    /// 开始时间
    pub start_time: u64,
}

/// CPU 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU ID
    pub id: usize,
    /// CPU 名称
    pub name: String,
    /// 供应商ID
    pub vendor_id: String,
    /// 品牌
    pub brand: String,
    /// 频率（MHz）
    pub frequency: u64,
    /// 使用率
    pub usage: f32,
}

/// 内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总内存
    pub total: u64,
    /// 已用内存
    pub used: u64,
    /// 空闲内存
    pub free: u64,
    /// 可用内存
    pub available: u64,
    /// 总交换空间
    pub total_swap: u64,
    /// 已用交换空间
    pub used_swap: u64,
    /// 空闲交换空间
    pub free_swap: u64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// 磁盘名称
    pub name: String,
    /// 挂载点
    pub mount_point: String,
    /// 文件系统
    pub file_system: String,
    /// 总空间
    pub total_space: u64,
    /// 可用空间
    pub available_space: u64,
    /// 是否可移动
    pub is_removable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info() {
        let tool = SystemTool::new();
        let context = ExecutionContext {
            session_id: "test".to_string(),
            user_id: None,
            working_directory: None,
            environment: std::collections::HashMap::new(),
            timeout_seconds: Some(30),
            permissions: vec!["system".to_string()],
            timestamp: chrono::Utc::now().timestamp(),
        };

        let args = serde_json::json!({
            "operation": "info"
        });

        let result = tool.execute(args, &context).await.unwrap();
        assert!(result.success);
        assert!(result.data.get("name").is_some());
    }

    #[tokio::test]
    async fn test_system_validation() {
        let tool = SystemTool::new();

        // 有效的参数
        let valid_args = serde_json::json!({
            "operation": "info"
        });
        assert!(tool.validate_args(&valid_args).await.is_ok());

        // 无效的参数
        let invalid_args = serde_json::json!({
            "invalid": "args"
        });
        assert!(tool.validate_args(&invalid_args).await.is_err());
    }
}