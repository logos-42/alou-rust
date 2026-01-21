//! 调研模块
//!
//! 实现MD文档、prompt和skills的AI增强调研功能

use super::super::AsyncWorkflowExecutor;
use super::super::*;
use crate::prompts::{PromptManager, PromptContext, PROMPT_MANAGER};

impl AsyncWorkflowExecutor {
    /// AI增强的文档调研（简化版，控制token消耗）
    pub async fn research_documentation_with_ai(
        &self,
        query: &str,
        api_key: &str,
    ) -> Result<serde_json::Value, String> {
        println!("📚 [AI-RESEARCH] Starting AI-enhanced research for: {}", query);

        // 1. 基础文档搜索
        let base_results = self.research_documentation(query).await?;

        // 2. AI分析和总结（控制token）
        let ai_summary = self.ai_summarize_research(&base_results, api_key).await?;

        // 3. 生成简化文档
        let simplified_docs = self.generate_simplified_docs(&base_results, &ai_summary, api_key).await?;

        let enhanced_results = serde_json::json!({
            "query": query,
            "timestamp": chrono::Utc::now().timestamp(),
            "base_findings": base_results,
            "ai_summary": ai_summary,
            "simplified_docs": simplified_docs,
            "token_optimized": true
        });

        println!("✅ [AI-RESEARCH] AI-enhanced research completed");
        Ok(enhanced_results)
    }

    /// MD文档和prompt调研
    async fn research_documentation(&self, query: &str) -> Result<serde_json::Value, String> {
        println!("📚 [RESEARCH] Researching documentation for: {}", query);

        let mut research_results = serde_json::json!({
            "query": query,
            "findings": [],
            "prompts": [],
            "skills": []
        });

        // 搜索MD文档
        if let Ok(md_files) = self.search_markdown_files(query).await {
            research_results["findings"] = serde_json::Value::Array(md_files);
        }

        // 搜索相关prompts
        if let Ok(prompts) = self.search_prompts(query).await {
            research_results["prompts"] = serde_json::Value::Array(prompts);
        }

        // 搜索相关skills
        if let Ok(skills) = self.search_skills(query).await {
            research_results["skills"] = serde_json::Value::Array(skills);
        }

        println!("📊 [RESEARCH] Found {} findings, {} prompts, {} skills", 
            research_results["findings"].as_array().unwrap().len(),
            research_results["prompts"].as_array().unwrap().len(),
            research_results["skills"].as_array().unwrap().len()
        );

        Ok(research_results)
    }

    /// 搜索MD文档
    async fn search_markdown_files(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let search_request = crate::bridges::ToolCallRequest {
            session_id: "ralph_loop_research".to_string(),
            user_id: None,
            tool_id: "search".to_string(),
            args: serde_json::json!({
                "query": query,
                "file_types": ["md"],
                "max_results": 10
            }),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string()],
        };

        match self.bridge_manager.tool_bridge().handle_request(search_request).await {
            Ok(response) => {
                if response.success {
                    if let Some(result) = response.result {
                        Ok(result.data.as_array()
                            .unwrap_or(&vec![])
                            .clone())
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Err(response.error.unwrap_or_else(|| "搜索失败".to_string()))
                }
            }
            Err(e) => Err(format!("搜索错误: {}", e))
        }
    }

    /// 搜索相关prompts
    async fn search_prompts(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        // 这里可以实现prompt搜索逻辑
        // 暂时返回空数组
        Ok(vec![])
    }

    /// 搜索相关skills
    async fn search_skills(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let skills_request = crate::bridges::ToolCallRequest {
            session_id: "ralph_loop_skills".to_string(),
            user_id: None,
            tool_id: "skills".to_string(),
            args: serde_json::json!({
                "action": "search",
                "query": query
            }),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(30),
            permissions: vec!["read".to_string()],
        };

        match self.bridge_manager.tool_bridge().handle_request(skills_request).await {
            Ok(response) => {
                if response.success {
                    if let Some(result) = response.result {
                        Ok(result.data.as_array()
                            .unwrap_or(&vec![])
                            .clone())
                    } else {
                        Ok(vec![])
                    }
                } else {
                    Err(response.error.unwrap_or_else(|| "Skills搜索失败".to_string()))
                }
            }
            Err(e) => Err(format!("Skills搜索错误: {}", e))
        }
    }

    /// AI总结调研结果（简化版，控制token）
    async fn ai_summarize_research(
        &self,
        research_results: &serde_json::Value,
        api_key: &str,
    ) -> Result<String, String> {
        let findings_count = research_results["findings"].as_array().map(|a| a.len()).unwrap_or(0);
        let prompts_count = research_results["prompts"].as_array().map(|a| a.len()).unwrap_or(0);
        let skills_count = research_results["skills"].as_array().map(|a| a.len()).unwrap_or(0);

        // 构建Prompt上下文
        let context = PromptContext {
            findings: Some(findings_count),
            prompts: Some(prompts_count),
            skills: Some(skills_count),
            ..Default::default()
        };

        // 使用统一Prompt管理器生成调研总结Prompt
        let summary_prompt = PROMPT_MANAGER.generate_workflow_prompt("research_summary", &context).await;

        self.call_ai_for_decision(&summary_prompt, api_key).await
    }

    /// 生成简化文档（控制token消耗）
    async fn generate_simplified_docs(
        &self,
        research_results: &serde_json::Value,
        ai_summary: &str,
        api_key: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut simplified_docs = Vec::new();

        // 处理文档发现（只取前3个最重要的）
        if let Some(findings) = research_results["findings"].as_array() {
            for (i, finding) in findings.iter().take(3).enumerate() {
                let doc_prompt = format!(
                    r#"
将以下文档内容简化为100字以内的要点：

{}

格式：标题: 要点内容
"#,
                    serde_json::to_string(finding).unwrap_or_default()
                );

                if let Ok(simplified) = self.call_ai_for_decision(&doc_prompt, api_key).await {
                    simplified_docs.push(serde_json::json!({
                        "type": "document",
                        "index": i,
                        "original": finding,
                        "simplified": simplified,
                        "token_count": simplified.len()
                    }));
                }
            }
        }

        Ok(simplified_docs)
    }

    /// 记录调研结果到历史
    pub async fn record_research_to_history(
        &self,
        execution_id: &str,
        research_results: &serde_json::Value,
    ) {
        println!("📝 [RESEARCH-HISTORY] Recording research results to history");

        // 创建调研记录
        let research_record = serde_json::json!({
            "execution_id": execution_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "research_results": research_results,
            "progress": "research_completed"
        });

        // 这里可以保存到持久化存储或内存中
        // 暂时只记录日志
        println!("✅ [RESEARCH-HISTORY] Research recorded: {}", 
            serde_json::to_string_pretty(&research_record).unwrap_or_default());
    }

    /// 总结调研结果（用于循环集成）
    pub async fn summarize_research_results(&self, research_results: &serde_json::Value) -> String {
        if let Some(summary) = research_results.get("ai_summary") {
            summary.as_str().unwrap_or("调研完成").to_string()
        } else {
            "调研完成，无AI总结".to_string()
        }
    }
}
