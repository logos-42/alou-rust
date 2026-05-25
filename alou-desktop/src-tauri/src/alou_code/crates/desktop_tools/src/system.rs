//! System Tool Adapter for alou_code Kernel

use alou_code_runtime::PermissionMode;
use sysinfo;
use serde_json::{json, Value};

pub fn tool_spec() -> (
    String,
    String,
    Value,
    PermissionMode,
    Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync>,
) {
    let name = "desktop_system".to_string();
    let description = "System information and operations".to_string();
    let schema = json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["info", "cpu", "memory", "disk", "processes"]
            }
        },
        "required": ["operation"]
    })
    let permission = PermissionMode::ReadOnly;

    let executor: Box<dyn Fn(&Value) -> Result<String, String> + Send + Sync> = Box::new(|input: &Value| {
        let operation = input.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        match operation {
            "info" => {
                let sys = sysinfo::System::new_all();
                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "os": sys.os_description().to_string(),
                    "hostname": sys.host_name().unwrap_or_default(),
                    "kernel_version": sys.kernel_version().unwrap_or_default(),
                })
            }
            "cpu" => {
                let sys = sysinfo::System::new_all();
                let cpus = sys.cpus();
                let cpu_info: Vec<_> = cpus.iter().map(|cpu| {
                    json!({
                        "name": cpu.name(),
                        "usage": cpu.cpu_usage(),
                        "frequency": cpu.frequency(),
                    })
                }).collect();

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "cpus": cpu_info,
                    "physical_core_count": sys.physical_core_count(),
                })
            }
            "memory" => {
                let sys = sysinfo::System::new_all();
                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "total_memory": sys.total_memory(),
                    "used_memory": sys.used_memory(),
                    "available_memory": sys.available_memory(),
                })
            }
            "disk" => {
                let sys = sysinfo::System::new_all();
                let disks = sysinfo::Disks::new_with_refreshed_list();
                let disk_info: Vec<_> = disks.iter().map(|disk| {
                    json!({
                        "name": disk.name().to_string_lossy(),
                        "mount_point": disk.mount_point().to_string_lossy(),
                        "total_space": disk.total_space(),
                        "available_space": disk.available_space(),
                    })
                }).collect();

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "disks": disk_info,
                })
            }
            "processes" => {
                let sys = sysinfo::System::new_all();
                let processes: Vec<_> = sys.processes().iter().take(10).map(|(pid, process)| {
                    json!({
                        "pid": pid.as_u32(),
                        "name": process.name().to_string_lossy(),
                        "cpu_usage": process.cpu_usage(),
                        "memory": process.memory(),
                    })
                }).collect();

                Ok(serde_json::to_string(&serde_json::json!({
                    "success": true,
                    "processes": processes,
                })
            }
            _ => Err(format!("Unknown operation: {}", operation))
        }
    })

    (name, description, schema, permission, executor)
}

pub fn tool_definition() -> ToolDefinition {
    let (name, description, schema, _, _) = tool_spec();
    ToolDefinition {
        name,
        description: Some(description),
        input_schema: schema,
    }
}
