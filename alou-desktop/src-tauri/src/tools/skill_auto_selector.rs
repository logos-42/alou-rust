//! Skills 自动选择器
//!
//! AI根据消息内容自动选择合适的工具和Skills
//! 实现真正的自主工具调用能力

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 工具/技能匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMatch {
    /// 工具ID
    pub tool_id: String,
    /// 工具名称
    pub name: String,
    /// 匹配分数 (0-1)
    pub confidence: f64,
    /// 匹配原因
    pub reason: String,
    /// 建议的参数
    pub suggested_params: HashMap<String, serde_json::Value>,
}

/// 自动选择器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSelectConfig {
    /// 是否启用自动选择
    pub enabled: bool,
    /// 最小置信度阈值
    pub min_confidence: f64,
    /// 最大选择数量
    pub max_selections: usize,
    /// 是否允许工具链
    pub allow_tool_chain: bool,
}

impl Default for AutoSelectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence: 0.3,
            max_selections: 3,
            allow_tool_chain: true,
        }
    }
}

/// Skills自动选择器
pub struct SkillAutoSelector {
    /// 配置
    pub config: AutoSelectConfig,
    /// 工具关键词映射
    pub tool_keywords: HashMap<String, Vec<String>>,
    /// 工具描述
    pub tool_descriptions: HashMap<String, String>,
}

impl SkillAutoSelector {
    /// 创建新的选择器
    pub fn new() -> Self {
        let mut selector = Self {
            config: AutoSelectConfig::default(),
            tool_keywords: HashMap::new(),
            tool_descriptions: HashMap::new(),
        };
        
        // 初始化工具关键词映射
        selector.initialize_tool_mappings();
        
        selector
    }

