//! Kapos Loop Commands
//!
//! 卡帕斯循环命令 - Tauri 命令接口
//! 测试 → 分析 → 修复 → 验证 → 循环

use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;
use chrono::Utc;
use log::{info, warn, error};

/// 卡帕斯循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaposLoopConfig {
    pub max_iterations: u32,
    pub test_timeout_ms: u64,
    pub test_command: String,
    pub auto_rollback: bool,
    pub auto_commit: bool,
    pub strict_mode: bool,
    pub project_path: Option<String>,
}

impl Default for KaposLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            test_timeout_ms: 60000,
            test_command: "cargo test".to_string(),
            auto_rollback: true,
            auto_commit: false,
            strict_mode: false,
            project_path: None,
        }
    }
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub passed: u32,
    pub failed: u32,
    pub total: u32,
    pub output: String,
    pub compiles: bool,
    pub compilation_errors: u32,
}

/// 文件变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file: String,
    pub old_lines: u32,
    pub new_lines: u32,
    pub change_type: String,
}

/// 卡帕斯循环状态
pub struct KaposLoopState {
    pub is_running: bool,
    pub current_iteration: u32,
    pub max_iterations: u32,
    pub iterations_history: Vec<KaposIterationHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaposIterationHistory {
    pub iteration: u32,
    pub hypothesis: String,
    pub test_before: TestResult,
    pub test_after: TestResult,
    pub success: bool,
    pub reflection: String,
}

/// 卡帕斯循环结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaposLoopResult {
    pub success: bool,
    pub iterations: u32,
    pub final_state: String,
    pub changes: Vec<FileChange>,
    pub test_result: TestResult,
    pub error: Option<String>,
}

/// 运行卡帕斯循环
#[tauri::command]
pub async fn run_kapos_loop(
    target_file: String,
    prompt: String,
    config: Option<KaposLoopState>,
    state: State<'_, Arc<Mutex<KaposLoopState>>>,
) -> Result<KaposLoopResult, String> {
    let loop_state = state.inner().lock().await;
    
    let cfg = KaposLoopConfig::default();
    let mut iterations_history = Vec::new();
    let mut all_changes = Vec::new();
    let mut last_test_result = TestResult {
        passed: 0,
        failed: 0,
        total: 0,
        output: String::new(),
        compiles: true,
        compilation_errors: 0,
    };

    info!("[KaposLoop] 开始循环 - 目标: {}", target_file);
    
    for i in 0..cfg.max_iterations {
        let iteration = i + 1;
        info!("[KaposLoop] 迭代 {}/{}", iteration, cfg.max_iterations);
        
        // 1. 获取基线测试
        let test_before = run_tests(&cfg.test_command, cfg.project_path.as_deref())
            .unwrap_or(TestResult {
                passed: 0,
                failed: 0,
                total: 0,
                output: String::new(),
                compiles: false,
                compilation_errors: 1,
            });
        
        // 2. 分析并生成修复 (调用 LLM)
        let hypothesis = format!("迭代 {}: 分析代码并尝试修复", iteration);
        
        // 3. 应用修复 (这里需要集成 LLM 生成修复)
        // 实际实现中应该调用 AI 生成修复并应用到文件
        
        // 4. 检查编译
        let compiles = check_compile(&cfg.test_command, cfg.project_path.as_deref())
            .unwrap_or(false);
        
        // 5. 运行测试
        let test_after = run_tests(&cfg.test_command, cfg.project_path.as_deref())
            .unwrap_or(test_before.clone());
        
        // 6. 验证结果
        let success = if cfg.strict_mode {
            test_after.passed == test_after.total && test_after.total > 0
        } else {
            test_after.passed >= test_before.passed && compiles
        };
        
        // 记录迭代历史
        iterations_history.push(KaposIterationHistory {
            iteration,
            hypothesis: hypothesis.clone(),
            test_before: test_before.clone(),
            test_after: test_after.clone(),
            success,
            reflection: if success { "验证通过".to_string() } else { "需要继续迭代".to_string() },
        });
        
        last_test_result = test_after;
        
        if success && compiles {
            info!("[KaposLoop] 迭代 {} 成功!", iteration);
            return KaposLoopResult {
                success: true,
                iterations: iteration,
                final_state: "passed".to_string(),
                changes: all_changes,
                test_result: last_test_result,
                error: None,
            };
        } else {
            warn!("[KaposLoop] 迭代 {} 失败，继续...", iteration);
            
            // 如果需要回滚
            if cfg.auto_rollback {
                let _ = git_checkout(&target_file);
            }
        }
    }
    
    info!("[KaposLoop] 达到最大迭代次数");
    
    Ok(KaposLoopResult {
        success: false,
        iterations: cfg.max_iterations,
        final_state: "failed".to_string(),
        changes: all_changes,
        test_result: last_test_result,
        error: Some("达到最大迭代次数".to_string()),
    })
}

/// 运行测试
fn run_tests(command: &str, cwd: Option<&str>) -> Result<TestResult, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd.unwrap_or("."))
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_str = format!("{}\n{}", stdout, stderr);
    
    // 解析测试结果 (简单解析)
    let passed = count_matches(&output_str, "test result: ok")
        + count_matches(&output_str, "running");
    let failed = count_matches(&output_str, "test result: FAILED");
    let total = passed + failed;
    
    Ok(TestResult {
        passed,
        failed,
        total,
        output: output_str,
        compiles: output.status.success(),
        compilation_errors: if output.status.success() { 0 } else { 1 },
    })
}

/// 检查编译
fn check_compile(command: &str, cwd: Option<&str>) -> Result<bool, String> {
    let check_cmd = command.replace("test", "check");
    let output = Command::new("sh")
        .arg("-c")
        .arg(&check_cmd)
        .current_dir(cwd.unwrap_or("."))
        .output()
        .map_err(|e| e.to_string())?;
    
    Ok(output.status.success())
}

/// Git 回滚
fn git_checkout(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["checkout", path])
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        Ok(())
    } else {
        Err("回滚失败".to_string())
    }
}

/// 统计字符串匹配次数
fn count_matches(s: &str, pattern: &str) -> u32 {
    s.matches(pattern).count() as u32
}

/// 获取卡帕斯循环状态
#[tauri::command]
pub fn get_kapos_loop_status(
    state: State<'_, Arc<Mutex<KaposLoopState>>>,
) -> Result<serde_json::Value, String> {
    let loop_state = state.inner().lock().await;
    
    Ok(json!({
        "is_running": loop_state.is_running,
        "current_iteration": loop_state.current_iteration,
        "max_iterations": loop_state.max_iterations,
        "iterations_history": loop_state.iterations_history,
    }))
}

/// 停止卡帕斯循环
#[tauri::command]
pub async fn stop_kapos_loop(
    state: State<'_, Arc<Mutex<KaposLoopState>>>,
) -> Result<serde_json::Value, String> {
    let mut loop_state = state.inner().lock().await;
    loop_state.is_running = false;
    
    Ok(json!({
        "success": true,
        "message": "卡帕斯循环已停止"
    }))
}

/// 创建新的卡帕斯循环状态
pub fn create_kapos_loop_state() -> Arc<Mutex<KaposLoopState>> {
    Arc::new(Mutex::new(KaposLoopState {
        is_running: false,
        current_iteration: 0,
        max_iterations: 5,
        iterations_history: Vec::new(),
    }))
}