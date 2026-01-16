//! 上下文压缩器
//!
//! 提供无限循环版本的上下文压缩算法，支持多种压缩策略

use super::{ContextEntry, ContextEntryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 压缩策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 自适应压缩（根据内容类型和重要性）
    Adaptive,
    /// 时间窗口压缩（保留最近的内容）
    TimeWindow,
    /// 重要性优先压缩（保留高重要性内容）
    ImportancePriority,
    /// 摘要压缩（生成内容摘要）
    Summary,
    /// 混合压缩（结合多种策略）
    Hybrid,
}

/// 压缩结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressionResult {
    /// 压缩后的条目
    pub entries: Vec<ContextEntry>,
    /// 原始长度
    pub original_length: usize,
    /// 压缩后长度
    pub compressed_length: usize,
    /// 压缩比例
    pub compression_ratio: f64,
    /// 保留的条目数
    pub retained_entries: usize,
    /// 移除的条目数
    pub removed_entries: usize,
}

/// 上下文压缩器
pub struct ContextCompressor {
    strategy: CompressionStrategy,
}

impl ContextCompressor {
    /// 创建新的压缩器
    pub fn new(strategy: CompressionStrategy) -> Self {
        Self { strategy }
    }

    /// 执行上下文压缩
    pub async fn compress(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
        keep_important_ratio: f64,
    ) -> Result<CompressionResult, Box<dyn std::error::Error>> {
        if entries.is_empty() {
            return Ok(CompressionResult {
                entries: Vec::new(),
                original_length: 0,
                compressed_length: 0,
                compression_ratio: 1.0,
                retained_entries: 0,
                removed_entries: 0,
            });
        }

        let original_length: usize = entries.iter().map(|e| e.content.len()).sum();

        let compressed_entries = match self.strategy {
            CompressionStrategy::Adaptive => {
                self.compress_adaptive(entries, max_length, keep_important_ratio).await?
            }
            CompressionStrategy::TimeWindow => {
                self.compress_time_window(entries, max_length).await?
            }
            CompressionStrategy::ImportancePriority => {
                self.compress_importance_priority(entries, max_length, keep_important_ratio).await?
            }
            CompressionStrategy::Summary => {
                self.compress_summary(entries, max_length).await?
            }
            CompressionStrategy::Hybrid => {
                self.compress_hybrid(entries, max_length, keep_important_ratio).await?
            }
        };

        let compressed_length: usize = compressed_entries.iter().map(|e| e.content.len()).sum();
        let compression_ratio = if original_length > 0 {
            compressed_length as f64 / original_length as f64
        } else {
            1.0
        };

        Ok(CompressionResult {
            entries: compressed_entries,
            original_length,
            compressed_length,
            compression_ratio,
            retained_entries: entries.len(),
            removed_entries: entries.len().saturating_sub(entries.len()),
        })
    }

    /// 自适应压缩（主要策略）
    async fn compress_adaptive(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
        keep_important_ratio: f64,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        let mut compressed = Vec::new();
        let mut current_length = 0;

        // 第一遍：保留高重要性和特殊类型的内容
        for entry in entries.iter().rev() { // 从最新开始
            if current_length >= max_length {
                break;
            }

            // 必须保留的条目类型
            let must_keep = matches!(
                entry.entry_type,
                ContextEntryType::SystemMessage |
                ContextEntryType::ToolCall |
                ContextEntryType::ToolResult
            ) || entry.importance_score >= 0.8;

            if must_keep || entry.content.len() <= 500 { // 小内容优先保留
                if current_length + entry.content.len() <= max_length {
                    compressed.push(entry.clone());
                    current_length += entry.content.len();
                } else {
                    // 内容过长，进行截断
                    let truncated = self.truncate_entry(entry, max_length - current_length);
                    compressed.push(truncated);
                    current_length = max_length;
                }
            }
        }

        // 如果空间还不够，进行第二遍压缩
        if current_length < max_length && compressed.len() < entries.len() {
            compressed = self.compress_secondary(&compressed, entries, max_length).await?;
        }

        // 按原始顺序排序
        compressed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(compressed)
    }