    /// 初始化工具关键词映射
    fn initialize_tool_mappings(&mut self) {
        // 文件系统工具
        self.tool_keywords.insert("filesystem".to_string(), vec![
            "文件".to_string(), "读取".to_string(), "写入".to_string(),
            "创建".to_string(), "删除".to_string(), "目录".to_string(),
            "文件夹".to_string(), "复制".to_string(), "移动".to_string(),
            "file".to_string(), "read".to_string(), "write".to_string(),
            "create".to_string(), "delete".to_string(), "directory".to_string(),
        ]);
        self.tool_descriptions.insert("filesystem".to_string(), 
            "文件系统操作：读取、写入、创建、删除文件和目录".to_string());

        // 搜索工具
        self.tool_keywords.insert("search".to_string(), vec![
            "搜索".to_string(), "查找".to_string(), "grep".to_string(),
            "模式".to_string(), "匹配".to_string(), "正则".to_string(),
            "search".to_string(), "find".to_string(), "grep".to_string(),
        ]);
        self.tool_descriptions.insert("search".to_string(),
            "搜索工具：在文件中搜索文本和模式".to_string());

        // Bash工具
        self.tool_keywords.insert("bash".to_string(), vec![
            "命令".to_string(), "终端".to_string(), "shell".to_string(),
            "执行".to_string(), "运行".to_string(), "脚本".to_string(),
            "command".to_string(), "terminal".to_string(), "execute".to_string(),
            "run".to_string(), "script".to_string(),
        ]);
        self.tool_descriptions.insert("bash".to_string(),
            "Bash工具：执行终端命令和脚本".to_string());

        // 计划工具
        self.tool_keywords.insert("plan".to_string(), vec![
            "计划".to_string(), "规划".to_string(), "步骤".to_string(),
            "任务".to_string(), "目标".to_string(), "设计".to_string(),
            "plan".to_string(), "step".to_string(), "task".to_string(),
            "goal".to_string(), "design".to_string(),
        ]);
        self.tool_descriptions.insert("plan".to_string(),
            "计划工具：创建和管理任务计划".to_string());

        // 待办工具
        self.tool_keywords.insert("todolist".to_string(), vec![
            "待办".to_string(), "todo".to_string(), "清单".to_string(),
            "完成".to_string(), "进度".to_string(), "事项".to_string(),
            "todo".to_string(), "list".to_string(), "progress".to_string(),
            "item".to_string(), "complete".to_string(),
        ]);
        self.tool_descriptions.insert("todolist".to_string(),
            "待办工具：管理待办事项列表".to_string());

        // PubSub工具
        self.tool_keywords.insert("pubsub".to_string(), vec![
            "群聊".to_string(), "主题".to_string(), "消息".to_string(),
            "订阅".to_string(), "发布".to_string(), "通信".to_string(),
            "chat".to_string(), "topic".to_string(), "message".to_string(),
            "subscribe".to_string(), "publish".to_string(),
        ]);
        self.tool_descriptions.insert("pubsub".to_string(),
            "PubSub工具：发布/订阅消息通信".to_string());

        // 钱包工具
        self.tool_keywords.insert("wallet".to_string(), vec![
            "钱包".to_string(), "转账".to_string(), "签名".to_string(),
            "交易".to_string(), "ETH".to_string(), "区块链".to_string(),
            "wallet".to_string(), "transfer".to_string(), "sign".to_string(),
            "transaction".to_string(), "crypto".to_string(),
        ]);
        self.tool_descriptions.insert("wallet".to_string(),
            "钱包工具：加密货币操作和签名验证".to_string());

        // IPFS工具
        self.tool_keywords.insert("ipfs".to_string(), vec![
            "IPFS".to_string(), "去中心化".to_string(), "CID".to_string(),
            "存储".to_string(), "内容寻址".to_string(), "pin".to_string(),
            "ipfs".to_string(), "decentralized".to_string(), "storage".to_string(),
        ]);
        self.tool_descriptions.insert("ipfs".to_string(),
            "IPFS工具：去中心化存储和内容寻址".to_string());

        // Git工具
        self.tool_keywords.insert("git".to_string(), vec![
            "git".to_string(), "版本控制".to_string(), "commit".to_string(),
            "branch".to_string(), "merge".to_string(), "push".to_string(),
            "pull".to_string(), "clone".to_string(),
        ]);
        self.tool_descriptions.insert("git".to_string(),
            "Git工具：版本控制和代码管理".to_string());

        // 网络工具
        self.tool_keywords.insert("network".to_string(), vec![
            "网络".to_string(), "HTTP".to_string(), "请求".to_string(),
            "API".to_string(), "下载".to_string(), "上传".to_string(),
            "network".to_string(), "http".to_string(), "request".to_string(),
            "api".to_string(), "download".to_string(),
        ]);
        self.tool_descriptions.insert("network".to_string(),
            "网络工具：HTTP请求和网络操作".to_string());

        // 浏览器工具
        self.tool_keywords.insert("browser".to_string(), vec![
            "浏览器".to_string(), "网页".to_string(), "浏览".to_string(),
            "打开".to_string(), "浏览器".to_string(),
            "browser".to_string(), "webpage".to_string(), "open".to_string(),
        ]);
        self.tool_descriptions.insert("browser".to_string(),
            "浏览器工具：自动化网页操作".to_string());

        // Agent工具
        self.tool_keywords.insert("agent".to_string(), vec![
            "智能体".to_string(), "AI".to_string(), "对话".to_string(),
            "助手".to_string(), "agent".to_string(), "assistant".to_string(),
        ]);
        self.tool_descriptions.insert("agent".to_string(),
            "Agent工具：AI智能体对话和任务执行".to_string());

        // 回滚工具
        self.tool_keywords.insert("rollback".to_string(), vec![
            "回滚".to_string(), "撤销".to_string(), "恢复".to_string(),
            "版本".to_string(), "rollback".to_string(), "undo".to_string(),
            "restore".to_string(), "version".to_string(),
        ]);
        self.tool_descriptions.insert("rollback".to_string(),
            "回滚工具：操作撤销和版本恢复".to_string());

        // Agent Skills工具
        self.tool_keywords.insert("agent_skills".to_string(), vec![
            "技能".to_string(), "skill".to_string(), "能力".to_string(),
            "功能".to_string(), " capability".to_string(),
        ]);
        self.tool_descriptions.insert("agent_skills".to_string(),
            "Agent Skills工具：管理和执行AI技能".to_string());


        // 工具创建工具
        self.tool_keywords.insert("tool_creation".to_string(), vec![
            "创建工具".to_string(), "自定义工具".to_string(), "新工具".to_string(),
            "create tool".to_string(), "custom tool".to_string(), "new tool".to_string(),
        ]);
        self.tool_descriptions.insert("tool_creation".to_string(),
            "工具创建工具：动态创建自定义工具".to_string());

        // Iroh工具
        self.tool_keywords.insert("iroh".to_string(), vec![
            "iroh".to_string(), "点对点".to_string(), "p2p".to_string(),
            "内容分发".to_string(), "peer".to_string(),
        ]);
        self.tool_descriptions.insert("iroh".to_string(),
            "Iroh工具：点对点内容分发".to_string());

        // 消息传递工具
        self.tool_keywords.insert("message_passing".to_string(), vec![
            "消息".to_string(), "传递".to_string(), "通信".to_string(),
            "message".to_string(), "passing".to_string(), "communication".to_string(),
        ]);
        self.tool_descriptions.insert("message_passing".to_string(),
            "消息传递工具：进程间消息通信".to_string());

        // UI控制工具
        self.tool_keywords.insert("ui_control".to_string(), vec![
            "界面".to_string(), "UI".to_string(), "控制".to_string(),
            "interface".to_string(), "control".to_string(), "窗口".to_string(),
        ]);
        self.tool_descriptions.insert("ui_control".to_string(),
            "UI控制工具：应用程序界面控制".to_string());

        // 系统工具
        self.tool_keywords.insert("system".to_string(), vec![
            "系统".to_string(), "信息".to_string(), "环境".to_string(),
            "system".to_string(), "info".to_string(), "environment".to_string(),
            "CPU".to_string(), "内存".to_string(), "memory".to_string(),
        ]);
        self.tool_descriptions.insert("system".to_string(),
            "系统工具：获取系统信息和环境变量".to_string());

        // IPFS归档工具
        self.tool_keywords.insert("ipfs_archive".to_string(), vec![
            "归档".to_string(), "备份".to_string(), "archive".to_string(),
            "backup".to_string(), "快照".to_string(), "snapshot".to_string(),
        ]);
        self.tool_descriptions.insert("ipfs_archive".to_string(),
            "IPFS归档工具：数据归档和备份".to_string());

        // Agent创建工具
        self.tool_keywords.insert("agent_creator".to_string(), vec![
            "创建智能体".to_string(), "新建Agent".to_string(), "创建助手".to_string(),
            "create agent".to_string(), "new assistant".to_string(),
        ]);
        self.tool_descriptions.insert("agent_creator".to_string(),
            "Agent创建工具：创建新的AI智能体".to_string());
    }

