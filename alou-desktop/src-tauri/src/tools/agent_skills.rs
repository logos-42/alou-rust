//! Agent Skills 系统
//!
//! 实现标准 Agent Skills 协议支持
//! 支持 SKILL.md 格式、Progressive Disclosure 和脚本执行

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use tokio::fs;
use regex::Regex;

/// Agent Skill 定义（标准格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    /// 技能名称
    pub name: String,
    /// 技能描述（用于发现）
    pub description: String,
    /// 技能版本
    pub version: Option<String>,
    /// 许可证
    pub license: Option<String>,
    /// 允许的工具列表
    pub allowed_tools: Option<Vec<String>>,
    /// 自定义元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 技能路径
    pub path: PathBuf,
    /// 完整指令内容（Progressive Disclosure Level 2）
    pub instructions: Option<String>,
    /// 是否已加载完整内容
    pub loaded: bool,
}

/// 技能发现信息（Progressive Disclosure Level 1）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

/// 技能执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionContext {
    /// 会话ID
    pub session_id: String,
    /// 技能名称
    pub skill_name: String,
    /// 执行ID
    pub execution_id: String,
    /// 输入参数
    pub inputs: HashMap<String, serde_json::Value>,
    /// 环境变量
    pub environment: HashMap<String, String>,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 调试模式
    pub debug_mode: bool,
}

/// 技能执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    /// 执行是否成功
    pub success: bool,
    /// 结果数据
    pub output: serde_json::Value,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 执行日志
    pub logs: Vec<String>,
}

/// Agent Skills 管理器
pub struct AgentSkillsManager {
    /// 技能存储目录
    skills_dir: PathBuf,
    /// 已发现的技能（Level 1: Metadata Only）
    discovered_skills: Arc<Mutex<HashMap<String, SkillMetadata>>>,
    /// 已加载的技能（Level 2: Full Content）
    loaded_skills: Arc<Mutex<HashMap<String, AgentSkill>>>,
}