    /// 时间窗口压缩
    async fn compress_time_window(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        let now = chrono::Utc::now().timestamp();
        let mut compressed = Vec::new();
        let mut current_length = 0;

        // 定义时间窗口权重
        let time_weights: Vec<(i64, f64)> = vec![
            (3600, 1.0),    // 1小时内：权重1.0
            (86400, 0.8),   // 1天内：权重0.8
            (604800, 0.6),  // 1周内：权重0.6
            (2592000, 0.4), // 1个月内：权重0.4
        ];

        // 计算每个条目的时间权重
        let mut entries_with_weights: Vec<(f64, &ContextEntry)> = entries
            .iter()
            .map(|entry| {
                let age_seconds = now - entry.timestamp;
                let weight = time_weights
                    .iter()
                    .find(|(threshold, _)| age_seconds <= *threshold)
                    .map(|(_, w)| *w)
                    .unwrap_or(0.2); // 更早的内容权重更低

                (weight, entry)
            })
            .collect();

        // 按权重排序（权重高的优先保留）
        entries_with_weights.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // 按权重选择条目
        for (weight, entry) in entries_with_weights {
            if current_length >= max_length {
                break;
            }

            if current_length + entry.content.len() <= max_length {
                compressed.push((*entry).clone());
                current_length += entry.content.len();
            } else {
                let truncated = self.truncate_entry(entry, max_length - current_length);
                compressed.push(truncated);
                current_length = max_length;
            }
        }

        // 按时间排序
        compressed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(compressed)
    }

    /// 重要性优先压缩
    async fn compress_importance_priority(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
        keep_important_ratio: f64,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        let mut compressed = Vec::new();
        let mut current_length = 0;

        // 按重要性排序
        let mut sorted_entries: Vec<&ContextEntry> = entries.iter().collect();
        sorted_entries.sort_by(|a, b| {
            b.importance_score.partial_cmp(&a.importance_score).unwrap()
        });

        // 计算保留阈值
        let keep_threshold = (entries.len() as f64 * keep_important_ratio) as usize;

        // 优先保留高重要性内容
        for (index, entry) in sorted_entries.iter().enumerate() {
            if current_length >= max_length {
                break;
            }

            // 前N个高重要性内容必须保留
            let must_keep = index < keep_threshold || entry.importance_score >= 0.7;

            if must_keep {
                if current_length + entry.content.len() <= max_length {
                    compressed.push((*entry).clone());
                    current_length += entry.content.len();
                } else {
                    let truncated = self.truncate_entry(entry, max_length - current_length);
                    compressed.push(truncated);
                    current_length = max_length;
                }
            }
        }

        // 按时间排序
        compressed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(compressed)
    }

    /// 摘要压缩
    async fn compress_summary(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        let mut compressed = Vec::new();

        // 按类型分组
        let mut type_groups: HashMap<ContextEntryType, Vec<&ContextEntry>> = HashMap::new();
        for entry in entries {
            type_groups.entry(entry.entry_type).or_insert_with(Vec::new).push(entry);
        }

        // 为每种类型生成摘要
        for (entry_type, group_entries) in type_groups {
            let summary = self.generate_type_summary(entry_type, &group_entries, max_length / 8)?;
            compressed.push(summary);
        }

        // 保留最新的几个原始条目
        let keep_recent = 3;
        let mut recent_entries: Vec<&ContextEntry> = entries.iter()
            .rev()
            .take(keep_recent)
            .collect();
        recent_entries.reverse();

        for entry in recent_entries {
            if compressed.iter().map(|e| e.content.len()).sum::<usize>() + entry.content.len() <= max_length {
                compressed.push(entry.clone());
            }
        }

        compressed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(compressed)
    }

    /// 混合压缩（结合多种策略）
    async fn compress_hybrid(
        &self,
        entries: &[ContextEntry],
        max_length: usize,
        keep_important_ratio: f64,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        // 先使用自适应压缩
        let mut compressed = self.compress_adaptive(entries, max_length, keep_important_ratio).await?;

        // 如果仍然超长，使用时间窗口压缩
        let current_length: usize = compressed.iter().map(|e| e.content.len()).sum();
        if current_length > max_length {
            compressed = self.compress_time_window(&compressed, max_length).await?;
        }

        Ok(compressed)
    }