    /// 分析消息内容，选择合适的工具
    pub fn analyze_and_select(&self, message: &str) -> Vec<ToolMatch> {
        if !self.config.enabled {
            return Vec::new();
        }

        let message_lower = message.to_lowercase();
        let mut matches: Vec<ToolMatch> = Vec::new();

        // 遍历所有工具关键词
        for (tool_id, keywords) in &self.tool_keywords {
            let mut max_confidence = 0.0f64;
            let mut matched_keyword = String::new();

            for keyword in keywords {
                if message_lower.contains(&keyword.to_lowercase()) {
                    // 计算置信度（基于关键词长度和位置）
                    let confidence = self.calculate_confidence(keyword, message);
                    if confidence > max_confidence {
                        max_confidence = confidence;
                        matched_keyword = keyword.clone();
                    }
                }
            }

            if max_confidence >= self.config.min_confidence {
                let reason = format!("检测到关键词: {}", matched_keyword);
                matches.push(ToolMatch {
                    tool_id: tool_id.clone(),
                    name: tool_descriptions_get(tool_id).unwrap_or_else(|| tool_id.clone()),
                    confidence: max_confidence,
                    reason,
                    suggested_params: HashMap::new(),
                });
            }
        }

        // 按置信度排序
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // 限制选择数量
        matches.truncate(self.config.max_selections);

        matches
    }

    /// 计算匹配置信度
    fn calculate_confidence(&self, keyword: &str, message: &str) -> f64 {
        let keyword_lower = keyword.to_lowercase();
        let message_lower = message.to_lowercase();

        // 基础分数
        let mut confidence = 0.5;

        // 关键词长度加权（更长的关键词更具体）
        confidence += (keyword.len() as f64 / 10.0).min(0.3);

        // 完全匹配加权
        if message_lower.contains(&format!(" {}", keyword_lower)) || 
           message_lower.contains(&format!("{}", keyword_lower)) {
            confidence += 0.2;
        }

        // 多个关键词匹配
        let keyword_chars: Vec<char> = keyword_lower.chars().collect();
        let message_chars: Vec<char> = message_lower.chars().collect();
        let overlap = keyword_chars.iter().filter(|c| message_chars.contains(c)).count();
        let overlap_ratio = overlap as f64 / keyword.len() as f64;
        confidence += overlap_ratio * 0.1;

        confidence.min(1.0)
    }

    /// 建议工具链
    pub fn suggest_tool_chain(&self, message: &str) -> Vec<Vec<ToolMatch>> {
        let mut chains = Vec::new();

        // 分析主需求
        let main_matches = self.analyze_and_select(message);

        if main_matches.is_empty() {
            return chains;
        }

        // 第一个工具通常是主要工具
        if let Some(first) = main_matches.first() {
            // 构建工具链
            let mut chain = vec![first.clone()];
            
            // 检查是否需要额外的辅助工具
            let secondary_tools = self.suggest_secondary_tools(&first.tool_id);
            for tool in secondary_tools {
                if self.config.allow_tool_chain {
                    chain.push(tool);
                }
            }

            chains.push(chain);
        }

        // 备选方案（不使用工具链）
        if self.config.allow_tool_chain {
            chains.push(main_matches.clone());
        }

        chains
    }

