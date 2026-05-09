//! 技能存储管理
//!
//! 提供技能的持久化存储和查询功能

use super::manifest::{SkillManifest, SkillSearchParams, SkillCategory, SkillSource};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// 技能存储
pub struct SkillStorage {
    /// 内存中的技能缓存
    skills: RwLock<HashMap<String, SkillManifest>>,
    /// 存储路径
    storage_path: PathBuf,
    /// 是否已初始化
    initialized: RwLock<bool>,
}

/// 技能存储数据 (用于序列化)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillStorageData {
    version: String,
    skills: Vec<SkillManifest>,
}

impl SkillStorage {
    /// 创建新的技能存储
    pub async fn new(storage_path: PathBuf) -> Result<Self, String> {
        let storage = Self {
            skills: RwLock::new(HashMap::new()),
            storage_path,
            initialized: RwLock::new(false),
        };

        // 确保存储目录存在
        if let Some(parent) = storage.storage_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        // 加载已存在的技能
        storage.load().await?;

        Ok(storage)
    }

    /// 初始化存储 (加载内置技能)
    pub async fn initialize(&self) -> Result<(), String> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        // 添加系统内置技能
        self.initialize_builtin_skills().await?;

        *initialized = true;
        Ok(())
    }

    /// 保存单个技能
    pub async fn save(&self, skill: &SkillManifest) -> Result<(), String> {
        // 更新内存缓存
        {
            let mut skills = self.skills.write().await;
            skills.insert(skill.id.clone(), skill.clone());
        }

        // 持久化到文件
        self.persist().await?;

        Ok(())
    }

    /// 批量保存技能
    pub async fn save_many(&self, skills: &[SkillManifest]) -> Result<(), String> {
        {
            let mut skills_map = self.skills.write().await;
            for skill in skills {
                skills_map.insert(skill.id.clone(), skill.clone());
            }
        }

        self.persist().await?;
        Ok(())
    }

    /// 获取单个技能
    pub async fn get(&self, skill_id: &str) -> Option<SkillManifest> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// 删除技能
    pub async fn delete(&self, skill_id: &str) -> Result<bool, String> {
        {
            let mut skills = self.skills.write().await;
            if skills.remove(skill_id).is_none() {
                return Ok(false);
            }
        }

        self.persist().await?;
        Ok(true)
    }

    /// 列出所有技能
    pub async fn list_all(&self) -> Vec<SkillManifest> {
        let skills = self.skills.read().await;
        skills.values().cloned().collect()
    }

    /// 搜索技能
    pub async fn search(&self, params: &SkillSearchParams) -> Vec<SkillManifest> {
        let skills = self.skills.read().await;
        
        let mut results: Vec<SkillManifest> = skills
            .values()
            .filter(|skill| {
                // 只显示启用的技能
                if params.enabled_only && !skill.enabled {
                    return false;
                }

                // 类别过滤
                if let Some(ref category) = params.category {
                    if skill.category != *category {
                        return false;
                    }
                }

                // 来源过滤
                if let Some(ref source) = params.source {
                    if skill.source != *source {
                        return false;
                    }
                }

                // 标签过滤
                if let Some(ref tags) = params.tags {
                    let has_all_tags = tags.iter().all(|tag| skill.tags.contains(tag));
                    if !has_all_tags {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 如果有查询词，按相关性排序
        if !params.query.is_empty() {
            results.sort_by(|a, b| {
                let score_a = a.relevance_score(&params.query);
                let score_b = b.relevance_score(&params.query);
                score_b.cmp(&score_a) // 降序排列
            });

            // 过滤掉相关性为0的
            results.retain(|s| s.relevance_score(&params.query) > 0);
        }

        results
    }

    /// 按类别获取技能
    pub async fn get_by_category(&self, category: SkillCategory) -> Vec<SkillManifest> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|s| s.category == category && s.enabled)
            .cloned()
            .collect()
    }

    /// 获取所有类别
    pub async fn get_categories(&self) -> Vec<(SkillCategory, usize)> {
        let skills = self.skills.read().await;
        let mut counts: HashMap<SkillCategory, usize> = HashMap::new();

        for skill in skills.values() {
            if skill.enabled {
                *counts.entry(skill.category).or_insert(0) += 1;
            }
        }

        counts.into_iter().collect()
    }

    /// 更新技能
    pub async fn update(&self, skill_id: &str, updates: super::manifest::SkillUpdate) -> Result<SkillManifest, String> {
        let mut skills = self.skills.write().await;
        
        let skill = skills.get_mut(skill_id)
            .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

        skill.update(updates);
        let updated = skill.clone();
        
        drop(skills);
        self.persist().await?;
        
        Ok(updated)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> SkillStorageStats {
        let skills = self.skills.read().await;
        
        let total = skills.len();
        let enabled = skills.values().filter(|s| s.enabled).count();
        let builtin = skills.values().filter(|s| matches!(s.source, SkillSource::System)).count();
        let user_created = skills.values().filter(|s| matches!(s.source, SkillSource::UserCreated)).count();

        let mut categories: HashMap<String, usize> = HashMap::new();
        for skill in skills.values() {
            if skill.enabled {
                *categories.entry(skill.category.as_str().to_string()).or_insert(0) += 1;
            }
        }

        SkillStorageStats {
            total,
            enabled,
            disabled: total - enabled,
            builtin,
            user_created,
            categories,
        }
    }

    /// 持久化到文件
    async fn persist(&self) -> Result<(), String> {
        let skills = self.skills.read().await;
        let data = SkillStorageData {
            version: "1.0".to_string(),
            skills: skills.values().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize skills: {}", e))?;

        // 写入临时文件，然后原子重命名
        let temp_path = self.storage_path.with_extension("tmp");
        tokio::fs::write(&temp_path, json).await
            .map_err(|e| format!("Failed to write skills file: {}", e))?;

        tokio::fs::rename(&temp_path, &self.storage_path).await
            .map_err(|e| format!("Failed to rename skills file: {}", e))?;

        Ok(())
    }

    /// 从文件加载
    async fn load(&self) -> Result<(), String> {
        // 检查文件是否存在
        if !self.storage_path.exists() {
            return Ok(());
        }

        let json = tokio::fs::read_to_string(&self.storage_path).await
            .map_err(|e| format!("Failed to read skills file: {}", e))?;

        let data: SkillStorageData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse skills file: {}", e))?;

        let mut skills = self.skills.write().await;
        skills.clear();
        
        for skill in data.skills {
            skills.insert(skill.id.clone(), skill);
        }

        Ok(())
    }

    /// 初始化内置技能
    async fn initialize_builtin_skills(&self) -> Result<(), String> {
        let builtin_skills = vec![
            SkillManifest {
                id: "skill_text_summarizer".to_string(),
                display_name: "文本摘要".to_string(),
                description: "对长文本进行智能摘要，提取关键信息".to_string(),
                category: SkillCategory::TextProcessing,
                version: "1.0.0".to_string(),
                implementation: super::manifest::SkillImplementation::Builtin {
                    handler: "text_summarizer".to_string(),
                },
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "需要摘要的文本"},
                        "max_length": {"type": "integer", "description": "最大摘要长度"}
                    },
                    "required": ["text"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "summary": {"type": "string"},
                        "original_length": {"type": "integer"},
                        "summary_length": {"type": "integer"}
                    }
                }),
                source: SkillSource::System,
                author: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                tags: vec!["text".to_string(), "ai".to_string()],
                enabled: true,
            },
            SkillManifest {
                id: "skill_code_formatter".to_string(),
                display_name: "代码格式化".to_string(),
                description: "对代码进行格式化和美化".to_string(),
                category: SkillCategory::CodeProcessing,
                version: "1.0.0".to_string(),
                implementation: super::manifest::SkillImplementation::Builtin {
                    handler: "code_formatter".to_string(),
                },
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string"},
                        "language": {"type": "string"}
                    },
                    "required": ["code", "language"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "formatted_code": {"type": "string"}
                    }
                }),
                source: SkillSource::System,
                author: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                tags: vec!["code".to_string(), "format".to_string()],
                enabled: true,
            },
            SkillManifest {
                id: "skill_data_validator".to_string(),
                display_name: "数据验证".to_string(),
                description: "验证数据是否符合指定模式".to_string(),
                category: SkillCategory::DataProcessing,
                version: "1.0.0".to_string(),
                implementation: super::manifest::SkillImplementation::Builtin {
                    handler: "data_validator".to_string(),
                },
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {},
                        "schema": {"type": "object"}
                    },
                    "required": ["data", "schema"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "valid": {"type": "boolean"},
                        "errors": {"type": "array"}
                    }
                }),
                source: SkillSource::System,
                author: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                tags: vec!["data".to_string(), "validation".to_string()],
                enabled: true,
            },
            SkillManifest {
                id: "skill_file_analyzer".to_string(),
                display_name: "文件分析".to_string(),
                description: "读取和分析文件内容".to_string(),
                category: SkillCategory::FileProcessing,
                version: "1.0.0".to_string(),
                implementation: super::manifest::SkillImplementation::Builtin {
                    handler: "file_analyzer".to_string(),
                },
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "operation": {"type": "string", "enum": ["read", "analyze"]}
                    },
                    "required": ["file_path"]
                }),
                output_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string"},
                        "analysis": {"type": "object"}
                    }
                }),
                source: SkillSource::System,
                author: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                tags: vec!["file".to_string(), "analysis".to_string()],
                enabled: true,
            },
        ];

        // 检查是否已存在
        let mut skills = self.skills.write().await;
        for skill in builtin_skills {
            if !skills.contains_key(&skill.id) {
                skills.insert(skill.id.clone(), skill);
            }
        }
        drop(skills);

        self.persist().await?;
        Ok(())
    }
}

/// 存储统计信息
#[derive(Debug, Clone)]
pub struct SkillStorageStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub builtin: usize,
    pub user_created: usize,
    pub categories: HashMap<String, usize>,
}

impl SkillStorageStats {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total": self.total,
            "enabled": self.enabled,
            "disabled": self.disabled,
            "builtin": self.builtin,
            "user_created": self.user_created,
            "categories": self.categories,
        })
    }
}
