//! 自动研究模式 - Karpathy 风格，进化"系统自身的代码"

use anyhow::{Result, Context};
use tracing::info;
use std::path::PathBuf;

use hyperagent::{
    AutoResearch, ResearchConfig,
    LLMClientImpl, LLMConfig, LLMProvider,
};

use super::config::KaizenConfig;
use super::progress::KaizenProgress;

/// 运行自动研究循环
pub async fn run_research(config: &KaizenConfig) -> Result<()> {
    info!("🔬 启动 Kaizen 自动研究 (Karpathy 模式)");
    
    // 初始化进度追踪
    let mut progress = KaizenProgress::new("research", config.max_iterations);
    progress.start();
    progress.save()?;
    
    // 确定项目根目录（默认当前目录）
    let project_root = std::env::current_dir()
        .context("无法获取当前工作目录")?;
    
    info!("📁 项目根目录: {}", project_root.display());
    
    // 创建研究配置
    let research_config = ResearchConfig {
        project_root: project_root.clone(),
        target_files: if config.target_files.is_empty() {
            // 默认目标：hyperagent 的源文件
            vec![
                "lib.rs".to_string(),
                "auto_research/mod.rs".to_string(),
                "runtime/loop_.rs".to_string(),
            ]
        } else {
            config.target_files.clone()
        },
        max_iterations: config.max_iterations as u32,
        auto_push: config.auto_push,
        dry_run: config.dry_run,
        strict: config.strict,
        enable_web: config.enable_web_search,
        push_interval: config.push_interval as u32,
        ..Default::default()
    };
    
    info!("🎯 目标文件: {:?}", research_config.target_files);
    info!("⚙️  配置: 最大迭代={}, 自动推送={}, 安全模式={}, 严格模式={}",
        research_config.max_iterations,
        research_config.auto_push,
        research_config.dry_run,
        research_config.strict,
    );
    
    // 创建 LLM 客户端
    let llm_config = LLMConfig {
        provider: match config.llm_provider.as_str() {
            "openai" => LLMProvider::OpenAI,
            "ollama" => LLMProvider::Ollama,
            "qwen" => LLMProvider::Qwen,
            _ => LLMProvider::OpenAI,
        },
        model: config.llm_model.clone(),
        api_key: config.llm_api_key.clone(),
        base_url: config.llm_base_url.clone(),
        max_concurrent: 8,
        temperature: Some(0.7),
        max_tokens: Some(2000),
    };
    
    let client = LLMClientImpl::from_config(&llm_config)?;
    info!("📡 使用 LLM: {:?}, 模型: {}", client.provider(), client.model());
    
    // 创建自动研究引擎
    let mut research_engine = AutoResearch::new(client, research_config);
    
    // 运行研究循环
    info!("🔄 开始研究循环...");
    
    let experiments = research_engine.run()
        .await
        .context("研究循环执行失败")?;
    
    // 更新进度
    let success_count = experiments.iter().filter(|e| e.is_improvement()).count();
    let failure_count = experiments.iter().filter(|e| e.is_failed() || e.is_regressed()).count();
    
    for (i, exp) in experiments.iter().enumerate() {
        let score = exp.multi_eval.as_ref().map(|m| m.score).unwrap_or(0.0);
        let outcome = match exp.outcome {
            hyperagent::auto_research::ExperimentOutcome::Improved => "improved",
            hyperagent::auto_research::ExperimentOutcome::Failed => "failed",
            hyperagent::auto_research::ExperimentOutcome::Regressed => "regressed",
            _ => "neutral",
        };
        progress.update_iteration(i + 1, outcome, score);
    }
    
    progress.stop();
    progress.save()?;
    
    // 输出结果
    info!("✅ 研究完成! 共 {} 次实验", experiments.len());
    info!("📊 统计:");
    info!("   ✅ 改进: {}", success_count);
    info!("   ❌ 失败: {}", failure_count);
    info!("   ➖ 中性: {}", experiments.len() - success_count - failure_count);
    
    // 显示最近 5 次实验
    info!("\n📝 最近实验记录:");
    for exp in experiments.iter().rev().take(5) {
        info!("   {}", exp.summary());
    }
    
    Ok(())
}

/// 查看研究状态
pub fn show_research_status() -> Result<()> {
    let progress = KaizenProgress::load()?;
    println!("{}", progress.summary());
    
    // 尝试加载实验日志
    let project_root = std::env::current_dir()?;
    let log_path = project_root.join(".hyperagent/experiments/research_log.md");
    
    if log_path.exists() {
        println!("\n📝 实验日志 (最近 20 行):");
        let content = std::fs::read_to_string(&log_path)?;
        let lines: Vec<&str> = content.lines().collect();
        for line in lines.iter().rev().take(20).rev() {
            println!("   {}", line);
        }
    }
    
    Ok(())
}

/// 停止研究循环
pub fn stop_research() -> Result<()> {
    let mut progress = KaizenProgress::load()?;
    
    if progress.is_running {
        progress.stop();
        progress.save()?;
        info!("🛑 研究循环已停止");
    } else {
        info!("⚠️  研究循环未运行");
    }
    
    Ok(())
}

/// 查看实验日志
pub fn show_research_log(last_n: usize) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let log_path = project_root.join(".hyperagent/experiments/research_log.md");
    
    if !log_path.exists() {
        info!("⚠️  实验日志不存在");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    info!("📝 实验日志 (最近 {} 行):", last_n);
    println!();
    for line in lines.iter().rev().take(last_n).rev() {
        println!("{}", line);
    }
    
    Ok(())
}