    /// 二级压缩（当主要压缩不够时使用）
    async fn compress_secondary(
        &self,
        current_entries: &[ContextEntry],
        all_entries: &[ContextEntry],
        max_length: usize,
    ) -> Result<Vec<ContextEntry>, Box<dyn std::error::Error>> {
        let mut compressed = current_entries.to_vec();

        // 计算需要压缩的空间
        let current_length: usize = compressed.iter().map(|e| e.content.len()).sum();
        if current_length <= max_length {
            return Ok(compressed);
        }

        let excess_length = current_length - max_length;
        let mut removed_length = 0;

        // 从低重要性内容开始移除
        let mut entries_to_remove = Vec::new();

        for (index, entry) in compressed.iter().enumerate() {
            if removed_length >= excess_length {
                break;
            }

            // 不移除系统消息、工具调用和工具结果
            if matches!(entry.entry_type,
                ContextEntryType::SystemMessage |
                ContextEntryType::ToolCall |
                ContextEntryType::ToolResult
            ) {
                continue;
            }

            // 移除低重要性内容
            if entry.importance_score < 0.6 {
                entries_to_remove.push(index);
                removed_length += entry.content.len();
            }
        }

        // 从后往前移除以保持索引有效性
        entries_to_remove.sort_by(|a, b| b.cmp(a));
        for index in entries_to_remove {
            compressed.remove(index);
        }

        Ok(compressed)
    }

    /// 截断条目内容
    fn truncate_entry(&self, entry: &ContextEntry, max_length: usize) -> ContextEntry {
        if entry.content.len() <= max_length {
            return entry.clone();
        }

        let mut truncated_content = entry.content.chars()
            .take(max_length.saturating_sub(3))
            .collect::<String>();
        truncated_content.push_str("...");

        ContextEntry {
            id: format!("{}_truncated", entry.id),
            content: truncated_content,
            importance_score: entry.importance_score * 0.8, // 截断内容重要性降低
            timestamp: entry.timestamp,
            entry_type: entry.entry_type,
            metadata: {
                let mut metadata = entry.metadata.clone();
                metadata.insert("truncated".to_string(), serde_json::json!(true));
                metadata.insert("original_length".to_string(), serde_json::json!(entry.content.len()));
                metadata
            },
        }
    }

    /// 生成类型摘要
    fn generate_type_summary(
        &self,
        entry_type: ContextEntryType,
        entries: &[&ContextEntry],
        max_length: usize,
    ) -> Result<ContextEntry, Box<dyn std::error::Error>> {
        let type_name = match entry_type {
            ContextEntryType::UserMessage => "用户消息",
            ContextEntryType::AssistantResponse => "助手响应",
            ContextEntryType::SystemMessage => "系统消息",
            ContextEntryType::ToolCall => "工具调用",
            ContextEntryType::ToolResult => "工具结果",
            ContextEntryType::CodeSnippet => "代码片段",
            ContextEntryType::FileContent => "文件内容",
            ContextEntryType::SearchResult => "搜索结果",
            ContextEntryType::Other => "其他内容",
        };

        let count = entries.len();
        let total_length: usize = entries.iter().map(|e| e.content.len()).sum();

        let summary = format!(
            "[摘要] {}: {} 条内容，总长度 {} 字符。主要内容包括: {}",
            type_name,
            count,
            total_length,
            self.generate_content_preview(entries, 200)
        );

        let truncated_summary = if summary.len() > max_length {
            format!("{}...", &summary[..max_length.saturating_sub(3)])
        } else {
            summary
        };

        Ok(ContextEntry {
            id: format!("summary_{}", entry_type as u8),
            content: truncated_summary,
            importance_score: 0.9, // 摘要有较高重要性
            timestamp: chrono::Utc::now().timestamp(),
            entry_type: ContextEntryType::Other,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("summary_type".to_string(), serde_json::json!(type_name));
                metadata.insert("original_count".to_string(), serde_json::json!(count));
                metadata.insert("total_length".to_string(), serde_json::json!(total_length));
                metadata
            },
        })
    }

    /// 生成内容预览
    fn generate_content_preview(&self, entries: &[&ContextEntry], max_preview_length: usize) -> String {
        let mut preview = String::new();

        for entry in entries.iter().take(3) { // 只预览前3个
            if preview.len() >= max_preview_length {
                break;
            }

            let snippet = if entry.content.len() > 50 {
                format!("{}...", &entry.content[..47])
            } else {
                entry.content.clone()
            };

            if !preview.is_empty() {
                preview.push_str("; ");
            }
            preview.push_str(&snippet);
        }

        if entries.len() > 3 {
            preview.push_str(&format!(" ... (还有{}条)", entries.len() - 3));
        }

        preview
    }

    /// 计算条目重要性分数
    fn calculate_importance_score(&self, entry: &ContextEntry) -> f64 {
        let mut score = entry.importance_score;

        // 基于内容类型的调整
        score *= match entry.entry_type {
            ContextEntryType::SystemMessage => 1.2, // 系统消息很重要
            ContextEntryType::ToolCall | ContextEntryType::ToolResult => 1.1, // 工具相关重要
            ContextEntryType::UserMessage => 1.0, // 用户消息基础重要性
            ContextEntryType::AssistantResponse => 0.9, // 助手响应稍低
            ContextEntryType::CodeSnippet => 1.0, // 代码中等重要性
            ContextEntryType::FileContent => 0.8, // 文件内容较低
            ContextEntryType::SearchResult => 0.7, // 搜索结果较低
            ContextEntryType::Other => 0.6, // 其他内容最低
        };

        // 基于内容长度的调整
        let content_len = entry.content.len();
        if content_len > 1000 {
            score *= 1.1; // 长内容可能更重要
        } else if content_len < 10 {
            score *= 0.8; // 太短的内容可能不重要
        }

        // 确保分数在0.0-1.0范围内
        score.max(0.0).min(1.0)
    }
}

