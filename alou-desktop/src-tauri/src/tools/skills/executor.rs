//! Skills 执行逻辑

use super::super::SkillsTool;
use super::types::*;
use crate::skills::{SkillCategory, SkillImplementation};
use crate::skills::executor::{SkillExecutionContext, SkillExecutorFactory};

/// 创建 Agent Skill
pub async fn create_agent_skill(
    tool: &SkillsTool,
    req: CreateSkillRequest,
) -> Result<crate::skills::SkillManifest, String> {
    use crate::skills::manifest::{SkillManifest, SkillSource};

    let category = parse_category(&req.category);

    let implementation = SkillImplementation::AgentSkill {
        persona: req.persona.unwrap_or_default(),
        capabilities: req.capabilities.unwrap_or_default(),
        constraints: req.constraints.unwrap_or_default(),
        examples: vec![],
        model: None,
    };

    let mut skill = SkillManifest::new(
        req.display_name,
        req.description,
        category,
        implementation,
    );

    if let Some(tags) = req.tags {
        skill.tags = tags;
    }
    skill.source = SkillSource::UserCreated;

    tool.storage().save(&skill).await?;
    Ok(skill)
}

/// 搜索技能
pub async fn search_skills(
    tool: &SkillsTool,
    req: SearchSkillsRequest,
) -> Vec<crate::skills::SkillManifest> {
    use crate::skills::SkillSearchParams;

    let category_filter = req.category.and_then(|c| parse_category_opt(&c));

    let params = SkillSearchParams {
        query: req.query,
        category: category_filter,
        tags: None,
        source: None,
        enabled_only: true,
    };

    tool.storage().search(&params).await
}

/// 执行技能
pub async fn execute_skill(
    tool: &SkillsTool,
    req: ExecuteSkillRequest,
    session_id: &str,
) -> Result<crate::skills::executor::SkillExecutionResult, String> {
    // 获取技能定义
    let skill = tool.storage().get(&req.skill_id).await
        .ok_or_else(|| format!("Skill '{}' not found", req.skill_id))?;

    // 创建执行上下文
    let context = SkillExecutionContext::new(
        session_id.to_string(),
        req.skill_id
    ).with_inputs(req.inputs);

    // 创建执行器并执行
    let executor = SkillExecutorFactory::create(&skill)?;
    let result = executor.execute(&context).await?;

    Ok(result)
}

/// 获取技能详情
pub async fn get_skill_detail(
    tool: &SkillsTool,
    skill_id: &str,
) -> Option<crate::skills::SkillManifest> {
    tool.storage().get(skill_id).await
}

/// 列出所有技能
pub async fn list_skills(
    tool: &SkillsTool,
    category: Option<&str>,
) -> Vec<crate::skills::SkillManifest> {
    if let Some(cat) = category {
        if let Some(cat_enum) = parse_category_opt(cat) {
            return tool.storage().get_by_category(cat_enum).await;
        }
    }
    tool.storage().list_all().await
}

/// 获取统计信息
pub async fn get_stats(tool: &SkillsTool) -> serde_json::Value {
    tool.storage().get_stats().await.to_json()
}

/// 删除技能
pub async fn delete_skill(tool: &SkillsTool, skill_id: &str) -> Result<bool, String> {
    tool.storage().delete(skill_id).await
}

// 辅助函数

fn parse_category(category: &str) -> SkillCategory {
    parse_category_opt(category).unwrap_or(SkillCategory::Other)
}

fn parse_category_opt(category: &str) -> Option<SkillCategory> {
    match category {
        "text_processing" => Some(SkillCategory::TextProcessing),
        "code_processing" => Some(SkillCategory::CodeProcessing),
        "data_processing" => Some(SkillCategory::DataProcessing),
        "file_processing" => Some(SkillCategory::FileProcessing),
        "automation" => Some(SkillCategory::Automation),
        "agent" => Some(SkillCategory::Agent),
        "tool_chain" => Some(SkillCategory::ToolChain),
        _ => None,
    }
}