    /// 建议辅助工具
    fn suggest_secondary_tools(&self, primary_tool: &str) -> Vec<ToolMatch> {
        let mut secondary = Vec::new();

        match primary_tool {
            "bash" => {
                // Bash命令后可能需要检查结果
                secondary.push(ToolMatch {
                    tool_id: "search".to_string(),
                    name: "搜索工具".to_string(),
                    confidence: 0.3,
                    reason: "bash执行后验证结果".to_string(),
                    suggested_params: HashMap::new(),
                });
            }
            "filesystem" => {
                // 文件操作后可能需要记录
                secondary.push(ToolMatch {
                    tool_id: "todolist".to_string(),
                    name: "待办工具".to_string(),
                    confidence: 0.2,
                    reason: "文件操作后更新进度".to_string(),
                    suggested_params: HashMap::new(),
                });
            }
            "pubsub" => {
                // 群聊后可能需要保存记录
                secondary.push(ToolMatch {
                    tool_id: "filesystem".to_string(),
                    name: "文件系统".to_string(),
                    confidence: 0.2,
                    reason: "保存聊天记录".to_string(),
                    suggested_params: HashMap::new(),
                });
            }
            _ => {}
        }

        secondary
    }

    /// 生成执行计划
    pub fn generate_execution_plan(&self, message: &str) -> serde_json::Value {
        let matches = self.analyze_and_select(message);
        
        if matches.is_empty() {
            return json!({
                "success": false,
                "message": "未能识别所需工具",
                "suggestion": "请尝试更详细地描述您的需求"
            });
        }

        let plan: Vec<serde_json::Value> = matches.iter().enumerate().map(|(idx, tool)| {
            json!({
                "step": idx + 1,
                "tool": tool.tool_id,
                "action": tool.reason,
                "confidence": tool.confidence,
                "params": tool.suggested_params
            })
        }).collect();

        json!({
            "success": true,
            "primary_intent": matches[0].tool_id,
            "confidence": matches[0].confidence,
            "plan": plan,
            "total_tools": matches.len(),
            "message": format!("建议使用 {} 个工具", matches.len())
        })
    }
}

/// 辅助函数：获取工具描述
fn tool_descriptions_get(tool_id: &str) -> Option<String> {
    match tool_id {
        "filesystem" => Some("文件系统操作".to_string()),
        "search" => Some("文本搜索".to_string()),
        "bash" => Some("终端命令".to_string()),
        "plan" => Some("任务规划".to_string()),
        "todolist" => Some("待办管理".to_string()),
        "pubsub" => Some("消息通信".to_string()),
        "wallet" => Some("钱包操作".to_string()),
        "ipfs" => Some("IPFS存储".to_string()),
        "git" => Some("版本控制".to_string()),
        "network" => Some("网络请求".to_string()),
        "browser" => Some("浏览器控制".to_string()),
        "agent" => Some("AI智能体".to_string()),
        "rollback" => Some("操作回滚".to_string()),
        "agent_skills" => Some("Agent Skills".to_string()),
        "tool_creation" => Some("工具创建".to_string()),
        "iroh" => Some("Iroh P2P".to_string()),
        "message_passing" => Some("消息传递".to_string()),
        "ui_control" => Some("UI控制".to_string()),
        "system" => Some("系统信息".to_string()),
        "ipfs_archive" => Some("IPFS归档".to_string()),
        "agent_creator" => Some("Agent创建".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching() {
        let selector = SkillAutoSelector::new();
        
        // 测试文件操作
        let result = selector.analyze_and_select("请读取 config.json 文件");
        assert!(!result.is_empty());
        assert!(result.iter().any(|r| r.tool_id == "filesystem"));
        
        // 测试钱包操作
        let result = selector.analyze_and_select("请转账 1 ETH 到这个地址");
        assert!(!result.is_empty());
        assert!(result.iter().any(|r| r.tool_id == "wallet"));
    }

    #[test]
    fn test_confidence_calculation() {
        let selector = SkillAutoSelector::new();
        
        // 长关键词应该匹配度更高
        let result1 = selector.analyze_and_select("请使用 git commit 提交代码");
        let result2 = selector.analyze_and_select("请使用 git");
        
        if let Some(git_match1) = result1.iter().find(|r| r.tool_id == "git") {
            if let Some(git_match2) = result2.iter().find(|r| r.tool_id == "git") {
                assert!(git_match1.confidence >= git_match2.confidence);
            }
        }
    }

    #[test]
    fn test_execution_plan() {
        let selector = SkillAutoSelector::new();
        
        let plan = selector.generate_execution_plan("分析项目结构并创建计划");
        assert!(plan["success"].as_bool().unwrap_or(false));
        assert!(plan["plan"].is_array());
        assert!(plan["plan"].as_array().unwrap().len() > 0);
    }
}
