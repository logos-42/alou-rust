//! 脚本执行器
//!
//! 执行脚本代码技能

use super::executor::{SkillExecutor, SkillExecutionContext, SkillExecutionResult, SkillExecutionStep};
use super::manifest::SkillManifest;
use async_trait::async_trait;

/// 脚本执行器
pub struct ScriptExecutor {
    manifest: SkillManifest,
    language: String,
}

impl ScriptExecutor {
    pub fn new(manifest: SkillManifest, language: String) -> Self {
        Self { manifest, language }
    }

    /// 执行 JavaScript 代码 (使用 Deno 或 Node.js)
    async fn execute_javascript(&self, code: &str, _context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        // 实际实现应该调用 Deno 或 Node.js 运行时
        // 这里返回模拟结果
        let elapsed = 0u64;

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "language": "javascript",
                "code_length": code.len(),
                "note": "JavaScript execution requires Deno/Node runtime"
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![],
            error: None,
            metrics: std::collections::HashMap::new(),
        })
    }

    /// 执行 Python 代码
    async fn execute_python(&self, code: &str, _context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        // 实际实现应该调用 Python 解释器
        let elapsed = 0u64;

        Ok(SkillExecutionResult {
            success: true,
            output: serde_json::json!({
                "language": "python",
                "code_length": code.len(),
                "note": "Python execution requires Python runtime"
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![],
            error: None,
            metrics: std::collections::HashMap::new(),
        })
    }

    /// 执行 Shell 脚本
    async fn execute_shell(&self, code: &str, _context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        use tokio::process::Command;

        let start_time = std::time::Instant::now();

        // 创建临时脚本文件
        let temp_file = std::env::temp_dir().join(format!("skill_script_{}.sh", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp_file, code).await
            .map_err(|e| format!("Failed to write temp script: {}", e))?;

        // 执行脚本
        let output = Command::new("sh")
            .arg(&temp_file)
            .output()
            .await
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        // 清理临时文件
        let _ = tokio::fs::remove_file(&temp_file).await;

        let elapsed = start_time.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(SkillExecutionResult {
            success: output.status.success(),
            output: serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code()
            }),
            execution_time_ms: elapsed,
            intermediate_steps: vec![
                SkillExecutionStep {
                    name: "execute_shell".to_string(),
                    step_type: "script_execution".to_string(),
                    input: serde_json::json!({"code_length": code.len()}),
                    output: serde_json::json!({
                        "exit_code": output.status.code(),
                        "stdout_len": stdout.len()
                    }),
                    execution_time_ms: elapsed,
                    timestamp: chrono::Utc::now().timestamp(),
                }
            ],
            error: if output.status.success() { None } else { Some(stderr) },
            metrics: std::collections::HashMap::new(),
        })
    }
}

#[async_trait]
impl SkillExecutor for ScriptExecutor {
    async fn execute(&self, context: &SkillExecutionContext) -> Result<SkillExecutionResult, String> {
        use super::manifest::SkillImplementation;

        // 提取脚本代码
        let code = match &self.manifest.implementation {
            SkillImplementation::Script { code, .. } => code.clone(),
            _ => return Err("Not a script skill".to_string()),
        };

        // 根据语言选择执行方式
        match self.language.as_str() {
            "javascript" | "js" => self.execute_javascript(&code, context).await,
            "python" | "py" => self.execute_python(&code, context).await,
            "shell" | "sh" | "bash" => self.execute_shell(&code, context).await,
            _ => Err(format!("Unsupported script language: {}", self.language)),
        }
    }
}
