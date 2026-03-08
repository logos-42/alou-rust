//! 智能体记忆系统
//!
//! 长期记忆存储和检索，支持自我学习和经验积累

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 记忆类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Conversation,      // 对话记忆
    Preference,         // 用户偏好
    Knowledge,         // 知识积累
    Experience,         // 经验总结
    Feedback,          // 用户反馈
    Pattern,           // 行为模式
}

/// 记忆重要性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Importance {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// 单条记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// 记忆ID
    pub id: String,
    
    /// 记忆类型
    pub memory_type: MemoryType,
    
    /// 记忆内容
    pub content: String,
    
    /// 标签/关键词
    pub tags: Vec<String>,
    
    /// 重要性级别
    pub importance: Importance,
    
    /// 创建时间
    pub created_at: i64,
    
    /// 最后访问时间
    pub last_accessed: i64,
    
    /// 访问次数
    pub access_count: u32,
    
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// 关联的记忆ID
    pub related_memories: Vec<String>,
    
    /// 是否已加密存储
    pub encrypted: bool,
}

/// 智能体记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 最大记忆条数
    pub max_memories: usize,
    
    /// 自动遗忘时间（秒）
    pub auto_forget_seconds: i64,
    
    /// 重要性阈值
    pub importance_threshold: u32,
    
    /// 是否启用长期记忆
    pub enable_long_term: bool,
    
    /// 记忆压缩间隔（秒）
    pub compression_interval: u64,
    
    /// 最大短期记忆条数
    pub short_term_max: usize,
}

/// 用户偏好记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub user_id: String,
    pub preference_key: String,
    pub preference_value: String,
    pub confidence: f64,
    pub last_confirmed: i64,
}

/// 行为模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_id: String,
    pub trigger_context: String,
    pub observed_behavior: String,
    pub frequency: u32,
    pub success_rate: f64,
    pub last_observed: i64,
}

/// 记忆管理器
pub struct MemoryManager {
    /// 短期记忆（内存）
    short_term: Arc<RwLock<Vec<Memory>>>,
    
    /// 长期记忆（磁盘）
    long_term_path: PathBuf,
    
    /// 配置
    config: MemoryConfig,
    
    /// 用户偏好
    preferences: Arc<RwLock<HashMap<String, UserPreference>>>,
    
    /// 行为模式
    patterns: Arc<RwLock<Vec<BehaviorPattern>>>,
}

impl MemoryManager {
    /// 创建新的记忆管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let path = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".alou")
            .join("memory");
        
        // 确保目录存在
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        // 创建子目录
        let long_term_path = path.join("long_term");
        if !long_term_path.exists() {
            fs::create_dir_all(&long_term_path)?;
        }

        Ok(Self {
            short_term: Arc::new(RwLock::new(Vec::new())),
            long_term_path,
            config: MemoryConfig::default(),
            preferences: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 创建新记忆
    pub async fn create_memory(
        &self,
        memory_type: MemoryType,
        content: String,
        tags: Vec<String>,
        importance: Importance,
    ) -> String {
        let id = format!("mem_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string());
        let now = Utc::now().timestamp();
        
        // 检查是否为重要记忆
        let is_important = match importance {
            Importance::Critical | Importance::High => true,
            _ => false,
        };
        
        // 创建记忆的各个部分
        let mem_id = id.clone();
        let mem_type = memory_type.clone();
        let mem_content = content.clone();
        let mem_tags = tags.clone();
        let mem_importance = importance.clone();
        
        let memory = Memory {
            id: mem_id,
            memory_type: mem_type,
            content: mem_content,
            tags: mem_tags,
            importance: mem_importance,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: HashMap::new(),
            related_memories: Vec::new(),
            encrypted: false,
        };
        
        if is_important {
            // 存储到长期记忆
            self.store_long_term(&memory).await;
        } else {
            // 存储到短期记忆
            let mut short = self.short_term.write().await;
            short.push(memory);
            // 保持短期记忆在限制内
            if short.len() > self.config.short_term_max {
                short.remove(0);
            }
        }

        id
    }

    /// 存储到长期记忆
    async fn store_long_term(&self, memory: &Memory) {
        let path = self.long_term_path.join(&format!("{}.json", memory.id));
        let content = serde_json::to_string_pretty(memory).unwrap_or_default();
        let _ = fs::write(&path, content);
    }

    /// 检索记忆
    pub async fn retrieve(
        &self,
        query: &str,
        memory_type: Option<MemoryType>,
        limit: usize,
    ) -> Vec<Memory> {
        let mut results = Vec::new();
        let now = Utc::now().timestamp();

        // 搜索短期记忆
        {
            let short = self.short_term.read().await;
            for mem in short.iter() {
                if self.matches_query(mem, query, &memory_type) {
                    results.push(mem.clone());
                }
            }
        }

        // 搜索长期记忆
        if let Ok(entries) = fs::read_dir(&self.long_term_path) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(mem) = serde_json::from_str::<Memory>(&content) {
                        // 检查是否过期
                        if now - mem.last_accessed > self.config.auto_forget_seconds {
                            // 移除过期记忆
                            let _ = fs::remove_file(entry.path());
                            continue;
                        }

                        if self.matches_query(&mem, query, &memory_type) {
                            results.push(mem);
                        }
                    }
                }
            }
        }

        // 按重要性排序 - 转换为数值进行比较
        results.sort_by(|a, b| {
            let a_val = match a.importance {
                Importance::Low => 1,
                Importance::Medium => 2,
                Importance::High => 3,
                Importance::Critical => 4,
            };
            let b_val = match b.importance {
                Importance::Low => 1,
                Importance::Medium => 2,
                Importance::High => 3,
                Importance::Critical => 4,
            };
            b_val.cmp(&a_val)
        });

        // 更新访问信息
        for mem in results.iter_mut().take(limit) {
            mem.last_accessed = now;
            mem.access_count += 1;
        }

        results.truncate(limit);
        results
    }