/// 压缩策略工厂
pub struct CompressionStrategyFactory;

impl CompressionStrategyFactory {
    /// 根据上下文特征推荐压缩策略
    pub fn recommend_strategy(entries: &[ContextEntry], context_length: usize) -> CompressionStrategy {
        if context_length > 10000 {
            // 长上下文使用混合策略
            CompressionStrategy::Hybrid
        } else if entries.iter().any(|e| e.importance_score > 0.8) {
            // 有高重要性内容时使用重要性优先
            CompressionStrategy::ImportancePriority
        } else {
            // 默认使用自适应策略
            CompressionStrategy::Adaptive
        }
    }

    /// 创建压缩器
    pub fn create_compressor(strategy: CompressionStrategy) -> ContextCompressor {
        ContextCompressor::new(strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: &str, content: &str, importance: f64, entry_type: ContextEntryType) -> ContextEntry {
        ContextEntry {
            id: id.to_string(),
            content: content.to_string(),
            importance_score: importance,
            timestamp: chrono::Utc::now().timestamp(),
            entry_type,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_adaptive_compression() {
        let compressor = ContextCompressor::new(CompressionStrategy::Adaptive);

        let entries = vec![
            create_test_entry("1", "Short message", 0.5, ContextEntryType::UserMessage),
            create_test_entry("2", "Very long message that should be truncated because it's too long for the context window", 0.3, ContextEntryType::AssistantResponse),
            create_test_entry("3", "System message", 0.9, ContextEntryType::SystemMessage),
        ];

        let result = compressor.compress(&entries, 100, 0.7).await.unwrap();

        assert!(result.compressed_length <= 100);
        assert!(result.compression_ratio < 1.0);
    }

    #[tokio::test]
    async fn test_importance_priority_compression() {
        let compressor = ContextCompressor::new(CompressionStrategy::ImportancePriority);

        let entries = vec![
            create_test_entry("1", "Low importance", 0.2, ContextEntryType::UserMessage),
            create_test_entry("2", "High importance", 0.9, ContextEntryType::AssistantResponse),
            create_test_entry("3", "Medium importance", 0.6, ContextEntryType::UserMessage),
        ];

        let result = compressor.compress(&entries, 50, 0.5).await.unwrap();

        // 高重要性内容应该被保留
        assert!(result.entries.iter().any(|e| e.id == "2"));
    }

    #[test]
    fn test_strategy_recommendation() {
        let entries = vec![
            create_test_entry("1", "Normal content", 0.5, ContextEntryType::UserMessage),
            create_test_entry("2", "Important content", 0.9, ContextEntryType::SystemMessage),
        ];

        let strategy = CompressionStrategyFactory::recommend_strategy(&entries, 5000);
        match strategy {
            CompressionStrategy::ImportancePriority => {} // 应该推荐重要性优先
            _ => panic!("Expected ImportancePriority strategy"),
        }
    }
}