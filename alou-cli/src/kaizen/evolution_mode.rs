//! 进化引擎模式 - 进化"解决任务的代码"

use anyhow::{Result, Context};
use tracing::info;

use hyperagent::{
    EvolutionLoop, RuntimeState, RuntimeConfig,
    LLMClientImpl, LLMConfig, LLMProvider,
};

use super::config::KaizenConfig;
use super::progress::KaizenProgress;

/// 运行进化引擎
pub async fn run_evolution(config: &KaizenConfig) -> Result<()> {
    info!("🚀 启动 Kaizen 进化引擎");
    
    // 初始化进度追踪
    let mut progress = KaizenProgress::new("evolution", config.max_iterations);
    progress.start();
    progress.save()?;
    
    // 创建 LLM 客户端
    let llm_config = LLMConfig {
        provider: match config.llm_provider.as_str() {
            "openai" => LLMProvider::OpenAI,
            "ollama" => LLMProvider::Ollama,
            "qwen" => LLMProvider::Qwen,
            "glm" | "zhipuai" => LLMProvider::GLM,
            "minimax" => LLMProvider::MiniMax,
            "deepseek" | "claude" | "gemini" | "openrouter" | "openai-compat" => LLMProvider::OpenAI,
            _ => LLMProvider::OpenAI,
        },
        model: config.llm_model.clone(),
        api_key: config.llm_api_key.clone(),
        base_url: config.llm_base_url.clone().or_else(|| {
            match config.llm_provider.as_str() {
                "deepseek" => Some("https://api.deepseek.com/v1".to_string()),
                "glm" | "zhipuai" => Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
                "minimax" => Some("https://api.minimax.chat/v1".to_string()),
                "qwen" => Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
                "openrouter" => Some("https://openrouter.ai/api/v1".to_string()),
                "ollama" => Some("http://localhost:11434".to_string()),
                _ => None,
            }
        }),
        max_concurrent: if config.llm_provider == "ollama" { 4 } else { 8 },
        temperature: Some(0.7),
        max_tokens: Some(2000),
    };
    
    let client = LLMClientImpl::new(&llm_config)?;
    info!("📡 使用 LLM: {:?}, 模型: {}", client.provider(), client.model());

    // 配置运行时
    let runtime_config = RuntimeConfig {
        max_generations: config.max_iterations as u32,
        population_size: 3,
        top_k_selection: 2,
        checkpoint_interval: 5,
        meta_mutation_interval: 3,
        initial_temperature: 1.5,
        annealing_rate: 0.9,
        mutation_rate: 0.1,
        selection_pressure: 0.3,
        num_branches: 3,
        novelty_weight: 0.5,
        diversity_threshold: 0.8,
    };

    // 持久化目录
    let persist_dir = super::get_hyperagent_data_dir()?;

    // 创建进化循环
    let mut evolution_loop = EvolutionLoop::new(
        client,
        RuntimeState::with_persistence(runtime_config, &persist_dir)
    );

    // 任务描述
    let task = config.task_description.as_deref()
        .unwrap_or("Improve the code quality and performance");

    info!("📋 任务: {}", task);

    // 运行进化循环
    info!("🔄 开始进化循环 ({} 代)...", config.max_iterations);

    let final_state = evolution_loop.run_with_iterations(task, config.max_iterations as usize)
        .await
        .context("进化循环执行失败")?;

    // 更新进度
    progress.update_iteration(
        config.max_iterations,
        "completed",
        final_state.best_score,
    );
    progress.stop();
    progress.save()?;
    
    // 输出结果
    info!("✅ 进化完成!");
    info!("{}", final_state.summary());
    
    if let Some(best_agent) = evolution_loop.get_best_agent() {
        info!("🏆 最佳智能体:");
        info!("   ID: {}", best_agent.id);
        info!("   代数: {}", best_agent.generation);
    }
    
    info!("📊 Archive 大小: {}", final_state.archive.size());
    
    Ok(())
}

/// 查看进化状态
pub fn show_evolution_status() -> Result<()> {
    let progress = KaizenProgress::load()?;
    println!("{}", progress.summary());
    
    // 尝试加载档案信息
    let data_dir = super::get_hyperagent_data_dir()?;
    let archive_path = data_dir.join("archive.json");
    
    if archive_path.exists() {
        let content = std::fs::read_to_string(&archive_path)?;
        let archive: serde_json::Value = serde_json::from_str(&content)?;
        
        println!("\n📦 进化档案:");
        if let Some(size) = archive.get("size").and_then(|v| v.as_u64()) {
            println!("   存档记录数: {}", size);
        }
    }
    
    Ok(())
}

/// 停止进化循环
pub fn stop_evolution() -> Result<()> {
    let mut progress = KaizenProgress::load()?;
    
    if progress.is_running {
        progress.stop();
        progress.save()?;
        info!("🛑 进化循环已停止");
    } else {
        info!("⚠️  进化循环未运行");
    }
    
    Ok(())
}