    /// 检查记忆是否匹配查询
    fn matches_query(&self, memory: &Memory, query: &str, memory_type: &Option<MemoryType>) -> bool {
        // 检查类型
        if let Some(ref expected_type) = memory_type {
            if &memory.memory_type != expected_type {
                return false;
            }
        }

        // 检查内容或标签
        let query_lower = query.to_lowercase();
        if memory.content.to_lowercase().contains(&query_lower) {
            return true;
        }

        for tag in &memory.tags {
            if tag.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        false
    }

    /// 记录用户偏好
    pub async fn record_preference(
        &self,
        user_id: String,
        key: String,
        value: String,
        confidence: f64,
    ) {
        let pref = UserPreference {
            user_id,
            preference_key: key.clone(),
            preference_value: value,
            confidence,
            last_confirmed: Utc::now().timestamp(),
        };

        let mut prefs = self.preferences.write().await;
        prefs.insert(format!("{}:{}", pref.user_id, key), pref);
    }

    /// 获取用户偏好
    pub async fn get_preference(&self, user_id: &str, key: &str) -> Option<UserPreference> {
        let prefs = self.preferences.read().await;
        prefs.get(&format!("{}:{}", user_id, key)).cloned()
    }

    /// 获取用户所有偏好
    pub async fn get_user_preferences(&self, user_id: &str) -> Vec<UserPreference> {
        let prefs = self.preferences.read().await;
        prefs.iter()
            .filter(|(k, _)| k.starts_with(user_id))
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// 观察并记录行为模式
    pub async fn observe_pattern(
        &self,
        context: &str,
        behavior: &str,
        success: bool,
    ) {
        let mut patterns = self.patterns.write().await;
        
        if let Some(pattern) = patterns.iter_mut().find(|p| p.trigger_context == context && p.observed_behavior == behavior) {
            pattern.frequency += 1;
            if success {
                pattern.success_rate = (pattern.success_rate * (pattern.frequency - 1) as f64 + 1.0) / pattern.frequency as f64;
            }
            pattern.last_observed = Utc::now().timestamp();
        } else {
            patterns.push(BehaviorPattern {
                pattern_id: format!("pat_{}", Uuid::new_v4().to_string().replace("-", "")[..8].to_string()),
                trigger_context: context.to_string(),
                observed_behavior: behavior.to_string(),
                frequency: 1,
                success_rate: if success { 1.0 } else { 0.0 },
                last_observed: Utc::now().timestamp(),
            });
        }
    }

    /// 获取行为模式
    pub async fn get_patterns(&self) -> Vec<BehaviorPattern> {
        let patterns = self.patterns.read().await;
        patterns.clone()
    }

    /// 自我学习：从反馈中学习
    pub async fn learn_from_feedback(
        &self,
        context: &str,
        action: &str,
        feedback: &str, // "positive" 或 "negative"
    ) {
        let success = feedback.to_lowercase() == "positive";
        self.observe_pattern(context, action, success).await;
        
        // 如果是负面反馈，创建一条警告记忆
        if !success {
            let _ = self.create_memory(
                MemoryType::Feedback,
                format!("用户对 '{}' 在 '{}' 上下文中的行为给出负面反馈", action, context),
                vec!["feedback".to_string(), "negative".to_string()],
                Importance::High,
            ).await;
        }
    }

    /// 生成自我总结
    pub async fn generate_summary(&self) -> String {
        let short = self.short_term.read().await;
        let patterns = self.patterns.read().await;
        
        let mut summary = format!("=== 智能体自我总结 ===\n\n");
        
        // 短期记忆统计
        summary += &format!("短期记忆: {} 条\n", short.len());
        
        // 行为模式统计
        let high_success_patterns: Vec<_> = patterns.iter()
            .filter(|p| p.success_rate >= 0.8 && p.frequency >= 3)
            .collect();
        
        summary += &format!("观察到的行为模式: {} 个\n", patterns.len());
        if !high_success_patterns.is_empty() {
            summary += "\n高成功率模式:\n";
            for p in high_success_patterns {
                summary += &format!("- {} → {} (成功率: {:.0}%)\n", 
                    p.trigger_context, p.observed_behavior, p.success_rate * 100.0);
            }
        }

        // 用户偏好
        let prefs = self.preferences.read().await;
        let unique_users: std::collections::HashSet<_> = prefs.values()
            .map(|p| p.user_id.clone())
            .collect();
        summary += &format!("\n记录的用户偏好: {} 个用户的 {} 条偏好\n", 
            unique_users.len(), prefs.len());

        summary
    }

    /// 导出所有记忆
    pub async fn export_all(&self) -> Vec<Memory> {
        let mut all = Vec::new();

        // 短期记忆
        {
            let short = self.short_term.read().await;
            all.extend(short.clone());
        }

        // 长期记忆
        if let Ok(entries) = fs::read_dir(&self.long_term_path) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(mem) = serde_json::from_str::<Memory>(&content) {
                        all.push(mem);
                    }
                }
            }
        }