impl AgentSkillsManager {
    /// 创建新的技能管理器
    pub fn new() -> Result<Self, ToolError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ToolError::InternalError("Cannot find home directory".to_string()))?;
        
        let skills_dir = home_dir.join(".alou").join("skills");
        
        Ok(Self {
            skills_dir,
            discovered_skills: Arc::new(Mutex::new(HashMap::new())),
            loaded_skills: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Progressive Disclosure Level 1: 发现技能（仅元数据）
    pub async fn discover_skills(&self) -> Result<Vec<SkillMetadata>, ToolError> {
        println!("[AgentSkillsManager] 开始发现技能...");
        println!("[AgentSkillsManager] 技能目录: {:?}", self.skills_dir);
        
        // 确保技能目录存在
        if !self.skills_dir.exists() {
            println!("[AgentSkillsManager] 技能目录不存在，创建目录...");
            fs::create_dir_all(&self.skills_dir).await
                .map_err(|e| ToolError::InternalError(format!("Failed to create skills directory: {}", e)))?;
        }

        let mut discovered = Vec::new();
        let mut entries = fs::read_dir(&self.skills_dir).await
            .map_err(|e| ToolError::InternalError(format!("Failed to read skills directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ToolError::InternalError(format!("Failed to read directory entry: {}", e)))? {
            
            let path = entry.path();
            println!("[AgentSkillsManager] 检查路径: {:?}", path);
            
            if path.is_dir() {
                let skill_file = path.join("SKILL.md");
                println!("[AgentSkillsManager] 检查技能文件: {:?}", skill_file);
                
                if skill_file.exists() {
                    println!("[AgentSkillsManager] 找到技能文件，解析元数据...");
                    match self.parse_skill_metadata(&skill_file).await {
                        Ok(metadata) => {
                            println!("[AgentSkillsManager] 成功解析技能: {}", metadata.name);
                            discovered.push(metadata.clone());
                            let mut skills = self.discovered_skills.lock().await;
                            skills.insert(metadata.name.clone(), metadata);
                        }
                        Err(e) => {
                            println!("[AgentSkillsManager] 解析技能失败: {:?} - {}", skill_file, e);
                            eprintln!("Failed to parse skill metadata from {:?}: {}", skill_file, e);
                        }
                    }
                } else {
                    println!("[AgentSkillsManager] 技能文件不存在: {:?}", skill_file);
                }
            }
        }

        println!("[AgentSkillsManager] 发现完成，找到 {} 个技能", discovered.len());
        Ok(discovered)
    }

    /// 解析技能元数据（仅 frontmatter）
    async fn parse_skill_metadata(&self, skill_file: &Path) -> Result<SkillMetadata, ToolError> {
        let content = fs::read_to_string(skill_file).await
            .map_err(|e| ToolError::InternalError(format!("Failed to read skill file: {}", e)))?;

        let (name, description) = self.extract_frontmatter(&content)?;
        
        Ok(SkillMetadata {
            name,
            description,
            path: skill_file.parent().unwrap().to_path_buf(),
        })
    }

    /// 提取 frontmatter 信息
    fn extract_frontmatter(&self, content: &str) -> Result<(String, String), ToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut name = None;
        let mut description = None;

        // 查找 YAML frontmatter 或 # 格式的元数据
        if content.starts_with("---") {
            // YAML frontmatter 格式
            let mut in_frontmatter = false;
            let mut frontmatter_lines = Vec::new();
            
            for line in lines {
                if line == "---" {
                    if in_frontmatter {
                        break; // 结束 frontmatter
                    } else {
                        in_frontmatter = true; // 开始 frontmatter
                        continue;
                    }
                }
                
                if in_frontmatter {
                    frontmatter_lines.push(line);
                }
            }
            
            let frontmatter = frontmatter_lines.join("\n");
            let yaml: serde_yaml::Value = serde_yaml::from_str(&frontmatter)
                .map_err(|e| ToolError::InvalidArguments(format!("Invalid YAML frontmatter: {}", e)))?;
            
            name = yaml.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
            description = yaml.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        } else {
            // # 格式的元数据
            let name_regex = Regex::new(r"^#\s*Name:\s*(.+)$").unwrap();
            let desc_regex = Regex::new(r"^#\s*Description:\s*(.+)$").unwrap();
            
            for line in lines {
                if let Some(captures) = name_regex.captures(line) {
                    name = Some(captures[1].trim().to_string());
                }
                if let Some(captures) = desc_regex.captures(line) {
                    description = Some(captures[1].trim().to_string());
                }
                
                // 如果遇到非注释行，停止解析
                if !line.starts_with('#') && !line.trim().is_empty() {
                    break;
                }
            }
        }

        let name = name.ok_or_else(|| ToolError::InvalidArguments("Missing 'name' in skill metadata".to_string()))?;
        let description = description.ok_or_else(|| ToolError::InvalidArguments("Missing 'description' in skill metadata".to_string()))?;

        Ok((name, description))
    }

    /// Progressive Disclosure Level 2: 加载完整技能
    pub async fn load_skill(&self, skill_name: &str) -> Result<AgentSkill, ToolError> {
        println!("[AgentSkillsManager] 开始加载技能: {}", skill_name);
        
        // 检查是否已加载
        {
            let loaded = self.loaded_skills.lock().await;
            if let Some(skill) = loaded.get(skill_name) {
                println!("[AgentSkillsManager] 技能已缓存: {}", skill_name);
                return Ok(skill.clone());
            }
        }

        // 从发现的技能中查找
        let skill_path = {
            let discovered = self.discovered_skills.lock().await;
            println!("[AgentSkillsManager] 当前发现的技能: {:?}", discovered.keys().collect::<Vec<_>>());
            
            let metadata = discovered.get(skill_name)
                .ok_or_else(|| ToolError::InvalidArguments(format!("Skill '{}' not found in discovered skills", skill_name)))?;
            println!("[AgentSkillsManager] 找到技能元数据，路径: {:?}", metadata.path);
            metadata.path.clone()
        };

        let skill_file = skill_path.join("SKILL.md");
        println!("[AgentSkillsManager] 读取技能文件: {:?}", skill_file);
        
        let content = fs::read_to_string(&skill_file).await
            .map_err(|e| ToolError::InternalError(format!("Failed to read skill file: {}", e)))?;

        println!("[AgentSkillsManager] 文件内容长度: {} 字符", content.len());
        
        let skill = self.parse_full_skill(&skill_path, &content).await?;
        
        println!("[AgentSkillsManager] 成功解析技能: {} (版本: {})", skill.name, skill.version.as_ref().unwrap_or(&"unknown".to_string()));

        // 缓存已加载的技能
        {
            let mut loaded = self.loaded_skills.lock().await;
            loaded.insert(skill_name.to_string(), skill.clone());
        }

        Ok(skill)
    }

    /// 解析完整技能内容
    async fn parse_full_skill(&self, skill_path: &Path, content: &str) -> Result<AgentSkill, ToolError> {
        let (name, description) = self.extract_frontmatter(content)?;
        
        // 解析完整的 frontmatter
        let lines: Vec<&str> = content.lines().collect();
        let mut version = None;
        let mut license = None;
        let mut allowed_tools = None;
        let mut metadata = HashMap::new();
        let mut instructions_start = 0;

        if content.starts_with("---") {
            // YAML frontmatter
            let mut in_frontmatter = false;
            let mut frontmatter_lines = Vec::new();
            
            for (i, line) in lines.iter().enumerate() {
                if *line == "---" {
                    if in_frontmatter {
                        instructions_start = i + 1;
                        break;
                    } else {
                        in_frontmatter = true;
                        continue;
                    }
                }
                
                if in_frontmatter {
                    frontmatter_lines.push(*line);
                }
            }
            
            let frontmatter = frontmatter_lines.join("\n");
            let yaml: serde_yaml::Value = serde_yaml::from_str(&frontmatter)
                .map_err(|e| ToolError::InvalidArguments(format!("Invalid YAML frontmatter: {}", e)))?;
            
            version = yaml.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
            license = yaml.get("license").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            if let Some(tools) = yaml.get("allowed-tools").and_then(|v| v.as_str()) {
                allowed_tools = Some(tools.split(',').map(|s| s.trim().to_string()).collect());
            }
            
            if let Some(meta) = yaml.get("metadata").and_then(|v| v.as_mapping()) {
                for (k, v) in meta {
                    if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                        metadata.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                    }
                }
            }
        } else {
            // # 格式的元数据
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with('#') {
                    // 解析其他元数据字段
                    if line.starts_with("# Version:") {
                        version = Some(line.replace("# Version:", "").trim().to_string());
                    } else if line.starts_with("# License:") {
                        license = Some(line.replace("# License:", "").trim().to_string());
                    } else if line.starts_with("# Allowed-Tools:") {
                        let tools_str = line.replace("# Allowed-Tools:", "").trim().to_string();
                        allowed_tools = Some(tools_str.split(',').map(|s| s.trim().to_string()).collect());
                    }
                } else if !line.trim().is_empty() {
                    instructions_start = i;
                    break;
                }
            }
        }

        // 提取指令内容
        let instructions = if instructions_start < lines.len() {
            lines[instructions_start..].join("\n")
        } else {
            String::new()
        };

        Ok(AgentSkill {
            name,
            description,
            version,
            license,
            allowed_tools,
            metadata: if metadata.is_empty() { None } else { Some(metadata) },
            path: skill_path.to_path_buf(),
            instructions: Some(instructions),
            loaded: true,
        })
    }

    /// Progressive Disclosure Level 3: 执行技能
    pub async fn execute_skill(&self, skill_name: &str, inputs: HashMap<String, serde_json::Value>, context: &ExecutionContext) -> Result<SkillExecutionResult, ToolError> {
        let start_time = std::time::Instant::now();
        let mut logs = Vec::new();

        // 加载技能
        let skill = self.load_skill(skill_name).await?;
        logs.push(format!("Loaded skill: {}", skill_name));

        // 创建执行上下文
        let execution_context = SkillExecutionContext {
            session_id: context.session_id.clone(),
            skill_name: skill_name.to_string(),
            execution_id: format!("exec_{}_{}", skill_name, uuid::Uuid::new_v4()),
            inputs,
            environment: context.environment.clone(),
            working_dir: skill.path.clone(),
            debug_mode: false,
        };

        // 执行技能指令
        let result = self.execute_skill_instructions(&skill, &execution_context, &mut logs).await?;

        Ok(SkillExecutionResult {
            success: result.success,
            output: result.output,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            error: result.error,
            logs,
        })
    }

    /// 执行技能指令
    async fn execute_skill_instructions(&self, skill: &AgentSkill, context: &SkillExecutionContext, logs: &mut Vec<String>) -> Result<SkillExecutionResult, ToolError> {
        let instructions = skill.instructions.as_ref()
            .ok_or_else(|| ToolError::ExecutionFailed("Skill instructions not loaded".to_string()))?;

        logs.push("Processing skill instructions".to_string());

        // 解析指令中的脚本引用 (@scripts/xxx)
        let script_regex = Regex::new(r"@(scripts|assets|references)/([^\s]+)").unwrap();
        let mut processed_instructions = instructions.clone();
        let mut script_outputs = HashMap::new();

        for captures in script_regex.captures_iter(instructions) {
            let full_match = &captures[0];
            let resource_type = &captures[1];
            let resource_path = &captures[2];
            
            let full_path = skill.path.join(resource_type).join(resource_path);
            
            match resource_type {
                "scripts" => {
                    // 执行脚本
                    match self.execute_script(&full_path, context, logs).await {
                        Ok(output) => {
                            script_outputs.insert(full_match.to_string(), output.clone());
                            processed_instructions = processed_instructions.replace(full_match, &output);
                            logs.push(format!("Executed script: {}", resource_path));
                        }
                        Err(e) => {
                            logs.push(format!("Failed to execute script {}: {}", resource_path, e));
                            return Ok(SkillExecutionResult {
                                success: false,
                                output: serde_json::json!({}),
                                execution_time_ms: 0,
                                error: Some(format!("Script execution failed: {}", e)),
                                logs: logs.clone(),
                            });
                        }
                    }
                }
                "assets" | "references" => {
                    // 读取文件内容
                    match fs::read_to_string(&full_path).await {
                        Ok(content) => {
                            processed_instructions = processed_instructions.replace(full_match, &content);
                            logs.push(format!("Loaded {}: {}", resource_type, resource_path));
                        }
                        Err(e) => {
                            logs.push(format!("Failed to load {} {}: {}", resource_type, resource_path, e));
                        }
                    }
                }
                _ => {}
            }
        }

        // 返回处理后的指令作为结果
        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "instructions": processed_instructions,
                "script_outputs": script_outputs,
                "skill_name": skill.name,
                "inputs": context.inputs
            }),
            execution_time_ms: 0,
            error: None,
            logs: logs.clone(),
        })
    }

    /// 执行脚本文件
    async fn execute_script(&self, script_path: &Path, context: &SkillExecutionContext, logs: &mut Vec<String>) -> Result<String, ToolError> {
        if !script_path.exists() {
            return Err(ToolError::ExecutionFailed(format!("Script not found: {:?}", script_path)));
        }

        let extension = script_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "py" => self.execute_python_script(script_path, context, logs).await,
            "js" => self.execute_node_script(script_path, context, logs).await,
            "sh" | "bash" => self.execute_bash_script(script_path, context, logs).await,
            _ => {
                // 尝试读取为文本文件
                fs::read_to_string(script_path).await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read script: {}", e)))
            }
        }
    }

    /// 执行 Python 脚本
    async fn execute_python_script(&self, script_path: &Path, context: &SkillExecutionContext, logs: &mut Vec<String>) -> Result<String, ToolError> {
        let mut cmd = tokio::process::Command::new("python");
        cmd.arg(script_path)
            .current_dir(&context.working_dir)
            .envs(&context.environment);

        // 传递输入参数作为环境变量
        for (key, value) in &context.inputs {
            if let Some(str_value) = value.as_str() {
                cmd.env(format!("INPUT_{}", key.to_uppercase()), str_value);
            }
        }

        logs.push(format!("Executing Python script: {:?}", script_path));
        
        let output = cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute Python script: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            logs.push("Python script executed successfully".to_string());
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            logs.push(format!("Python script failed: {}", stderr));
            Err(ToolError::ExecutionFailed(format!("Python script failed: {}", stderr)))
        }
    }

    /// 执行 Node.js 脚本
    async fn execute_node_script(&self, script_path: &Path, context: &SkillExecutionContext, logs: &mut Vec<String>) -> Result<String, ToolError> {
        let mut cmd = tokio::process::Command::new("node");
        cmd.arg(script_path)
            .current_dir(&context.working_dir)
            .envs(&context.environment);

        // 传递输入参数作为环境变量
        for (key, value) in &context.inputs {
            if let Some(str_value) = value.as_str() {
                cmd.env(format!("INPUT_{}", key.to_uppercase()), str_value);
            }
        }

        logs.push(format!("Executing Node.js script: {:?}", script_path));
        
        let output = cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute Node.js script: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            logs.push("Node.js script executed successfully".to_string());
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            logs.push(format!("Node.js script failed: {}", stderr));
            Err(ToolError::ExecutionFailed(format!("Node.js script failed: {}", stderr)))
        }
    }

    /// 执行 Bash 脚本
    async fn execute_bash_script(&self, script_path: &Path, context: &SkillExecutionContext, logs: &mut Vec<String>) -> Result<String, ToolError> {
        let mut cmd = if cfg!(windows) {
            let mut cmd = tokio::process::Command::new("powershell");
            cmd.arg("-File").arg(script_path);
            cmd
        } else {
            let mut cmd = tokio::process::Command::new("bash");
            cmd.arg(script_path);
            cmd
        };

        cmd.current_dir(&context.working_dir)
            .envs(&context.environment);

        // 传递输入参数作为环境变量
        for (key, value) in &context.inputs {
            if let Some(str_value) = value.as_str() {
                cmd.env(format!("INPUT_{}", key.to_uppercase()), str_value);
            }
        }

        logs.push(format!("Executing shell script: {:?}", script_path));
        
        let output = cmd.output().await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to execute shell script: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            logs.push("Shell script executed successfully".to_string());
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            logs.push(format!("Shell script failed: {}", stderr));
            Err(ToolError::ExecutionFailed(format!("Shell script failed: {}", stderr)))
        }
    }

    /// 获取所有已发现的技能
    pub async fn get_discovered_skills(&self) -> Vec<SkillMetadata> {
        let discovered = self.discovered_skills.lock().await;
        discovered.values().cloned().collect()
    }

    /// 搜索技能
    pub async fn search_skills(&self, query: &str) -> Vec<SkillMetadata> {
        let discovered = self.discovered_skills.lock().await;
        let query_lower = query.to_lowercase();
        
        discovered.values()
            .filter(|skill| {
                skill.name.to_lowercase().contains(&query_lower) ||
                skill.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

/// Agent Skills 工具
pub struct AgentSkillsTool {
    metadata: ToolMetadata,
    manager: AgentSkillsManager,
}

impl AgentSkillsTool {
    /// 创建新的 Agent Skills 工具
    pub fn new() -> Result<Self, ToolError> {
        let manager = AgentSkillsManager::new()?;
        
        Ok(Self {
            metadata: ToolMetadata {
                id: "agent_skills".to_string(),
                name: "Agent Skills Tool".to_string(),
                description: "管理和执行标准 Agent Skills 协议技能".to_string(),
                category: ToolCategory::Skills,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
            manager,
        })
    }
}

#[async_trait]
impl ToolExecutor for AgentSkillsTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "discover" => {
                let skills = self.manager.discover_skills().await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skills": skills,
                        "count": skills.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Discovered {} skills", skills.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "load" => {
                let skill_name = args.get("skill_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_name' field".to_string()))?;

                let skill = self.manager.load_skill(skill_name).await?;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skill": skill
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Loaded skill: {}", skill_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "execute" => {
                let skill_name = args.get("skill_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_name' field".to_string()))?;

                let inputs: HashMap<String, serde_json::Value> = args.get("inputs")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = self.manager.execute_skill(skill_name, inputs, context).await?;

                Ok(ToolResult {
                    success: result.success,
                    data: serde_json::json!({
                        "skill_name": skill_name,
                        "result": result
                    }),
                    error: result.error,
                    execution_time_ms: result.execution_time_ms,
                    output: Some(format!("Executed skill: {}", skill_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list" => {
                let skills = self.manager.get_discovered_skills().await;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "skills": skills,
                        "count": skills.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Listed {} skills", skills.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "search" => {
                let query = args.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'query' field".to_string()))?;

                let skills = self.manager.search_skills(query).await;
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "query": query,
                        "skills": skills,
                        "count": skills.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} skills matching '{}'", skills.len(), query)),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }

        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }

        Ok(())
    }

    fn help(&self) -> String {
        r#"Agent Skills Tool

Manage and execute standard Agent Skills protocol skills.

Actions:
  - discover: Discover all available skills in ~/.alou/skills/
  - load: Load a specific skill with full instructions
  - execute: Execute a skill with inputs
  - list: List all discovered skills
  - search: Search skills by query

Examples:

Discover skills:
{
  "action": "discover"
}

Load a skill:
{
  "action": "load",
  "skill_name": "web-scraper"
}

Execute a skill:
{
  "action": "execute",
  "skill_name": "web-scraper",
  "inputs": {
    "url": "https://example.com",
    "selector": ".content"
  }
}

Search skills:
{
  "action": "search",
  "query": "web"
}"#
        .to_string()
    }
}