        all
    }

    /// 清空所有记忆
    pub async fn clear_all(&self) {
        // 清空短期记忆
        {
            let mut short = self.short_term.write().await;
            short.clear();
        }

        // 清空长期记忆
        if let Ok(entries) = fs::read_dir(&self.long_term_path) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }

        // 清空偏好
        {
            let mut prefs = self.preferences.write().await;
            prefs.clear();
        }

        // 清空模式
        {
            let mut patterns = self.patterns.write().await;
            patterns.clear();
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memories: 10000,
            auto_forget_seconds: 30 * 24 * 60 * 60, // 30天
            importance_threshold: 2,
            enable_long_term: true,
            compression_interval: 3600, // 1小时
            short_term_max: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_memory() {
        let manager = MemoryManager::new().unwrap();
        
        let id = manager.create_memory(
            MemoryType::Conversation,
            "用户喜欢简洁的回答".to_string(),
            vec!["user_preference".to_string()],
            Importance::High,
        ).await;
        
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_retrieve_memory() {
        let manager = MemoryManager::new().unwrap();
        
        // 创建记忆
        manager.create_memory(
            MemoryType::Preference,
            "用户喜欢使用表情符号".to_string(),
            vec!["emoji".to_string()],
            Importance::Medium,
        ).await;

        // 检索
        let results = manager.retrieve("emoji", Some(MemoryType::Preference), 10).await;
        
        assert!(!results.is_empty());
        assert!(results[0].content.contains("表情符号"));
    }

    #[tokio::test]
    async fn test_preference_recording() {
        let manager = MemoryManager::new().unwrap();
        
        manager.record_preference(
            "user1".to_string(),
            "language".to_string(),
            "中文".to_string(),
            0.9,
        ).await;
        
        let pref = manager.get_preference("user1", "language").await;
        assert!(pref.is_some());
        assert_eq!(pref.unwrap().preference_value, "中文");
    }

    #[tokio::test]
    async fn test_pattern_observation() {
        let manager = MemoryManager::new().unwrap();
        
        manager.observe_pattern("代码问题", "提供详细解释", true).await;
        manager.observe_pattern("代码问题", "提供详细解释", true).await;
        manager.observe_pattern("代码问题", "提供详细解释", true).await;
        
        let patterns = manager.get_patterns().await;
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].frequency, 3);
        assert_eq!(patterns[0].success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_summary_generation() {
        let manager = MemoryManager::new().unwrap();
        
        // 添加一些数据
        manager.observe_pattern("测试上下文", "测试行为", true).await;
        manager.observe_pattern("测试上下文", "测试行为", true).await;
        manager.record_preference("user1", "key1", "value1", 0.8).await;
        
        let summary = manager.generate_summary().await;
        
        assert!(summary.contains("智能体自我总结"));
        assert!(summary.contains("观察到的行为模式"));
        assert!(summary.contains("记录的用户偏好"));
    }
}
